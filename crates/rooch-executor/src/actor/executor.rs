// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::messages::{
    AnnotatedStatesMessage, ExecuteTransactionMessage, ExecuteTransactionResult,
    ExecuteViewFunctionMessage, GetEventsByEventHandleMessage, GetEventsMessage, ResolveMessage,
    StatesMessage, ValidateTransactionMessage,
};
use crate::actor::messages::{
    GetTxExecutionInfosByHashMessage, ListAnnotatedStatesMessage, ListStatesMessage,
};
use accumulator::inmemory::InMemoryAccumulator;
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use move_core_types::account_address::AccountAddress;
use move_core_types::vm_status::VMStatus;
use move_resource_viewer::MoveValueAnnotator;
use moveos::moveos::MoveOS;
use moveos::vm::vm_status_explainer::explain_vm_status;
use moveos_store::transaction_store::TransactionStore;
use moveos_store::MoveOSStore;
use moveos_types::event::AnnotatedMoveOSEvent;
use moveos_types::event::EventHandle;
use moveos_types::function_return_value::AnnotatedFunctionResult;
use moveos_types::function_return_value::AnnotatedFunctionReturnValue;
use moveos_types::genesis_info::GenesisInfo;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::move_types::as_struct_tag;
use moveos_types::state::{AnnotatedState, State};
use moveos_types::state_resolver::{AnnotatedStateReader, StateReader};
use moveos_types::transaction::FunctionCall;
use moveos_types::transaction::TransactionExecutionInfo;
use moveos_types::transaction::TransactionOutput;
use moveos_types::transaction::VerifiedMoveOSTransaction;
use moveos_types::tx_context::TxContext;
use rooch_genesis::RoochGenesis;
use rooch_store::RoochStore;
use rooch_types::address::MultiChainAddress;
use rooch_types::framework::address_mapping::AddressMapping;
use rooch_types::framework::auth_validator::AuthValidatorCaller;
use rooch_types::framework::auth_validator::TxValidateResult;
use rooch_types::framework::genesis::GenesisContext;
use rooch_types::framework::transaction_validator::TransactionValidator;
use rooch_types::framework::{system_post_execute_functions, system_pre_execute_functions};
use rooch_types::transaction::AbstractTransaction;
use rooch_types::transaction::AuthenticatorInfo;
use rooch_types::H256;

pub struct ExecutorActor {
    genesis: RoochGenesis,
    moveos: MoveOS,
    rooch_store: RoochStore,
}

type ValidateAuthenticatorResult =
    Result<(TxValidateResult, Vec<FunctionCall>, Vec<FunctionCall>), VMStatus>;

impl ExecutorActor {
    pub fn new(
        genesis_ctx: GenesisContext,
        moveos_store: MoveOSStore,
        rooch_store: RoochStore,
    ) -> Result<Self> {
        let genesis: RoochGenesis = rooch_genesis::RoochGenesis::build(genesis_ctx)?;
        let moveos = MoveOS::new(
            moveos_store,
            genesis.all_natives(),
            genesis.config.clone(),
            system_pre_execute_functions(),
            system_post_execute_functions(),
        )?;

        let executor = Self {
            genesis,
            moveos,
            rooch_store,
        };
        executor.init_or_check_genesis()
    }

    fn init_or_check_genesis(mut self) -> Result<Self> {
        if self.moveos.state().is_genesis() {
            let genesis_result = self
                .moveos
                .init_genesis(self.genesis.genesis_txs(), self.genesis.genesis_ctx())?;
            let genesis_state_root = genesis_result
                .last()
                .expect("Genesis result must not empty")
                .0;

            //TODO should we save the genesis txs to sequencer?
            for (genesis_tx, (state_root, genesis_tx_output)) in
                self.genesis.genesis_txs().into_iter().zip(genesis_result)
            {
                let tx_hash = genesis_tx.tx_hash();
                self.handle_tx_output(tx_hash, state_root, genesis_tx_output)?;
            }

            debug_assert!(
                genesis_state_root == self.genesis.genesis_state_root(),
                "Genesis state root mismatch"
            );
            let genesis_info =
                GenesisInfo::new(self.genesis.genesis_package_hash(), genesis_state_root);
            self.moveos.config_store().save_genesis(genesis_info)?;
        } else {
            self.genesis.check_genesis(self.moveos.config_store())?;
        }
        Ok(self)
    }

    pub fn resolve_or_generate(
        &self,
        multi_chain_address_sender: MultiChainAddress,
    ) -> Result<AccountAddress> {
        let resolved_sender = {
            let address_mapping = self.moveos.as_module_binding::<AddressMapping>();
            address_mapping.resovle_or_generate(multi_chain_address_sender)?
        };

        Ok(resolved_sender)
    }

