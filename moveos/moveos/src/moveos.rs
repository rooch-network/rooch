// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::gas::table::{
    get_gas_schedule_entries, initial_cost_schedule, CostTable, MoveOSGasMeter,
};
use crate::vm::data_cache::MoveosDataCache;
use crate::vm::module_cache::{GlobalModuleCache, RoochModuleExtension};
use crate::vm::moveos_vm::{MoveOSSession, MoveOSVM};
use anyhow::{bail, format_err, Error, Result};
use move_binary_format::binary_views::BinaryIndexedView;
use move_binary_format::deserializer::DeserializerConfig;
use move_binary_format::errors::VMError;
use move_binary_format::errors::{vm_status_of_result, Location, PartialVMError, VMResult};
use move_binary_format::file_format::FunctionDefinitionIndex;
use move_binary_format::CompiledModule;
use move_bytecode_verifier::VerifierConfig;
use move_core_types::identifier::IdentStr;
use move_core_types::language_storage::ModuleId;
use move_core_types::value::MoveTypeLayout;
use move_core_types::vm_status::{KeptVMStatus, VMStatus};
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::Identifier, vm_status::StatusCode,
};
use move_vm_runtime::config::{VMConfig, DEFAULT_MAX_VALUE_NEST_DEPTH};
use move_vm_runtime::data_cache::TransactionCache;
use move_vm_runtime::native_functions::NativeFunction;
use move_vm_runtime::{Module, RuntimeEnvironment};
use move_vm_types::loaded_data::runtime_types::TypeBuilder;
use moveos_common::types::ClassifiedGasMeter;
use moveos_store::config_store::ConfigDBStore;
use moveos_store::event_store::EventDBStore;
use moveos_store::state_store::statedb::StateDBStore;
use moveos_store::transaction_store::TransactionDBStore;
use moveos_store::{load_feature_store_object, MoveOSStore};
use moveos_types::addresses::MOVEOS_STD_ADDRESS;
use moveos_types::function_return_value::FunctionResult;
use moveos_types::moveos_std::gas_schedule::{GasScheduleConfig, GasScheduleUpdated};
use moveos_types::moveos_std::object::ObjectMeta;
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::moveos_std::tx_result::TxResult;
use moveos_types::state::{MoveStructState, MoveStructType, ObjectState};
use moveos_types::state_resolver::{GenesisResolver, RootObjectResolver};
use moveos_types::transaction::{FunctionCall, VMErrorInfo};
use moveos_types::transaction::{
    MoveOSTransaction, RawTransactionOutput, VerifiedMoveAction, VerifiedMoveOSTransaction,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(thiserror::Error, Debug)]
pub enum VMPanicError {
    #[error("Verifier panic {0:?}.")]
    VerifierPanicError(Error),
    #[error("System call panic {0:?}.")]
    SystemCallPanicError(Error),
}

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
                verifier_config: VerifierConfig::default(),
                deserializer_config: DeserializerConfig::default(),
                paranoid_type_checks: false,
                check_invariant_in_swap_loc: true,
                max_value_nest_depth: Some(DEFAULT_MAX_VALUE_NEST_DEPTH),
                type_max_cost: 0,
                type_base_cost: 0,
                type_byte_cost: 0,
                delayed_field_optimization_enabled: false,
                ty_builder: TypeBuilder::with_limits(128, 20),
                disallow_dispatch_for_native: true,
                use_compatibility_checker_v2: true,
                use_loader_v2: true,
            },
        }
    }
}

pub type MoveOSGlobalModuleCache =
    Arc<RwLock<GlobalModuleCache<ModuleId, CompiledModule, Module, RoochModuleExtension>>>;

pub fn new_moveos_global_module_cache() -> MoveOSGlobalModuleCache {
    Arc::new(RwLock::new(GlobalModuleCache::empty()))
}

#[derive(Clone)]
pub struct MoveOSCacheManager {
    pub runtime_environment: Arc<RwLock<RuntimeEnvironment>>,
    pub global_module_cache:
        Arc<RwLock<GlobalModuleCache<ModuleId, CompiledModule, Module, RoochModuleExtension>>>,
}

impl MoveOSCacheManager {
    pub fn new(
        all_natives: Vec<(AccountAddress, Identifier, Identifier, NativeFunction)>,
        global_module_cache: MoveOSGlobalModuleCache,
    ) -> Self {
        Self {
            runtime_environment: Arc::new(RwLock::new(RuntimeEnvironment::new(all_natives))),
            global_module_cache,
        }
    }
}

