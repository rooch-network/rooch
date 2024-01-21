// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::gas::table::{initial_cost_schedule, MoveOSGasMeter};
use crate::vm::moveos_vm::{MoveOSSession, MoveOSVM};
use anyhow::{bail, ensure, Result};
use backtrace::Backtrace;
use itertools::Itertools;
use move_binary_format::errors::VMError;
use move_binary_format::errors::{vm_status_of_result, Location, PartialVMError, VMResult};
use move_core_types::identifier::IdentStr;
use move_core_types::value::MoveValue;
use move_core_types::vm_status::{KeptVMStatus, VMStatus};
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::Identifier, vm_status::StatusCode,
};
use move_vm_runtime::config::VMConfig;
use move_vm_runtime::native_functions::NativeFunction;
use moveos_store::config_store::ConfigDBStore;
use moveos_store::event_store::EventDBStore;
use moveos_store::state_store::statedb::StateDBStore;
use moveos_store::transaction_store::TransactionDBStore;
use moveos_store::MoveOSStore;
use moveos_types::addresses::MOVEOS_STD_ADDRESS;
use moveos_types::function_return_value::FunctionResult;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::move_types::FunctionId;
use moveos_types::moveos_std::event::EventID;
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::moveos_std::tx_result::TxResult;
use moveos_types::startup_info::StartupInfo;
use moveos_types::state::{MoveState, MoveStructState, MoveStructType};
use moveos_types::state_resolver::MoveOSResolverProxy;
use moveos_types::transaction::{
    MoveOSTransaction, RawTransactionOutput, TransactionOutput, VerifiedMoveAction,
    VerifiedMoveOSTransaction,
};
use moveos_types::{h256::H256, transaction::FunctionCall};
use moveos_verifier::metadata::load_module_metadata;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct GasPaymentAccount {
    pub account: AccountAddress,
    pub pay_gas_by_module_account: bool,
}

impl MoveStructType for GasPaymentAccount {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("tx_context");
    const STRUCT_NAME: &'static IdentStr = ident_str!("GasPaymentAccount");
}

impl MoveStructState for GasPaymentAccount {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::Address,
            move_core_types::value::MoveTypeLayout::Bool,
        ])
    }
}

pub struct MoveOSConfig {
    pub vm_config: VMConfig,
}

impl std::fmt::Debug for MoveOSConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MoveOSConfig")
            .field(
                "vm_config.max_binary_format_version",
                &self.vm_config.max_binary_format_version,
            )
            .field(
                "vm_config.paranoid_type_checks",
                &self.vm_config.paranoid_type_checks,
            )
            .finish()
    }
}

//TODO make VMConfig cloneable and debug
impl Clone for MoveOSConfig {
    fn clone(&self) -> Self {
        Self {
            vm_config: VMConfig {
                verifier: self.vm_config.verifier.clone(),
                max_binary_format_version: self.vm_config.max_binary_format_version,
                paranoid_type_checks: self.vm_config.paranoid_type_checks,
                enable_invariant_violation_check_in_swap_loc: false,
                type_size_limit: false,
                max_value_nest_depth: None,
            },
        }
    }
}

pub struct MoveOS {
    vm: MoveOSVM,
    db: MoveOSResolverProxy<MoveOSStore>,
    system_pre_execute_functions: Vec<FunctionCall>,
    system_post_execute_functions: Vec<FunctionCall>,
}

impl MoveOS {
    pub fn new(
        db: MoveOSStore,
        natives: impl IntoIterator<Item = (AccountAddress, Identifier, Identifier, NativeFunction)>,
        config: MoveOSConfig,
        system_pre_execute_functions: Vec<FunctionCall>,
        system_post_execute_functions: Vec<FunctionCall>,
    ) -> Result<Self> {
        let vm = MoveOSVM::new(natives, config.vm_config)?;
        Ok(Self {
            vm,
            db: MoveOSResolverProxy(db),
            system_pre_execute_functions,
            system_post_execute_functions,
        })
    }

