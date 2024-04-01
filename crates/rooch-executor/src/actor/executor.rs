// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::messages::{
    ExecuteTransactionMessage, ExecuteTransactionResult, ResolveMessage, ValidateL1BlockMessage,
    ValidateL2TxMessage,
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
use moveos::gas::table::{get_gas_schedule_entries, initial_cost_schedule, MoveOSGasMeter};
use moveos::moveos::MoveOS;
use moveos::vm::vm_status_explainer::explain_vm_status;
use moveos_store::transaction_store::TransactionStore;
use moveos_store::MoveOSStore;
use moveos_types::genesis_info::GenesisInfo;
use moveos_types::h256::H256;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::move_types::FunctionId;
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::state_resolver::MoveOSResolverProxy;
use moveos_types::transaction::TransactionOutput;
use moveos_types::transaction::VerifiedMoveOSTransaction;
use moveos_types::transaction::{
    FunctionCall, MoveOSTransaction, TransactionExecutionInfo, VerifiedMoveAction,
};
use moveos_verifier::metadata::load_module_metadata;
use rooch_framework::natives::gas_parameter::gas_member::FromOnChainGasSchedule;
use rooch_genesis::RoochGenesis;
use rooch_store::RoochStore;
use rooch_types::address::MultiChainAddress;
use rooch_types::bitcoin::genesis::BitcoinGenesisContext;
use rooch_types::bitcoin::light_client::BitcoinLightClientModule;
use rooch_types::framework::address_mapping::AddressMapping;
use rooch_types::framework::auth_validator::{AuthValidatorCaller, TxValidateResult};
use rooch_types::framework::ethereum_light_client::EthereumLightClientModule;
use rooch_types::framework::genesis::GenesisContext;
use rooch_types::framework::transaction_validator::TransactionValidator;
use rooch_types::framework::{system_post_execute_functions, system_pre_execute_functions};
use rooch_types::multichain_id::RoochMultiChainID;
use rooch_types::transaction::{AuthenticatorInfo, L1Block, L1BlockWithBody, RoochTransaction};

pub struct ExecutorActor {
    genesis: RoochGenesis,
    moveos: MoveOS,
    rooch_store: RoochStore,
}

type ValidateAuthenticatorResult = Result<
    (
        TxValidateResult,
        Option<MultiChainAddress>,
        Vec<FunctionCall>,
        Vec<FunctionCall>,
    ),
    VMStatus,
>;

impl ExecutorActor {
    pub fn new(
        genesis_ctx: GenesisContext,
        bitcoin_genesis_ctx: BitcoinGenesisContext,
        moveos_store: MoveOSStore,
        rooch_store: RoochStore,
    ) -> Result<Self> {
        let mut genesis: RoochGenesis = RoochGenesis::build(genesis_ctx, bitcoin_genesis_ctx)?;

        let gas_schedule_entries =
            get_gas_schedule_entries(&MoveOSResolverProxy(moveos_store.clone()));
        if let Some(gas_entries) = gas_schedule_entries {
            if let Some(gas_parameters) =
                rooch_framework::natives::NativeGasParameters::from_on_chain_gas_schedule(
                    &gas_entries,
                )
            {
                genesis.rooch_framework_gas_params = gas_parameters;
            }
        }

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
        if self.moveos().state().is_genesis() {
            let genesis_result = self.moveos.init_genesis(
                self.genesis.genesis_txs(),
                self.genesis.genesis_ctx(),
                self.genesis.bitcoin_genesis_ctx(),
            )?;
            let genesis_state_root = genesis_result
                .last()
                .expect("Genesis result must not empty")
                .0;

            //TODO should we save the genesis txs to sequencer?
            for (genesis_tx, (state_root, size, genesis_tx_output)) in
                self.genesis.genesis_txs().into_iter().zip(genesis_result)
            {
                let tx_hash = genesis_tx.tx_hash();
                self.handle_tx_output(tx_hash, state_root, size, genesis_tx_output)?;
            }

            debug_assert!(
                genesis_state_root == self.genesis.genesis_state_root(),
                "Genesis state root mismatch"
            );
            let genesis_info =
                GenesisInfo::new(self.genesis.genesis_package_hash(), genesis_state_root);
            self.moveos().config_store().save_genesis(genesis_info)?;
        } else {
            self.genesis.check_genesis(self.moveos().config_store())?;
        }
        Ok(self)
    }

