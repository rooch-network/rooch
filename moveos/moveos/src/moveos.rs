// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::gas::table::{
    get_gas_schedule_entries, initial_cost_schedule, ClassifiedGasMeter, CostTable, MoveOSGasMeter,
};
use crate::vm::data_cache::MoveosDataCache;
use crate::vm::moveos_vm::{MoveOSSession, MoveOSVM};
use anyhow::{bail, Result};
use backtrace::Backtrace;
use move_binary_format::binary_views::BinaryIndexedView;
use move_binary_format::errors::VMError;
use move_binary_format::errors::{vm_status_of_result, Location, PartialVMError, VMResult};
use move_binary_format::file_format::FunctionDefinitionIndex;
use move_binary_format::CompiledModule;
use move_core_types::identifier::IdentStr;
use move_core_types::language_storage::ModuleId;
use move_core_types::value::MoveTypeLayout;
use move_core_types::vm_status::{KeptVMStatus, VMStatus};
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::Identifier, vm_status::StatusCode,
};
use move_vm_runtime::config::VMConfig;
use move_vm_runtime::data_cache::TransactionCache;
use move_vm_runtime::native_functions::NativeFunction;
use moveos_store::config_store::ConfigDBStore;
use moveos_store::event_store::EventDBStore;
use moveos_store::state_store::statedb::StateDBStore;
use moveos_store::transaction_store::TransactionDBStore;
use moveos_store::MoveOSStore;
use moveos_types::addresses::MOVEOS_STD_ADDRESS;
use moveos_types::function_return_value::FunctionResult;
use moveos_types::moveos_std::event::Event;
use moveos_types::moveos_std::gas_schedule::{GasScheduleConfig, GasScheduleUpdated};
use moveos_types::moveos_std::object::ObjectMeta;
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::moveos_std::tx_result::TxResult;
use moveos_types::startup_info::StartupInfo;
use moveos_types::state::{MoveStructState, MoveStructType, ObjectState};
use moveos_types::state_resolver::RootObjectResolver;
use moveos_types::transaction::{FunctionCall, VMErrorInfo};
use moveos_types::transaction::{
    MoveOSTransaction, RawTransactionOutput, TransactionOutput, VerifiedMoveAction,
    VerifiedMoveOSTransaction,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

#[derive(Default)]
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
    pub db: MoveOSStore,
    pub cost_table: Arc<RwLock<Option<CostTable>>>,
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
        //TODO load the gas table from argument, and remove the cost_table lock.

        let vm = MoveOSVM::new(natives, config.vm_config)?;
        Ok(Self {
            vm,
            db,
            cost_table: Arc::new(RwLock::new(None)),
            system_pre_execute_functions,
            system_post_execute_functions,
        })
    }

    pub fn init_genesis(
        &self,
        genesis_tx: MoveOSTransaction,
        genesis_objects: Vec<(ObjectState, MoveTypeLayout)>,
    ) -> Result<TransactionOutput> {
        self.verify_and_execute_genesis_tx(genesis_tx, genesis_objects)
    }

    fn verify_and_execute_genesis_tx(
        &self,
        tx: MoveOSTransaction,
        genesis_objects: Vec<(ObjectState, MoveTypeLayout)>,
    ) -> Result<TransactionOutput> {
        let MoveOSTransaction { root, ctx, action } = tx;

        let resolver = RootObjectResolver::new(root, &self.db);
        let mut session = self.vm.new_genesis_session(&resolver, ctx, genesis_objects);

        let verified_action = session.verify_move_action(action).map_err(|e| {
            log::error!("verify_genesis_tx error:{:?}", e);
            e
        })?;

        // execute main tx
        let execute_result = session.execute_move_action(verified_action);
        if let Some(vm_error) = execute_result.clone().err() {
            log::error!("execute_genesis_tx vm_error:{:?}", vm_error,);
        }
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
        let output = Self::apply_transaction_output(&self.db, raw_output.clone())?;
        log::info!(
            "execute genesis tx state_root:{:?}, state_size:{}",
            output.changeset.state_root,
            output.changeset.global_size
        );
        Ok(output)
    }

    fn load_cost_table(&self, root: &ObjectMeta) -> VMResult<CostTable> {
        // We use a scoped lock here to avoid holding the lock for a long time.
        {
            let rlock = self.cost_table.read();
            if let Some(cost_table) = rlock.as_ref() {
                return Ok(cost_table.clone());
            }
        }

        if log::log_enabled!(log::Level::Trace) {
            log::trace!("load_cost_table from db");
        }
        let resolver = RootObjectResolver::new(root.clone(), &self.db);
        let gas_entries = get_gas_schedule_entries(&resolver).map_err(|e| {
            PartialVMError::new(StatusCode::STORAGE_ERROR)
                .with_message(format!("Load gas schedule entries failed: {}", e))
                .finish(Location::Undefined)
        })?;
        let cost_table = initial_cost_schedule(gas_entries);
        match self.cost_table.try_write() {
            Some(mut w) => {
                w.replace(cost_table.clone());
            }
            None => {
                log::warn!("load_cost_table try_write failed");
            }
        }
        Ok(cost_table)
    }

    pub fn state(&self) -> &StateDBStore {
        self.db.get_state_store()
    }

    pub fn moveos_store(&self) -> &MoveOSStore {
        &self.db
    }

    pub fn event_store(&self) -> &EventDBStore {
        self.db.get_event_store()
    }

    pub fn transaction_store(&self) -> &TransactionDBStore {
        self.db.get_transaction_store()
    }

    pub fn config_store(&self) -> &ConfigDBStore {
        self.db.get_config_store()
    }

    pub fn verify(&self, tx: MoveOSTransaction) -> VMResult<VerifiedMoveOSTransaction> {
        let MoveOSTransaction { root, ctx, action } = tx;
        let cost_table = self.load_cost_table(&root)?;
        let mut gas_meter = MoveOSGasMeter::new(cost_table, ctx.max_gas_amount);
        gas_meter.set_metering(false);

        let resolver = RootObjectResolver::new(root.clone(), &self.db);
        let session = self
            .vm
            .new_readonly_session(&resolver, ctx.clone(), gas_meter);

        let verified_action = session.verify_move_action(action)?;
        let (_, _) = session.finish_with_extensions(KeptVMStatus::Executed)?;
        Ok(VerifiedMoveOSTransaction {
            root,
            ctx,
            action: verified_action,
        })
    }

    pub fn execute(
        &self,
        tx: VerifiedMoveOSTransaction,
    ) -> Result<(RawTransactionOutput, Option<VMErrorInfo>)> {
        let VerifiedMoveOSTransaction { root, ctx, action } = tx;
        let tx_hash = ctx.tx_hash();
        if log::log_enabled!(log::Level::Debug) {
            log::debug!(
                "execute tx(sender:{}, hash:{}, action:{})",
                ctx.sender(),
                tx_hash,
                action
            );
        }
        let is_system_call = ctx.is_system_call();

        // When a session is respawned, all the variables in TxContext kv store will be cleaned.
        // The variables in TxContext kv store before this executions should not be cleaned,
        // So we keep a backup here, and then insert to the TxContext kv store when session respawed.
        let system_env = ctx.map.clone();

        let cost_table = self.load_cost_table(&root)?;
        let mut gas_meter = MoveOSGasMeter::new(cost_table, ctx.max_gas_amount);
        gas_meter.charge_io_write(ctx.tx_size)?;

        let resolver = RootObjectResolver::new(root, &self.db);
        let mut session = self.vm.new_session(&resolver, ctx, gas_meter);

        //We do not execute pre_execute and post_execute functions for system call
        if !is_system_call {
            // system pre_execute
            // we do not charge gas for system_pre_execute function
            session
                .execute_function_call(self.system_pre_execute_functions.clone(), false)
                .expect("system_pre_execute should not fail.");
        }

        match self.execute_action(&mut session, action.clone()) {
            Ok(status) => {
                if log::log_enabled!(log::Level::Debug) {
                    log::debug!(
                        "execute_action ok tx(hash:{}) vm_status:{:?}",
                        tx_hash,
                        status
                    );
                }
                self.execution_cleanup(is_system_call, session, status, None)
            }
            Err(vm_err) => {
                if log::log_enabled!(log::Level::Warn) {
                    log::warn!(
                        "execute_action error tx(hash:{}) vm_err:{:?} need respawn session.",
                        tx_hash,
                        vm_err
                    );
                }

                let vm_error_info = VMErrorInfo {
                    error_message: vm_err.to_string(),
                    execution_state: extract_execution_state(
                        vm_err.clone(),
                        &session.session.data_cache,
                    )?,
                };
                // If it is a system call, we should not respawn the session.
                if !is_system_call {
                    let mut s = session.respawn(system_env);
                    //Because the session is respawned, the pre_execute function should be called again.
                    s.execute_function_call(self.system_pre_execute_functions.clone(), false)
                        .expect("system_pre_execute should not fail.");
                    self.execution_cleanup(
                        is_system_call,
                        s,
                        vm_err.into_vm_status(),
                        Some(vm_error_info),
                    )
                } else {
                    self.execution_cleanup(
                        is_system_call,
                        session,
                        vm_err.into_vm_status(),
                        Some(vm_error_info),
                    )
                }
            }
        }
    }

    pub fn execute_only(
        &self,
        tx: VerifiedMoveOSTransaction,
    ) -> Result<(RawTransactionOutput, Option<VMErrorInfo>)> {
        self.execute(tx)
    }

    pub fn apply_transaction_output(
        db: &MoveOSStore,
        output: RawTransactionOutput,
    ) -> Result<TransactionOutput> {
        let RawTransactionOutput {
            status,
            mut changeset,
            events: tx_events,
            gas_used,
            is_upgrade,
            is_gas_upgrade: _,
        } = output;

        db.get_state_store()
            .apply_change_set(&mut changeset)
            .map_err(|e| {
                PartialVMError::new(StatusCode::STORAGE_ERROR)
                    .with_message(e.to_string())
                    .finish(Location::Undefined)
            })?;
        let event_ids = db
            .get_event_store()
            .save_events(tx_events.clone())
            .map_err(|e| {
                PartialVMError::new(StatusCode::STORAGE_ERROR)
                    .with_message(e.to_string())
                    .finish(Location::Undefined)
            })?;
        let events = tx_events
            .clone()
            .into_iter()
            .zip(event_ids)
            .map(|(event, event_id)| Event::new_with_event_id(event_id, event))
            .collect::<Vec<_>>();

        let new_state_root = changeset.state_root;
        let size = changeset.global_size;

        db.get_config_store()
            .save_startup_info(StartupInfo::new(new_state_root, size))
            .map_err(|e| {
                PartialVMError::new(StatusCode::STORAGE_ERROR)
                    .with_message(e.to_string())
                    .finish(Location::Undefined)
            })?;

        Ok(TransactionOutput::new(
            status, changeset, events, gas_used, is_upgrade,
        ))
    }

    /// Execute readonly view function
    pub fn execute_view_function(
        &self,
        root: ObjectMeta,
        function_call: FunctionCall,
    ) -> FunctionResult {
        //TODO allow user to specify the sender
        let tx_context = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        //TODO verify the view function
        self.execute_readonly_function(root, &tx_context, function_call)
    }

    pub fn execute_readonly_function(
        &self,
        root: ObjectMeta,
        tx_context: &TxContext,
        function_call: FunctionCall,
    ) -> FunctionResult {
        if tx_context.max_gas_amount > GasScheduleConfig::READONLY_MAX_GAS_AMOUNT {
            return FunctionResult::err(
                PartialVMError::new(StatusCode::MAX_GAS_UNITS_EXCEEDS_MAX_GAS_UNITS_BOUND)
                    .with_message("Max gas amount too large for readonly function".to_string())
                    .finish(Location::Undefined),
            );
        }
        let cost_table = match self.load_cost_table(&root) {
            Ok(cost_table) => cost_table,
            Err(e) => {
                return FunctionResult::err(e);
            }
        };
        let mut gas_meter = MoveOSGasMeter::new(cost_table, tx_context.max_gas_amount);
        gas_meter.set_metering(true);
        let resolver = RootObjectResolver::new(root, &self.db);
        let mut session = self
            .vm
            .new_readonly_session(&resolver, tx_context.clone(), gas_meter);

        let result = session.execute_function_bypass_visibility(function_call);
        match result {
            Ok(return_values) => {
                // if execute success, finish the session to check if it change the state
                match session.finish_with_extensions(KeptVMStatus::Executed) {
                    Ok(_) => FunctionResult::ok(return_values),
                    Err(e) => FunctionResult::err(e),
                }
            }
            Err(e) => {
                if log::log_enabled!(log::Level::Debug) {
                    log::warn!("execute_readonly_function error:{:?}", e);
                }
                FunctionResult::err(e)
            }
        }
    }

    // Execute action with pre_execute and post_execute.
    // Return the action execution status if success,
    // else return VMError and a bool which indicate if we should respawn the session.
    fn execute_action(
        &self,
        session: &mut MoveOSSession<'_, '_, RootObjectResolver<MoveOSStore>, MoveOSGasMeter>,
        action: VerifiedMoveAction,
    ) -> Result<VMStatus, VMError> {
        // execute main tx
        let execute_result = session.execute_move_action(action);
        let vm_status = vm_status_of_result(execute_result.clone());

        // If the user action failed, we need respawn the session,
        // then execute system_pre_execute, system_post_execute.
        let status = match vm_status.clone().keep_or_discard() {
            Ok(status) => {
                if status != KeptVMStatus::Executed {
                    debug_assert!(execute_result.is_err());
                    return Err(execute_result.unwrap_err());
                }
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

    fn execution_cleanup(
        &self,
        is_system_call: bool,
        mut session: MoveOSSession<'_, '_, RootObjectResolver<MoveOSStore>, MoveOSGasMeter>,
        status: VMStatus,
        vm_error_info: Option<VMErrorInfo>,
    ) -> Result<(RawTransactionOutput, Option<VMErrorInfo>)> {
        let kept_status = match status.keep_or_discard() {
            Ok(kept_status) => {
                if is_system_call && kept_status != KeptVMStatus::Executed {
                    // system call should always success
                    let backtrace = Backtrace::new();
                    panic!(
                        "System call failed: {:?}, vm_err: {:?} \n{:?}",
                        kept_status, vm_error_info, backtrace
                    );
                }
                kept_status
            }
            Err(discard_status) => {
                //This should not happen, if it happens, it means that the VM or verifer has a bug
                let backtrace = Backtrace::new();
                panic!("Discard status: {:?}\n{:?}", discard_status, backtrace);
            }
        };

        // update txn result to TxContext
        let gas_used = session.query_gas_used();
        let tx_result = TxResult::new(&kept_status, gas_used);
        {
            let mut runtime = session.object_runtime.write();

            runtime
                .add_to_tx_context(tx_result)
                .expect("Add tx_result to TxContext should always success");
            //We need to release the arguments before the post_execute function.
            //Because the post_execute function may use the Object in the argument.
            runtime
                .release_arguments()
                .expect("release_arguments should always success");
        }

        // We do not execute post_execute function for system call
        if !is_system_call {
            // system post_execute
            // we do not charge gas for system_post_execute function
            session
                .execute_function_call(self.system_post_execute_functions.clone(), false)
                .expect("system_post_execute should not fail.");
        }

        let mut gas_upgrade = false;
        let gas_schedule_updated = session.tx_context().get::<GasScheduleUpdated>()?;
        if let Some(_updated) = gas_schedule_updated {
            log::info!("Gas schedule updated");
            gas_upgrade = true;
            self.cost_table.write().take();
        }

        let (_ctx, mut output) = session.finish_with_extensions(kept_status)?;
        output.is_gas_upgrade = gas_upgrade;
        Ok((output, vm_error_info))
    }

    pub fn flush_module_cache(&self, is_upgrade: bool) -> Result<()> {
        if is_upgrade {
            self.vm.mark_loader_cache_as_invalid();
        };
        Ok(())
    }
}

fn extract_execution_state(
    vm_err: VMError,
    data_cache: &MoveosDataCache<RootObjectResolver<MoveOSStore>>,
) -> Result<Vec<String>> {
    let mut execution_stack_trace = Vec::new();
    if let Some(exec_state) = vm_err.exec_state() {
        for execute_record in exec_state.stack_trace() {
            match execute_record {
                (Some(module_id), func_idx, code_offset) => {
                    let func_name = func_name_from_db(module_id, func_idx, data_cache)?;
                    execution_stack_trace.push(format!(
                        "{}::{}.{}",
                        module_id.short_str_lossless(),
                        func_name,
                        code_offset
                    ));
                }
                (None, func_idx, code_offset) => {
                    execution_stack_trace.push(format!("{}::{}", func_idx, code_offset));
                }
            }
        }
    };

    Ok(execution_stack_trace)
}

fn func_name_from_db(
    module_id: &ModuleId,
    func_idx: &FunctionDefinitionIndex,
    data_cache: &MoveosDataCache<RootObjectResolver<MoveOSStore>>,
) -> Result<String> {
    let module_bytes = data_cache.load_module(module_id)?;
    let compiled_module = CompiledModule::deserialize(module_bytes.as_slice())?;
    let module_bin_view = BinaryIndexedView::Module(&compiled_module);
    let func_def = module_bin_view.function_def_at(*func_idx)?;
    Ok(module_bin_view
        .identifier_at(module_bin_view.function_handle_at(func_def.function).name)
        .to_string())
}