    pub fn init_genesis<
        T: Into<MoveOSTransaction>,
        GT: MoveState + Clone,
        BGT: MoveState + Clone,
    >(
        &mut self,
        genesis_txs: Vec<T>,
        genesis_ctx: GT,
        bitcoin_genesis_ctx: BGT,
    ) -> Result<Vec<(H256, TransactionOutput)>> {
        ensure!(
            self.db.0.get_state_store().is_genesis(),
            "genesis already initialized"
        );
        genesis_txs
            .into_iter()
            .map(|tx| {
                self.verify_and_execute_genesis_tx(
                    tx.into(),
                    genesis_ctx.clone(),
                    bitcoin_genesis_ctx.clone(),
                )
            })
            .collect::<Result<Vec<_>>>()
    }

    fn verify_and_execute_genesis_tx<GT: MoveState, BGT: MoveState>(
        &mut self,
        tx: MoveOSTransaction,
        genesis_ctx: GT,
        bitcoin_genesis_ctx: BGT,
    ) -> Result<(H256, TransactionOutput)> {
        let MoveOSTransaction {
            mut ctx,
            action,
            pre_execute_functions: _,
            post_execute_functions: _,
        } = tx;
        ctx.add(genesis_ctx)?;
        ctx.add(bitcoin_genesis_ctx)?;
        let mut session = self.vm.new_genesis_session(&self.db, ctx);
        let verified_action = session.verify_move_action(action)?;

        // execute main tx
        let execute_result = session.execute_move_action(verified_action);
        let status = match vm_status_of_result(execute_result.clone()).keep_or_discard() {
            Ok(status) => status,
            Err(discard_status) => {
                bail!("Discard status: {:?}", discard_status);
            }
        };

        let (_ctx, raw_output) = session.finish_with_extensions(status)?;
        if raw_output.status != KeptVMStatus::Executed {
            bail!("genesis tx should success, error: {:?}", raw_output.status);
        }
        let (state_root, event_ids) = self.apply_transaction_output(raw_output.clone())?;
        let output = TransactionOutput::new(raw_output, event_ids);
        Ok((state_root, output))
    }

    pub fn state(&self) -> &StateDBStore {
        self.db.0.get_state_store()
    }

    pub fn moveos_resolver(&self) -> &MoveOSResolverProxy<MoveOSStore> {
        &self.db
    }

    pub fn event_store(&self) -> &EventDBStore {
        self.db.0.get_event_store()
    }

    pub fn transaction_store(&self) -> &TransactionDBStore {
        self.db.0.get_transaction_store()
    }

    pub fn config_store(&self) -> &ConfigDBStore {
        self.db.0.get_config_store()
    }

    pub fn verify(&self, tx: MoveOSTransaction) -> VMResult<VerifiedMoveOSTransaction> {
        let MoveOSTransaction {
            ctx,
            action,
            pre_execute_functions,
            post_execute_functions,
        } = tx;

        let cost_table = initial_cost_schedule();
        let mut gas_meter = MoveOSGasMeter::new(cost_table, ctx.max_gas_amount);
        gas_meter.set_metering(false);
        let session = self
            .vm
            .new_readonly_session(&self.db, ctx.clone(), gas_meter);

        let verified_action = session.verify_move_action(action)?;
        let (_, _) = session.finish_with_extensions(KeptVMStatus::Executed)?;
        Ok(VerifiedMoveOSTransaction {
            ctx,
            action: verified_action,
            pre_execute_functions,
            post_execute_functions,
        })
    }