pub struct MoveOS {
    vm: MoveOSVM,
    //MoveOS do not need to hold the db
    //It just need a StateResolver to get the state.
    //TODO remove the db from MoveOS
    db: MoveOSStore,
    cost_table: Arc<RwLock<Option<CostTable>>>,
    system_pre_execute_functions: Vec<FunctionCall>,
    system_post_execute_functions: Vec<FunctionCall>,
    cache_manager: MoveOSCacheManager,
}

impl MoveOS {
    pub fn new(
        all_natives: Vec<(AccountAddress, Identifier, Identifier, NativeFunction)>,
        db: MoveOSStore,
        system_pre_execute_functions: Vec<FunctionCall>,
        system_post_execute_functions: Vec<FunctionCall>,
        global_module_cache: MoveOSGlobalModuleCache,
    ) -> Result<Self> {
        //TODO load the gas table from argument, and remove the cost_table lock.
        let moveos_cache_manager = MoveOSCacheManager::new(all_natives, global_module_cache);

        let vm = MoveOSVM::new(moveos_cache_manager.clone())?;
        Ok(Self {
            vm,
            db,
            cost_table: Arc::new(RwLock::new(None)),
            system_pre_execute_functions,
            system_post_execute_functions,
            cache_manager: moveos_cache_manager,
        })
    }

    pub fn init_genesis(
        &self,
        genesis_tx: MoveOSTransaction,
        genesis_objects: Vec<(ObjectState, MoveTypeLayout)>,
    ) -> Result<RawTransactionOutput> {
        self.verify_and_execute_genesis_tx(genesis_tx, genesis_objects)
    }