    pub fn validate<T: AbstractTransaction>(&self, tx: T) -> Result<VerifiedMoveOSTransaction> {
        let multi_chain_address_sender = tx.sender();

        let resolved_sender = self.resolve_or_generate(multi_chain_address_sender.clone())?;
        let authenticator = tx.authenticator_info()?;

        let mut moveos_tx = tx.construct_moveos_transaction(resolved_sender)?;

        let vm_result = self.validate_authenticator(&moveos_tx.ctx, authenticator)?;

        match vm_result {
            Ok((tx_validate_result, pre_execute_functions, post_execute_functions)) => {
                // Add the original multichain address to the context
                moveos_tx
                    .ctx
                    .add(multi_chain_address_sender)
                    .expect("add sender to context failed");
                // Add the tx_validate_result to the context
                moveos_tx
                    .ctx
                    .add(tx_validate_result)
                    .expect("add tx_validate_result failed");

                moveos_tx.append_pre_execute_functions(pre_execute_functions);
                moveos_tx.append_post_execute_functions(post_execute_functions);
                Ok(self.moveos.verify(moveos_tx)?)
            }
            Err(e) => {
                let status_view = explain_vm_status(self.moveos.moveos_resolver(), e.clone())?;
                log::warn!(
                    "transaction validate vm error, tx_hash: {}, error:{:?}",
                    moveos_tx.ctx.tx_hash(),
                    status_view,
                );
                //TODO how to return the vm status to rpc client.
                Err(e.into())
            }
        }
    }

    pub fn validate_authenticator(
        &self,
        ctx: &TxContext,
        authenticator: AuthenticatorInfo,
    ) -> Result<ValidateAuthenticatorResult> {
        let tx_validator = self.moveos.as_module_binding::<TransactionValidator>();
        let tx_validate_function_result = tx_validator
            .validate(ctx, authenticator.clone())?
            .into_result();

        let vm_result = match tx_validate_function_result {
            Ok(tx_validate_result) => {
                let auth_validator_option = tx_validate_result.auth_validator();
                match auth_validator_option {
                    Some(auth_validator) => {
                        let auth_validator_caller =
                            AuthValidatorCaller::new(&self.moveos, auth_validator);
                        let auth_validator_function_result = auth_validator_caller
                            .validate(ctx, authenticator.authenticator.payload)?
                            .into_result();
                        match auth_validator_function_result {
                            Ok(_) => {
                                // pre_execute_function: AuthValidator
                                let pre_execute_functions =
                                    vec![auth_validator_caller.pre_execute_function_call()];
                                // post_execute_function: AuthValidator
                                let post_execute_functions =
                                    vec![auth_validator_caller.post_execute_function_call()];
                                Ok((
                                    tx_validate_result,
                                    pre_execute_functions,
                                    post_execute_functions,
                                ))
                            }
                            Err(vm_status) => Err(vm_status),
                        }
                    }
                    None => {
                        let pre_execute_functions = vec![];
                        let post_execute_functions = vec![];
                        Ok((
                            tx_validate_result,
                            pre_execute_functions,
                            post_execute_functions,
                        ))
                    }
                }
            }
            Err(vm_status) => Err(vm_status),
        };
        Ok(vm_result)
    }

    pub fn get_rooch_store(&self) -> RoochStore {
        self.rooch_store.clone()
    }

    pub fn moveos(&self) -> &MoveOS {
        &self.moveos
    }

    pub fn execute(&mut self, tx: VerifiedMoveOSTransaction) -> Result<ExecuteTransactionResult> {
        let tx_hash = tx.ctx.tx_hash();
        let (state_root, output) = self.moveos.execute_and_apply(tx)?;
        self.handle_tx_output(tx_hash, state_root, output)
    }

    fn handle_tx_output(
        &mut self,
        tx_hash: H256,
        state_root: H256,
        output: TransactionOutput,
    ) -> Result<ExecuteTransactionResult> {
        let event_hashes: Vec<_> = output.events.iter().map(|e| e.hash()).collect();
        let event_root = InMemoryAccumulator::from_leaves(event_hashes.as_slice()).root_hash();

        let transaction_info = TransactionExecutionInfo::new(
            tx_hash,
            state_root,
            event_root,
            output.gas_used,
            output.status.clone(),
        );
        self.moveos
            .transaction_store()
            .save_tx_execution_info(transaction_info.clone())
            .map_err(|e| {
                anyhow::anyhow!(
                    "ExecuteTransactionMessage handler save tx info failed: {:?} {}",
                    transaction_info,
                    e
                )
            })?;
        Ok(ExecuteTransactionResult {
            output,
            transaction_info,
        })
    }
}

impl Actor for ExecutorActor {}

#[async_trait]
impl<T> Handler<ValidateTransactionMessage<T>> for ExecutorActor
where
    T: 'static + AbstractTransaction + Send + Sync,
{
    async fn handle(
        &mut self,
        msg: ValidateTransactionMessage<T>,
        _ctx: &mut ActorContext,
    ) -> Result<VerifiedMoveOSTransaction> {
        self.validate(msg.tx)
    }
}