    pub fn execute(&self, tx: VerifiedMoveOSTransaction) -> Result<RawTransactionOutput> {
        let VerifiedMoveOSTransaction {
            ctx,
            action,
            pre_execute_functions,
            post_execute_functions,
        } = tx;
        let tx_hash = ctx.tx_hash();
        if log::log_enabled!(log::Level::Debug) {
            log::debug!(
                "execute tx(sender:{}, hash:{}, action:{})",
                ctx.sender(),
                tx_hash,
                action
            );
        }

        // When a session is respawned, all the variables in TxContext kv store will be cleaned.
        // The variables in TxContext kv store before this executions should not be cleaned,
        // So we keep a backup here, and then insert to the TxContext kv store when session respawed.
        let system_env = ctx.map.clone();

        let cost_table = initial_cost_schedule();
        let gas_meter = MoveOSGasMeter::new(cost_table, ctx.max_gas_amount);

        // Temporary behavior, will enable this in the future.
        // gas_meter.charge_io_write(ctx.tx_size)?;

        let mut session = self.vm.new_session(&self.db, ctx, gas_meter);

        // system pre_execute
        // we do not charge gas for system_pre_execute function
        session
            .execute_function_call(self.system_pre_execute_functions.clone(), false)
            .expect("system_pre_execute should not fail.");

        match self.execute_user_action(
            &mut session,
            action.clone(),
            pre_execute_functions.clone(),
            post_execute_functions.clone(),
        ) {
            Ok(status) => {
                if log::log_enabled!(log::Level::Debug) {
                    log::debug!(
                        "execute_user_action ok tx(hash:{}) vm_status:{:?}",
                        tx_hash,
                        status
                    );
                }
                self.execution_cleanup(session, status, Some(action))
            }
            Err((vm_err, need_respawn)) => {
                if log::log_enabled!(log::Level::Warn) {
                    log::warn!(
                        "execute_user_action error tx(hash:{}) vm_err:{:?} need_respawn:{}",
                        tx_hash,
                        vm_err,
                        need_respawn
                    );
                }
                if need_respawn {
                    let mut s = session.respawn(system_env);
                    //Because the session is respawned, the pre_execute function should be called again.
                    s.execute_function_call(self.system_pre_execute_functions.clone(), false)
                        .expect("system_pre_execute should not fail.");
                    let _ = self.execute_pre_and_post(
                        &mut s,
                        pre_execute_functions,
                        post_execute_functions,
                    );
                    // when respawn session, VM error occurs in user move action or post execution.
                    // We just cleanup with the VM error return by `execute_user_action`, ignore
                    // the result of `execute_pre_and_post`
                    // TODO: do we need to handle the result of `execute_pre_and_post` after respawn?
                    self.execution_cleanup(s, vm_err.into_vm_status(), None)
                } else {
                    self.execution_cleanup(session, vm_err.into_vm_status(), None)
                }
            }
        }
    }