    fn verify_and_execute_genesis_tx(
        &self,
        tx: MoveOSTransaction,
        genesis_objects: Vec<(ObjectState, MoveTypeLayout)>,
    ) -> Result<RawTransactionOutput> {
        let MoveOSTransaction { root, ctx, action } = tx;
        assert!(root.is_genesis());
        let resolver = GenesisResolver::default();
        let runtime_environment = self.cache_manager.runtime_environment.read();
        let global_module_cache = self.cache_manager.global_module_cache.clone();
        let mut session = self.vm.new_genesis_session(
            &resolver,
            ctx,
            genesis_objects,
            global_module_cache,
            &runtime_environment,
        );

        let verified_action = session.verify_move_action(action).map_err(|e| {
            tracing::error!("verify_genesis_tx error:{:?}", e);
            e
        })?;

        // execute main tx
        let execute_result = session.execute_move_action(verified_action);
        if let Some(vm_error) = execute_result.clone().err() {
            tracing::error!("execute_genesis_tx vm_error:{:?}", vm_error,);
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
        Ok(raw_output)
    }

    fn load_cost_table(&self, root: &ObjectMeta) -> VMResult<CostTable> {
        // We use a scoped lock here to avoid holding the lock for a long time.
        {
            let rlock = self.cost_table.read();
            if let Some(cost_table) = rlock.as_ref() {
                return Ok(cost_table.clone());
            }
        }

        if tracing::enabled!(tracing::Level::TRACE) {
            tracing::trace!("load_cost_table from db");
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
                tracing::warn!("load_cost_table try_write failed");
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
        let mut gas_meter = MoveOSGasMeter::new(cost_table, ctx.max_gas_amount, false);
        gas_meter.set_metering(false);

        let resolver = RootObjectResolver::new(root.clone(), &self.db);
        let runtime_environment = self.cache_manager.runtime_environment.read();
        let global_module_cache = self.cache_manager.global_module_cache.clone();
        let mut session = self.vm.new_readonly_session(
            &resolver,
            ctx.clone(),
            gas_meter,
            global_module_cache,
            &runtime_environment,
        );

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
        let VerifiedMoveOSTransaction { root, ctx, action } = tx.clone();
        let tx_hash = ctx.tx_hash();
        if tracing::enabled!(tracing::Level::DEBUG) {
            tracing::debug!(
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

        let feature_resolver = RootObjectResolver::new(root.clone(), &self.db);
        let feature_store_opt = load_feature_store_object(&feature_resolver);
        let has_io_tired_write_feature = match feature_store_opt {
            None => false,
            Some(feature_store) => feature_store.has_value_size_gas_feature(),
        };

        let cost_table = self.load_cost_table(&root)?;
        let mut gas_meter =
            MoveOSGasMeter::new(cost_table, ctx.max_gas_amount, has_io_tired_write_feature);
        gas_meter.charge_io_write(ctx.tx_size)?;

        let resolver = RootObjectResolver::new(root, &self.db);
        let runtime_environment = self.cache_manager.runtime_environment.read();
        let global_module_cache = self.cache_manager.global_module_cache.clone();
        let mut session = self.vm.new_session(
            &resolver,
            ctx,
            gas_meter,
            global_module_cache,
            &runtime_environment,
        );

        //We do not execute pre_execute and post_execute functions for system call
        if !is_system_call {
            // system pre_execute
            // we do not charge gas for system_pre_execute function
            match session.execute_function_call(self.system_pre_execute_functions.clone(), false) {
                Ok(_) => {}
                Err(error) => {
                    tracing::warn!("System pre execution failed: {:?}", error);
                    return Err(Error::from(VMPanicError::SystemCallPanicError(
                        format_err!("Execute System Pre call Panic {:?}", error),
                    )));
                }
            }
        }

        match self.execute_action(&mut session, action.clone()) {
            Ok(_) => {
                let status = VMStatus::Executed;
                if tracing::enabled!(tracing::Level::DEBUG) {
                    tracing::debug!(
                        "execute_action ok tx(hash:{}) vm_status:{:?}",
                        tx_hash,
                        status
                    );
                }
                self.execution_cleanup(is_system_call, session, status, None)
            }
            Err(vm_err) => {
                if tracing::enabled!(tracing::Level::WARN) {
                    tracing::warn!(
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

        let feature_resolver = RootObjectResolver::new(root.clone(), &self.db);
        let feature_store_opt = load_feature_store_object(&feature_resolver);
        let has_io_tired_write_feature = match feature_store_opt {
            None => false,
            Some(feature_store) => feature_store.has_value_size_gas_feature(),
        };

        let mut gas_meter = MoveOSGasMeter::new(
            cost_table,
            tx_context.max_gas_amount,
            has_io_tired_write_feature,
        );
        gas_meter.set_metering(true);
        let resolver = RootObjectResolver::new(root, &self.db);
        let runtime_environment = self.cache_manager.runtime_environment.read();
        let global_module_cache = self.cache_manager.global_module_cache.clone();
        let mut session = self.vm.new_readonly_session(
            &resolver,
            tx_context.clone(),
            gas_meter,
            global_module_cache,
            &runtime_environment,
        );

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
                if tracing::enabled!(tracing::Level::DEBUG) {
                    tracing::warn!("execute_readonly_function error:{:?}", e);
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
    ) -> Result<(), VMError> {
        session.execute_move_action(action)
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
                    tracing::warn!("System call failed: {:?}", kept_status);
                    return Err(Error::from(VMPanicError::SystemCallPanicError(
                        format_err!("Execute system call with Panic {:?}", vm_error_info),
                    )));
                }

                kept_status
            }
            Err(discard_status) => {
                //This should not happen, if it happens, it means that the VM or verifer has a bug
                tracing::warn!("Discard status: {:?}", discard_status);
                return Err(Error::from(VMPanicError::VerifierPanicError(format_err!(
                    "Execute Action with Panic {:?}",
                    vm_error_info
                ))));
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
            tracing::info!("Gas schedule updated");
            gas_upgrade = true;
            self.cost_table.write().take();
        }

        let (_ctx, mut output) = session.finish_with_extensions(kept_status)?;
        output.is_gas_upgrade = gas_upgrade;
        Ok((output, vm_error_info))
    }

    pub fn flush_module_cache(&self, is_upgrade: bool) -> Result<()> {
        if is_upgrade {
            // the V1 calling of Loader must be disabled
            // self.vm.mark_loader_cache_as_invalid();
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
    let compiled_module = CompiledModule::deserialize(module_bytes.as_ref())?;
    let module_bin_view = BinaryIndexedView::Module(&compiled_module);
    let func_def = module_bin_view.function_def_at(*func_idx)?;
    Ok(module_bin_view
        .identifier_at(module_bin_view.function_handle_at(func_def.function).name)
        .to_string())
}