#[async_trait]
impl Handler<ExecuteTransactionMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: ExecuteTransactionMessage,
        _ctx: &mut ActorContext,
    ) -> Result<ExecuteTransactionResult> {
        self.execute(msg.tx)
    }
}

#[async_trait]
impl Handler<ExecuteViewFunctionMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: ExecuteViewFunctionMessage,
        _ctx: &mut ActorContext,
    ) -> Result<AnnotatedFunctionResult, anyhow::Error> {
        let resoler = self.moveos.moveos_resolver();

        let function_result = self.moveos.execute_view_function(msg.call);
        Ok(AnnotatedFunctionResult {
            vm_status: function_result.vm_status,
            return_values: match function_result.return_values {
                Some(values) => Some(
                    values
                        .into_iter()
                        .map(|v| {
                            let move_value = resoler.view_value(&v.type_tag, &v.value)?;
                            Ok(AnnotatedFunctionReturnValue {
                                value: v,
                                move_value,
                            })
                        })
                        .collect::<Result<Vec<AnnotatedFunctionReturnValue>, anyhow::Error>>()?,
                ),
                None => None,
            },
        })
    }
}

#[async_trait]
impl Handler<ResolveMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: ResolveMessage,
        _ctx: &mut ActorContext,
    ) -> Result<AccountAddress, anyhow::Error> {
        self.resolve_or_generate(msg.address)
    }
}

#[async_trait]
impl Handler<StatesMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: StatesMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<State>>, anyhow::Error> {
        let statedb = self.moveos.moveos_resolver();
        statedb.get_states(msg.access_path)
    }
}

#[async_trait]
impl Handler<AnnotatedStatesMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: AnnotatedStatesMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<AnnotatedState>>, anyhow::Error> {
        let statedb = self.moveos.moveos_resolver();
        statedb.get_annotated_states(msg.access_path)
    }
}

#[async_trait]
impl Handler<ListStatesMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: ListStatesMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<(Vec<u8>, State)>>, anyhow::Error> {
        let statedb = self.moveos.moveos_resolver();
        statedb.list_states(msg.access_path, msg.cursor, msg.limit)
    }
}

#[async_trait]
impl Handler<ListAnnotatedStatesMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: ListAnnotatedStatesMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<(Vec<u8>, AnnotatedState)>>, anyhow::Error> {
        let statedb = self.moveos.moveos_resolver();
        statedb.list_annotated_states(msg.access_path, msg.cursor, msg.limit)
    }
}

#[async_trait]
impl Handler<GetEventsByEventHandleMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: GetEventsByEventHandleMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<AnnotatedMoveOSEvent>>> {
        let GetEventsByEventHandleMessage {
            event_handle_type,
            cursor,
            limit,
        } = msg;
        let event_store = self.moveos.event_store();
        let resolver = self.moveos.moveos_resolver();

        let event_handle_id = EventHandle::derive_event_handle_id(event_handle_type.clone());
        let events = event_store.get_events_by_event_handle_id(&event_handle_id, cursor, limit)?;

        // for ev in events
        let result = events
            .into_iter()
            // .enumerate()
            .map(|event| {
                let state = State::new(event.event_data.clone(), event.type_tag.clone());
                let annotated_event_data = MoveValueAnnotator::new(resolver)
                    .view_resource(&event_handle_type, state.value.as_slice())
                    .unwrap();
                Some(AnnotatedMoveOSEvent::new(
                    event,
                    annotated_event_data,
                    None,
                    None,
                ))
            })
            .collect();
        Ok(result)
    }
}

#[async_trait]
impl Handler<GetEventsMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: GetEventsMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<AnnotatedMoveOSEvent>>> {
        let GetEventsMessage { filter } = msg;
        let event_store = self.moveos.event_store();
        let resolver = self.moveos.moveos_resolver();
        //TODO handle tx hash
        let mut result: Vec<Option<AnnotatedMoveOSEvent>> = Vec::new();
        let events = event_store.get_events_with_filter(filter)?;
        for ev in events
            .into_iter()
            .enumerate()
            .map(|(_i, event)| {
                let state = State::new(event.event_data.clone(), event.type_tag.clone());
                let struct_tag = as_struct_tag(event.type_tag.clone()).unwrap();
                let annotated_event_data = MoveValueAnnotator::new(resolver)
                    .view_resource(&struct_tag, state.value.as_slice())
                    .unwrap();
                AnnotatedMoveOSEvent::new(event, annotated_event_data, None, None)
            })
            .collect::<Vec<_>>()
        {
            result.push(Some(ev));
        }
        Ok(result)
    }
}

#[async_trait]
impl Handler<GetTxExecutionInfosByHashMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: GetTxExecutionInfosByHashMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<TransactionExecutionInfo>>> {
        let GetTxExecutionInfosByHashMessage { tx_hashes } = msg;
        self.moveos
            .transaction_store()
            .multi_get_tx_execution_infos(tx_hashes)
    }
}
