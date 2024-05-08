// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use crate::gas::table::{
    get_gas_schedule_entries, initial_cost_schedule, CostTable, MoveOSGasMeter,
};
use crate::vm::moveos_vm::{MoveOSSession, MoveOSVM};
use anyhow::{bail, Result};
use backtrace::Backtrace;
use move_binary_format::errors::VMError;
use move_binary_format::errors::{vm_status_of_result, Location, PartialVMError, VMResult};
use move_core_types::identifier::IdentStr;
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
use moveos_types::moveos_std::event::EventID;
use moveos_types::moveos_std::gas_schedule::GasScheduleUpdated;
use moveos_types::moveos_std::object::RootObjectEntity;
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::moveos_std::tx_result::TxResult;
use moveos_types::startup_info::StartupInfo;
use moveos_types::state::{MoveStructState, MoveStructType};
use moveos_types::state_resolver::RootObjectResolver;
use moveos_types::transaction::{
    MoveOSTransaction, RawTransactionOutput, TransactionOutput, VerifiedMoveAction,
    VerifiedMoveOSTransaction,
};
use moveos_types::{h256::H256, transaction::FunctionCall};
use parking_lot::RwLock;
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
    db: MoveOSStore,
    cost_table: Arc<RwLock<Option<CostTable>>>,
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
    ) -> Result<(H256, u64, TransactionOutput)> {
        self.verify_and_execute_genesis_tx(genesis_tx)
    }

    fn verify_and_execute_genesis_tx(
        &self,
        tx: MoveOSTransaction,
    ) -> Result<(H256, u64, TransactionOutput)> {
        let MoveOSTransaction {
            root,
            ctx,
            action,
            pre_execute_functions: _,
            post_execute_functions: _,
        } = tx;

        let resolver = RootObjectResolver::new(root, &self.db);
        let mut session = self.vm.new_genesis_session(&resolver, ctx);

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
        let (state_root, size, event_ids) = self.apply_transaction_output(raw_output.clone())?;
        let output = TransactionOutput::new(raw_output, event_ids);
        log::info!(
            "execute genesis tx state_root:{:?}, state_size:{}",
            state_root,
            size
        );
        Ok((state_root, size, output))
    }

    fn load_cost_table(&self, root: &RootObjectEntity) -> VMResult<CostTable> {
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
        let MoveOSTransaction {
            root,
            ctx,
            action,
            pre_execute_functions,
            post_execute_functions,
        } = tx;
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
            pre_execute_functions,
            post_execute_functions,
        })
    }

    pub fn execute(&self, tx: VerifiedMoveOSTransaction) -> Result<RawTransactionOutput> {
        let VerifiedMoveOSTransaction {
            root,
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

        let cost_table = self.load_cost_table(&root)?;
        let gas_meter = MoveOSGasMeter::new(cost_table, ctx.max_gas_amount);

        // Temporary behavior, will enable this in the future.
        // gas_meter.charge_io_write(ctx.tx_size)?;

        let resolver = RootObjectResolver::new(root, &self.db);
        let mut session = self.vm.new_session(&resolver, ctx, gas_meter);

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

    pub fn execute_and_apply(
        &self,
        tx: VerifiedMoveOSTransaction,
    ) -> Result<(H256, u64, TransactionOutput)> {
        let raw_output = self.execute(tx)?;
        let (state_root, size, event_ids) = self.apply_transaction_output(raw_output.clone())?;
        let output = TransactionOutput::new(raw_output, event_ids);

        Ok((state_root, size, output))
    }

    fn apply_transaction_output(
        &self,
        output: RawTransactionOutput,
    ) -> Result<(H256, u64, Vec<EventID>)> {
        //TODO move apply change set to a suitable place, and make MoveOS stateless?
        let RawTransactionOutput {
            status: _,
            changeset,
            events,
            gas_used: _,
            is_upgrade: _,
        } = output;

        let (new_state_root, size) = self
            .db
            .get_state_store()
            .apply_change_set(changeset)
            .map_err(|e| {
                PartialVMError::new(StatusCode::STORAGE_ERROR)
                    .with_message(e.to_string())
                    .finish(Location::Undefined)
            })?;
        let event_ids = self.db.get_event_store().save_events(events).map_err(|e| {
            PartialVMError::new(StatusCode::STORAGE_ERROR)
                .with_message(e.to_string())
                .finish(Location::Undefined)
        })?;
        self.db
            .get_config_store()
            .save_startup_info(StartupInfo::new(new_state_root, size))
            .map_err(|e| {
                PartialVMError::new(StatusCode::STORAGE_ERROR)
                    .with_message(e.to_string())
                    .finish(Location::Undefined)
            })?;
        Ok((new_state_root, size, event_ids))
    }

    /// Execute readonly view function
    pub fn execute_view_function(
        &self,
        root: RootObjectEntity,
        function_call: FunctionCall,
    ) -> FunctionResult {
        //TODO allow user to specify the sender
        let tx_context = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        //TODO verify the view function
        self.execute_readonly_function(root, &tx_context, function_call)
    }

    pub fn execute_readonly_function(
        &self,
        root: RootObjectEntity,
        tx_context: &TxContext,
        function_call: FunctionCall,
    ) -> FunctionResult {
        //TODO limit the view function max gas usage
        let cost_table = match self.load_cost_table(&root) {
            Ok(cost_table) => cost_table,
            Err(e) => {
                return FunctionResult::err(e);
            }
        };
        let mut gas_meter = MoveOSGasMeter::new(cost_table, tx_context.max_gas_amount);
        gas_meter.set_metering(false);
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

    // Execute use action with pre_execute and post_execute.
    // Return the user action execution status if success,
    // else return VMError and a bool which indicate if we should respawn the session.
    fn execute_user_action(
        &self,
        session: &mut MoveOSSession<'_, '_, RootObjectResolver<MoveOSStore>, MoveOSGasMeter>,
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
        session: &mut MoveOSSession<'_, '_, RootObjectResolver<MoveOSStore>, MoveOSGasMeter>,
        pre_execute_functions: Vec<FunctionCall>,
        post_execute_functions: Vec<FunctionCall>,
    ) -> VMResult<()> {
        session.execute_function_call(pre_execute_functions, true)?;
        session.execute_function_call(post_execute_functions, true)?;
        Ok(())
    }

    fn execution_cleanup(
        &self,
        mut session: MoveOSSession<'_, '_, RootObjectResolver<MoveOSStore>, MoveOSGasMeter>,
        status: VMStatus,
        _action_opt: Option<VerifiedMoveAction>,
    ) -> Result<RawTransactionOutput> {
        let kept_status = match status.keep_or_discard() {
            Ok(kept_status) => kept_status,
            Err(discard_status) => {
                //This should not happen, if it happens, it means that the VM or verifer has a bug
                let backtrace = Backtrace::new();
                panic!("Discard status: {:?}\n{:?}", discard_status, backtrace);
            }
        };

        //TODO gas_free
        // let mut pay_gas = false;
        // let gas_payment_account_opt = session.tx_context().get::<GasPaymentAccount>()?;

        // if let Some(gas_payment_account) = gas_payment_account_opt {
        //     pay_gas = gas_payment_account.pay_gas_by_module_account;
        // }

        // update txn result to TxContext
        let gas_used = session.query_gas_used();
        //TODO is it a good approach to add tx_result to TxContext?
        let tx_result = TxResult::new(&kept_status, gas_used);
        session
            .object_runtime
            .write()
            .add_to_tx_context(tx_result)
            .expect("Add tx_result to TxContext should always success");

        // system post_execute
        // we do not charge gas for system_post_execute function
        session
            .execute_function_call(self.system_post_execute_functions.clone(), false)
            .expect("system_post_execute should not fail.");

        //TODO gas_free
        // if pay_gas {
        //     self.execute_gas_charge_post(&mut session, &action_opt.unwrap())?;
        // }

        let gas_schedule_updated = session.tx_context().get::<GasScheduleUpdated>()?;
        if let Some(_updated) = gas_schedule_updated {
            log::info!("Gas schedule updated");
            self.cost_table.write().take();
        }

        let (_ctx, output) = session.finish_with_extensions(kept_status)?;
        Ok(output)
    }

    pub fn flush_module_cache(&self, is_upgrade: bool) -> Result<()> {
        if is_upgrade {
            self.vm.mark_loader_cache_as_invalid();
        };
        Ok(())
    }
}