    pub fn get_rooch_store(&self) -> RoochStore {
        self.rooch_store.clone()
    }

    pub fn get_moveos_store(&self) -> MoveOSStore {
        self.moveos.moveos_store().clone()
    }

    pub fn moveos(&self) -> &MoveOS {
        &self.moveos
    }

    pub fn genesis(&self) -> &RoochGenesis {
        &self.genesis
    }

    pub fn resolve_or_generate(
        &self,
        multi_chain_address_sender: MultiChainAddress,
    ) -> Result<AccountAddress> {
        let resolved_sender = {
            let address_mapping = self.moveos().as_module_binding::<AddressMapping>();
            address_mapping.resolve_or_generate(multi_chain_address_sender)?
        };

        Ok(resolved_sender)
    }

    pub fn execute(&mut self, tx: VerifiedMoveOSTransaction) -> Result<ExecuteTransactionResult> {
        let tx_hash = tx.ctx.tx_hash();
        let (state_root, size, output) = self.moveos.execute_and_apply(tx)?;
        self.handle_tx_output(tx_hash, state_root, size, output)
    }

    fn handle_tx_output(
        &mut self,
        tx_hash: H256,
        state_root: H256,
        size: u64,
        output: TransactionOutput,
    ) -> Result<ExecuteTransactionResult> {
        if log::log_enabled!(log::Level::Debug) {
            log::debug!(
                "tx_hash: {}, state_root: {}, size: {}, gas_used: {}, status: {:?}",
                tx_hash,
                state_root,
                size,
                output.gas_used,
                output.status
            );
        }
        let event_hashes: Vec<_> = output.events.iter().map(|e| e.hash()).collect();
        let event_root = InMemoryAccumulator::from_leaves(event_hashes.as_slice()).root_hash();

        let transaction_info = TransactionExecutionInfo::new(
            tx_hash,
            state_root,
            size,
            event_root,
            output.gas_used,
            output.status.clone(),
        );
        self.moveos()
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

    pub fn validate_l1_block(
        &self,
        ctx: TxContext,
        l1_block: L1BlockWithBody,
    ) -> Result<VerifiedMoveOSTransaction> {
        //In the future, we should verify the block PoW difficulty or PoS validator signature before the sequencer decentralized
        let L1BlockWithBody {
            block:
                L1Block {
                    chain_id,
                    block_height,
                    block_hash,
                },
            block_body,
        } = l1_block;
        match RoochMultiChainID::try_from(chain_id.id())? {
            RoochMultiChainID::Bitcoin => {
                let action = VerifiedMoveAction::Function {
                    call: BitcoinLightClientModule::create_submit_new_block_call_bytes(
                        block_height,
                        block_hash,
                        block_body,
                    )?,
                    bypass_visibility: true,
                };
                Ok(VerifiedMoveOSTransaction::new(ctx, action))
            }
            RoochMultiChainID::Ether => {
                let action = VerifiedMoveAction::Function {
                    call: EthereumLightClientModule::create_submit_new_block_call_bytes(block_body),
                    bypass_visibility: true,
                };
                Ok(VerifiedMoveOSTransaction::new(ctx, action))
            }
            id => Err(anyhow::anyhow!("Chain {} not supported yet", id)),
        }
    }

    pub fn validate_l2_tx(&self, tx: RoochTransaction) -> Result<VerifiedMoveOSTransaction> {
        let sender = tx.sender();

        let authenticator = tx.authenticator_info()?;

        let mut moveos_tx: MoveOSTransaction = tx.into();

        let vm_result = self.validate_authenticator(&moveos_tx.ctx, authenticator)?;

        match vm_result {
            Ok((
                tx_validate_result,
                multi_chain_address,
                pre_execute_functions,
                post_execute_functions,
            )) => {
                // Add the original multichain address to the context
                moveos_tx
                    .ctx
                    .add(multi_chain_address.unwrap_or(sender.into()))
                    .expect("add sender to context failed");

                // Add the tx_validate_result to the context
                moveos_tx
                    .ctx
                    .add(tx_validate_result)
                    .expect("add tx_validate_result failed");

                moveos_tx.append_pre_execute_functions(pre_execute_functions);
                moveos_tx.append_post_execute_functions(post_execute_functions);
                Ok(self.moveos().verify(moveos_tx)?)
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
        let tx_validator = self.moveos().as_module_binding::<TransactionValidator>();
        let tx_validate_function_result = tx_validator
            .validate(ctx, authenticator.clone())?
            .into_result();

        let vm_result = match tx_validate_function_result {
            Ok(tx_validate_result) => {
                let auth_validator_option = tx_validate_result.auth_validator();
                match auth_validator_option {
                    Some(auth_validator) => {
                        let auth_validator_caller =
                            AuthValidatorCaller::new(self.moveos(), auth_validator);
                        let auth_validator_function_result = auth_validator_caller
                            .validate(ctx, authenticator.authenticator.payload)?
                            .into_result();
                        match auth_validator_function_result {
                            Ok(multi_chain_address) => {
                                // pre_execute_function: AuthValidator
                                let pre_execute_functions =
                                    vec![auth_validator_caller.pre_execute_function_call()];
                                // post_execute_function: AuthValidator
                                let post_execute_functions =
                                    vec![auth_validator_caller.post_execute_function_call()];
                                Ok((
                                    tx_validate_result,
                                    multi_chain_address,
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
                            None,
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

        let gas_entries = get_gas_schedule_entries(self.moveos.moveos_resolver());
        let cost_table = initial_cost_schedule(gas_entries);
        let mut gas_meter = MoveOSGasMeter::new(cost_table, ctx.max_gas_amount);
        gas_meter.set_metering(false);

        let verified_moveos_action = self.moveos().verify(tx.clone())?;
        let verified_action = verified_moveos_action.action;

        match verified_action {
            VerifiedMoveAction::Function {
                call,
                bypass_visibility: _,
            } => {
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
                                self.moveos.execute_view_function(gas_validate_func_call);

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

        let gas_entries = get_gas_schedule_entries(self.moveos.moveos_resolver());
        let cost_table = initial_cost_schedule(gas_entries);
        let mut gas_meter = MoveOSGasMeter::new(cost_table, ctx.max_gas_amount);
        gas_meter.set_metering(false);

        let verified_moveos_action = self.moveos().verify(tx.clone())?;
        let verified_action = verified_moveos_action.action;

        match verified_action {
            VerifiedMoveAction::Function {
                call,
                bypass_visibility: _,
            } => {
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
                    self.moveos.execute_view_function(gas_balance_func_call);

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
}

impl Actor for ExecutorActor {}

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
impl Handler<ValidateL2TxMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: ValidateL2TxMessage,
        _ctx: &mut ActorContext,
    ) -> Result<VerifiedMoveOSTransaction> {
        self.validate_l2_tx(msg.tx)
    }
}

#[async_trait]
impl Handler<ValidateL1BlockMessage> for ExecutorActor {
    async fn handle(
        &mut self,
        msg: ValidateL1BlockMessage,
        _ctx: &mut ActorContext,
    ) -> Result<VerifiedMoveOSTransaction> {
        self.validate_l1_block(msg.ctx, msg.l1_block)
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