    fn execute_gas_charge_post(
        &self,
        session: &mut MoveOSSession<'_, '_, MoveOSResolverProxy<MoveOSStore>, MoveOSGasMeter>,
        action: &VerifiedMoveAction,
    ) -> VMResult<Option<bool>> {
        match action {
            VerifiedMoveAction::Function { call } => {
                let module_id = &call.function_id.module_id;
                let loaded_module_bytes = session.get_data_store().load_module(module_id);

                let module_metadata = load_module_metadata(module_id, loaded_module_bytes)?;
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
                            let gas_charge_post_func_name = gas_func_info.gas_charge_post.clone();

                            let split_function =
                                gas_charge_post_func_name.split("::").collect_vec();
                            if split_function.len() != 3 {
                                return Err(PartialVMError::new(StatusCode::VM_EXTENSION_ERROR)
                                    .with_message(
                                        "The name of the gas_validate_function is incorrect."
                                            .to_string(),
                                    )
                                    .finish(Location::Module(call.clone().function_id.module_id)));
                            }
                            let real_gas_charge_post_func_name =
                                split_function.get(2).unwrap().to_string();

                            let gas_validate_func_call = FunctionCall::new(
                                FunctionId::new(
                                    call.function_id.module_id.clone(),
                                    Identifier::new(real_gas_charge_post_func_name).unwrap(),
                                ),
                                vec![],
                                vec![MoveValue::U128(session.query_gas_used() as u128)
                                    .simple_serialize()
                                    .unwrap()],
                            );

                            let return_value = session
                                .execute_function_bypass_visibility(gas_validate_func_call)?;

                            return if !return_value.is_empty() {
                                let first_return_value = return_value.get(0).unwrap();
                                Ok(Some(
                                    bcs::from_bytes::<bool>(first_return_value.value.as_slice())
                                        .expect(
                                            "the type of the return value of gas_charge_post_function should be bool",
                                        ),
                                ))
                            } else {
                                return Err(PartialVMError::new(StatusCode::VM_EXTENSION_ERROR)
                                    .with_message(
                                        "the return value of gas_charge_post_function is empty."
                                            .to_string(),
                                    )
                                    .finish(Location::Module(call.clone().function_id.module_id)));
                            };
                        }

                        Ok(None)
                    }
                }
            }
            _ => Ok(None),
        }
    }

    pub fn execute_and_apply(
        &mut self,
        tx: VerifiedMoveOSTransaction,
    ) -> Result<(H256, TransactionOutput)> {
        let raw_output = self.execute(tx)?;
        let (state_root, event_ids) = self.apply_transaction_output(raw_output.clone())?;
        let output = TransactionOutput::new(raw_output, event_ids);

        Ok((state_root, output))
    }

    fn apply_transaction_output(
        &mut self,
        output: RawTransactionOutput,
    ) -> Result<(H256, Vec<EventID>)> {
        //TODO move apply change set to a suitable place, and make MoveOS stateless?
        let RawTransactionOutput {
            status: _,
            changeset,
            state_changeset,
            events,
            gas_used: _,
            is_upgrade: _,
        } = output;
        let new_state_root = self
            .db
            .0
            .get_state_store()
            .apply_change_set(changeset, state_changeset)
            .map_err(|e| {
                PartialVMError::new(StatusCode::STORAGE_ERROR)
                    .with_message(e.to_string())
                    .finish(Location::Undefined)
            })?;
        let event_ids = self
            .db
            .0
            .get_event_store()
            .save_events(events)
            .map_err(|e| {
                PartialVMError::new(StatusCode::STORAGE_ERROR)
                    .with_message(e.to_string())
                    .finish(Location::Undefined)
            })?;
        self.db
            .0
            .get_config_store()
            .save_startup_info(StartupInfo::new(new_state_root))
            .map_err(|e| {
                PartialVMError::new(StatusCode::STORAGE_ERROR)
                    .with_message(e.to_string())
                    .finish(Location::Undefined)
            })?;
        Ok((new_state_root, event_ids))
    }

    /// Execute readonly view function
    pub fn execute_view_function(&self, function_call: FunctionCall) -> FunctionResult {
        //TODO allow user to specify the sender
        let tx_context = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        //TODO verify the view function
        self.execute_readonly_function(&tx_context, function_call)
    }

    pub fn execute_readonly_function(
        &self,
        tx_context: &TxContext,
        function_call: FunctionCall,
    ) -> FunctionResult {
        //TODO limit the view function max gas usage
        let cost_table = initial_cost_schedule();
        let mut gas_meter = MoveOSGasMeter::new(cost_table, tx_context.max_gas_amount);
        gas_meter.set_metering(false);
        let mut session = self
            .vm
            .new_readonly_session(&self.db, tx_context.clone(), gas_meter);

        let result = session.execute_function_bypass_visibility(function_call);
        match result {
            Ok(return_values) => {
                // if execute success, finish the session to check if it change the state
                match session.finish_with_extensions(KeptVMStatus::Executed) {
                    Ok(_) => FunctionResult::ok(return_values),
                    Err(e) => FunctionResult::err(e),
                }
            }
            Err(e) => FunctionResult::err(e),
        }
    }

    // Execute use action with pre_execute and post_execute.
    // Return the user action execution status if success,
    // else return VMError and a bool which indicate if we should respawn the session.
    fn execute_user_action(
        &self,
        session: &mut MoveOSSession<'_, '_, MoveOSResolverProxy<MoveOSStore>, MoveOSGasMeter>,
        action: VerifiedMoveAction,
        pre_execute_functions: Vec<FunctionCall>,
        post_execute_functions: Vec<FunctionCall>,
    ) -> Result<VMStatus, (VMError, bool)> {
        // user pre_execute
        // If the pre_execute failed, we finish the session directly and return the TransactionOutput.
        session
            .execute_function_call(pre_execute_functions, true)
            .map_err(|e| (e, false))?;

        // execute main tx
        let execute_result = session.execute_move_action(action);
        let vm_status = vm_status_of_result(execute_result.clone());

        // If the user action or post_execute failed, we need respawn the session,
        // and execute system_pre_execute, system_post_execute and user pre_execute, user post_execute.
        let status = match vm_status.clone().keep_or_discard() {
            Ok(status) => {
                if status != KeptVMStatus::Executed {
                    debug_assert!(execute_result.is_err());
                    return Err((execute_result.unwrap_err(), true));
                }
                session
                    .execute_function_call(post_execute_functions, true)
                    .map_err(|e| (e, true))?;
                vm_status
            }
            Err(discard_status) => {
                //This should not happen, if it happens, it means that the VM or verifer has a bug
                let backtrace = Backtrace::new();
                panic!(
                    "Discard status: {:?}, execute_result: {:?} \n{:?}",
                    discard_status, execute_result, backtrace
                );
            }
        };
        Ok(status)
    }

    // Execute pre_execute and post_execute only.
    fn execute_pre_and_post(
        &self,
        session: &mut MoveOSSession<'_, '_, MoveOSResolverProxy<MoveOSStore>, MoveOSGasMeter>,
        pre_execute_functions: Vec<FunctionCall>,
        post_execute_functions: Vec<FunctionCall>,
    ) -> VMResult<()> {
        session.execute_function_call(pre_execute_functions, true)?;
        session.execute_function_call(post_execute_functions, true)?;
        Ok(())
    }

    fn execution_cleanup(
        &self,
        mut session: MoveOSSession<'_, '_, MoveOSResolverProxy<MoveOSStore>, MoveOSGasMeter>,
        status: VMStatus,
        action_opt: Option<VerifiedMoveAction>,
    ) -> Result<RawTransactionOutput> {
        let kept_status = match status.keep_or_discard() {
            Ok(kept_status) => kept_status,
            Err(discard_status) => {
                //This should not happen, if it happens, it means that the VM or verifer has a bug
                let backtrace = Backtrace::new();
                panic!("Discard status: {:?}\n{:?}", discard_status, backtrace);
            }
        };

        let mut pay_gas = false;
        let gas_payment_account_opt = session
            .storage_context_mut()
            .tx_context
            .get::<GasPaymentAccount>()?;

        if let Some(gas_payment_account) = gas_payment_account_opt {
            pay_gas = gas_payment_account.pay_gas_by_module_account;
        }

        // update txn result to TxContext
        let gas_used = session.query_gas_used();
        //TODO is it a good approach to add tx_result to TxContext?
        let tx_result = TxResult::new(&kept_status, gas_used);
        session
            .storage_context_mut()
            .tx_context
            .add(tx_result)
            .expect("Add tx_result to TxContext should always success");

        // system post_execute
        // we do not charge gas for system_post_execute function
        session
            .execute_function_call(self.system_post_execute_functions.clone(), false)
            .expect("system_post_execute should not fail.");

        if pay_gas {
            self.execute_gas_charge_post(&mut session, &action_opt.unwrap())?;
        }

        let (_ctx, output) = session.finish_with_extensions(kept_status)?;
        Ok(output)
    }

    pub fn refresh_state(&self, new_state_root: H256, is_upgrade: bool) -> Result<()> {
        self.state().update_state_root(new_state_root)?;

        if is_upgrade {
            self.vm.mark_loader_cache_as_invalid();
        };
        Ok(())
    }
}

impl MoveFunctionCaller for MoveOS {
    fn call_function(
        &self,
        ctx: &TxContext,
        function_call: FunctionCall,
    ) -> Result<FunctionResult> {
        let result = self.execute_readonly_function(ctx, function_call);
        Ok(result)
    }
}
