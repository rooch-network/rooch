// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::messages::{
    AnnotatedStatesMessage, ExecuteTransactionMessage, ExecuteTransactionResult,
    ExecuteViewFunctionMessage, GetEventsByEventHandleMessage, ResolveMessage, StatesMessage,
    ValidateTransactionMessage,
};
use crate::actor::messages::{
    GetEventsByEventIDsMessage, GetTxExecutionInfosByHashMessage, ListAnnotatedStatesMessage,
    ListStatesMessage,
};
use accumulator::inmemory::InMemoryAccumulator;
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use itertools::Itertools;
use move_binary_format::errors::{Location, PartialVMError, VMResult};
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::{IdentStr, Identifier};
use move_core_types::language_storage::ModuleId;
use move_core_types::resolver::ModuleResolver;
use move_core_types::value::MoveValue;
use move_core_types::vm_status::{StatusCode, VMStatus};
use move_resource_viewer::MoveValueAnnotator;
use moveos::gas::table::{initial_cost_schedule, MoveOSGasMeter};
use moveos::moveos::{GasPaymentAccount, MoveOS};
use moveos::vm::vm_status_explainer::explain_vm_status;
use moveos_store::transaction_store::TransactionStore;
use moveos_store::MoveOSStore;
use moveos_types::function_return_value::AnnotatedFunctionResult;
use moveos_types::function_return_value::AnnotatedFunctionReturnValue;
use moveos_types::genesis_info::GenesisInfo;
use moveos_types::h256::H256;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::move_types::FunctionId;
use moveos_types::moveos_std::event::AnnotatedEvent;
use moveos_types::moveos_std::event::EventHandle;
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::state::{AnnotatedState, State};
use moveos_types::state_resolver::{AnnotatedStateReader, StateReader};
use moveos_types::transaction::VerifiedMoveOSTransaction;
use moveos_types::transaction::{FunctionCall, MoveAction};
use moveos_types::transaction::{MoveOSTransaction, TransactionExecutionInfo};
use moveos_types::transaction::{TransactionOutput, VerifiedMoveAction};
use moveos_verifier::metadata::load_module_metadata;
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
            address_mapping.resolve_or_generate(multi_chain_address_sender)?
        };

        Ok(resolved_sender)
    }

    pub fn validate<T: AbstractTransaction>(&self, tx: T) -> Result<VerifiedMoveOSTransaction> {
        let multi_chain_address_sender = tx.sender();

        let resolved_sender = self.resolve_or_generate(multi_chain_address_sender.clone())?;
        let authenticator = tx.authenticator_info()?;

        let mut moveos_tx = tx.construct_moveos_transaction(resolved_sender)?;

        let vm_result = self.validate_authenticator(&moveos_tx.ctx, authenticator)?;

        let can_pay_gas = self.validate_gas_function(&moveos_tx)?;

        let mut pay_by_module_account = false;
        let mut gas_payment_account = moveos_tx.ctx.sender;

        if let Some(pay_gas) = can_pay_gas {
            if pay_gas {
                let account_balance = self.get_account_balance(&moveos_tx)?;
                let module_account = {
                    match &moveos_tx.action {
                        MoveAction::Function(call) => Some(*call.function_id.module_id.address()),
                        _ => None,
                    }
                };

                let gas_payment_address = {
                    if account_balance >= moveos_tx.ctx.max_gas_amount as u128 {
                        pay_by_module_account = true;
                        module_account.unwrap()
                    } else {
                        moveos_tx.ctx.sender
                    }
                };

                gas_payment_account = gas_payment_address;
            }
        }

        moveos_tx
            .ctx
            .add(GasPaymentAccount {
                account: gas_payment_account,
                pay_gas_by_module_account: pay_by_module_account,
            })
            .expect("adding GasPaymentAccount to tx context failed.");

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

    pub fn validate_gas_function(&self, tx: &MoveOSTransaction) -> VMResult<Option<bool>> {
        let MoveOSTransaction { ctx, .. } = tx;

        let cost_table = initial_cost_schedule();
        let mut gas_meter = MoveOSGasMeter::new(cost_table, ctx.max_gas_amount);
        gas_meter.set_metering(false);

        let verified_moveos_action = self.moveos().verify(tx.clone())?;
        let verified_action = verified_moveos_action.action;

        match verified_action {
            VerifiedMoveAction::Function { call } => {
                let module_id = &call.function_id.module_id;
                let loaded_module_bytes_result =
                    self.moveos().moveos_resolver().get_module(module_id);
                let loaded_module_bytes = match loaded_module_bytes_result {
                    Ok(loaded_module_bytes_opt) => match loaded_module_bytes_opt {
                        None => {
                            return Err(PartialVMError::new(StatusCode::RESOURCE_DOES_NOT_EXIST)
                                .with_message(
                                    "The name of the gas_validate_function does not exist."
                                        .to_string(),
                                )
                                .finish(Location::Module(module_id.clone())));
                        }
                        Some(module_bytes) => module_bytes,
                    },
                    Err(error) => {
                        return Err(PartialVMError::new(StatusCode::RESOURCE_DOES_NOT_EXIST)
                            .with_message(format!(
                                "Load module data from module_id {:} was failed {:}.",
                                module_id.clone(),
                                error
                            ))
                            .finish(Location::Module(module_id.clone())));
                    }
                };

                let module_metadata = load_module_metadata(module_id, Ok(loaded_module_bytes))?;
                let gas_free_function_info = {
                    match module_metadata {
                        None => None,
                        Some(runtime_metadata) => Some(runtime_metadata.gas_free_function_map),
                    }
                };

                let called_function_name = call.function_id.function_name.to_string();
                match gas_free_function_info {
                    None => Ok(None),
                    Some(gas_func_info) => {
                        let full_called_function = format!(
                            "0x{}::{}::{}",
                            call.function_id.module_id.address().to_hex(),
                            call.function_id.module_id.name(),
                            called_function_name
                        );
                        let gas_func_info_opt = gas_func_info.get(&full_called_function);

                        if let Some(gas_func_info) = gas_func_info_opt {
                            let gas_validate_func_name = gas_func_info.gas_validate.clone();

                            let split_function = gas_validate_func_name.split("::").collect_vec();
                            if split_function.len() != 3 {
                                return Err(PartialVMError::new(StatusCode::VM_EXTENSION_ERROR)
                                    .with_message(
                                        "The name of the gas_validate_function is incorrect."
                                            .to_string(),
                                    )
                                    .finish(Location::Module(call.clone().function_id.module_id)));
                            }
                            let real_gas_validate_func_name =
                                split_function.get(2).unwrap().to_string();

                            let gas_validate_func_call = FunctionCall::new(
                                FunctionId::new(
                                    call.function_id.module_id.clone(),
                                    Identifier::new(real_gas_validate_func_name).unwrap(),
                                ),
                                vec![],
                                vec![],
                            );

                            let function_execution_result =
                                self.moveos().execute_view_function(gas_validate_func_call);

                            return if function_execution_result.vm_status == VMStatus::Executed {
                                let return_value = function_execution_result.return_values.unwrap();
                                if !return_value.is_empty() {
                                    let first_return_value = return_value.get(0).unwrap();
                                    Ok(Some(
                                        bcs::from_bytes::<bool>(first_return_value.value.as_slice())
                                            .expect(
                                                "the return value of gas validate function should be bool",
                                            ),
                                    ))
                                } else {
                                    return Err(PartialVMError::new(
                                        StatusCode::VM_EXTENSION_ERROR,
                                    )
                                    .with_message(
                                        "the return value of gas_validate_function is empty."
                                            .to_string(),
                                    )
                                    .finish(Location::Module(call.clone().function_id.module_id)));
                                }
                            } else {
                                Ok(None)
                            };
                        };

                        Ok(None)
                    }
                }
            }
            _ => Ok(None),
        }
    }

    pub fn get_account_balance(&self, tx: &MoveOSTransaction) -> VMResult<u128> {
        let MoveOSTransaction { ctx, .. } = tx;

        let cost_table = initial_cost_schedule();
        let mut gas_meter = MoveOSGasMeter::new(cost_table, ctx.max_gas_amount);
        gas_meter.set_metering(false);

        let verified_moveos_action = self.moveos().verify(tx.clone())?;
        let verified_action = verified_moveos_action.action;

        match verified_action {
            VerifiedMoveAction::Function { call } => {
                let module_address = call.function_id.module_id.address();

                let gas_coin_module_id = ModuleId::new(
                    AccountAddress::from_hex_literal("0x3").unwrap(),
                    Identifier::from(IdentStr::new("gas_coin").unwrap()),
                );
                let gas_balance_func_call = FunctionCall::new(
                    FunctionId::new(gas_coin_module_id, Identifier::new("balance").unwrap()),
                    vec![],
                    vec![MoveValue::Address(*module_address)
                        .simple_serialize()
                        .unwrap()],
                );

                let function_execution_result =
                    self.moveos().execute_view_function(gas_balance_func_call);

                if function_execution_result.vm_status == VMStatus::Executed {
                    let return_value = function_execution_result.return_values.unwrap();
                    let first_return_value = return_value.get(0).unwrap();

                    let balance = bcs::from_bytes::<move_core_types::u256::U256>(
                        first_return_value.value.as_slice(),
                    )
                    .expect("the return value of gas validate function should be u128");

                    Ok(balance.unchecked_as_u128())
                } else {
                    Ok(0)
                }
            }
            _ => Ok(0),
        }
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
                            let decoded_value = resoler.view_value(&v.type_tag, &v.value)?;
                            Ok(AnnotatedFunctionReturnValue {
                                value: v,
                                decoded_value,
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
    ) -> Result<Vec<(Vec<u8>, State)>, anyhow::Error> {
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
    ) -> Result<Vec<(Vec<u8>, AnnotatedState)>, anyhow::Error> {
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
    ) -> Result<Vec<AnnotatedEvent>> {
        let GetEventsByEventHandleMessage {
            event_handle_type,
            cursor,
            limit,
        } = msg;
        let event_store = self.moveos.event_store();
        let resolver = self.moveos.moveos_resolver();

        let event_handle_id = EventHandle::derive_event_handle_id(&event_handle_type);
        let events = event_store.get_events_by_event_handle_id(&event_handle_id, cursor, limit)?;

        events
            .into_iter()
            .map(|event| {
                let event_move_value = MoveValueAnnotator::new(resolver)
                    .view_resource(&event_handle_type, event.event_data())?;
                Ok(AnnotatedEvent::new(event, event_move_value))
            })
            .collect::<Result<Vec<_>>>()
    }
}

#[async_trait]
impl Handler<GetEventsByEventIDsMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: GetEventsByEventIDsMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<AnnotatedEvent>>> {
        let GetEventsByEventIDsMessage { event_ids } = msg;
        let event_store = self.moveos.event_store();
        let resolver = self.moveos.moveos_resolver();

        event_store
            .multi_get_events(event_ids)?
            .into_iter()
            .map(|v| match v {
                Some(event) => {
                    let event_move_value = MoveValueAnnotator::new(resolver)
                        .view_resource(event.event_type(), event.event_data())?;
                    Ok(Some(AnnotatedEvent::new(event, event_move_value)))
                }
                None => Ok(None),
            })
            .collect::<Result<Vec<_>>>()
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
