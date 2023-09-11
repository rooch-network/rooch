// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::gas::table::{initial_cost_schedule, MoveOSGasMeter};
use crate::vm::moveos_vm::MoveOSVM;
use anyhow::{bail, ensure, Result};
use move_binary_format::errors::{vm_status_of_result, Location, PartialVMError, VMResult};
use move_core_types::vm_status::KeptVMStatus;
use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, vm_status::StatusCode,
};
use move_vm_runtime::config::VMConfig;
use move_vm_runtime::native_functions::NativeFunction;
use moveos_store::config_store::ConfigDBStore;
use moveos_store::event_store::EventDBStore;
use moveos_store::state_store::statedb::StateDBStore;
use moveos_store::transaction_store::TransactionDBStore;
use moveos_store::MoveOSStore;
use moveos_types::function_return_value::FunctionResult;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::moveos_std::tx_result::TxResult;
use moveos_types::startup_info::StartupInfo;
use moveos_types::state::MoveState;
use moveos_types::state_resolver::MoveOSResolverProxy;
use moveos_types::transaction::{MoveOSTransaction, TransactionOutput, VerifiedMoveOSTransaction};
use moveos_types::tx_context::TxContext;
use moveos_types::{h256::H256, transaction::FunctionCall};

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

    pub fn init_genesis<T: Into<MoveOSTransaction>, GT: MoveState + Clone>(
        &mut self,
        genesis_txs: Vec<T>,
        genesis_ctx: GT,
    ) -> Result<Vec<(H256, TransactionOutput)>> {
        ensure!(
            self.db.0.get_state_store().is_genesis(),
            "genesis already initialized"
        );
        genesis_txs
            .into_iter()
            .map(|tx| self.verify_and_execute_genesis_tx(tx.into(), genesis_ctx.clone()))
            .collect::<Result<Vec<_>>>()
    }

    fn verify_and_execute_genesis_tx<GT: MoveState>(
        &mut self,
        tx: MoveOSTransaction,
        genesis_ctx: GT,
    ) -> Result<(H256, TransactionOutput)> {
        let MoveOSTransaction {
            mut ctx,
            action,
            pre_execute_functions: _,
            post_execute_functions: _,
        } = tx;
        ctx.add(genesis_ctx)?;
        let mut session = self.vm.new_genesis_session(&self.db, ctx);
        let verified_action = session.verify_move_action(action)?;

        // execute main tx
        let execute_result = session.execute_move_action(verified_action);
        let status = match vm_status_of_result(execute_result).keep_or_discard() {
            Ok(status) => status,
            Err(discard_status) => {
                bail!("Discard status: {:?}", discard_status);
            }
        };

        let (_ctx, output) = session.finish_with_extensions(status)?;
        if output.status != KeptVMStatus::Executed {
            bail!("genesis tx should success, error: {:?}", output.status);
        }
        let state_root = self.apply_transaction_output(output.clone())?;
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

    pub fn execute(&self, tx: VerifiedMoveOSTransaction) -> Result<TransactionOutput> {
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
        let system_env = ctx.map.clone();
        let cost_table = initial_cost_schedule();
        let gas_meter = MoveOSGasMeter::new(cost_table, ctx.max_gas_amount);
        let mut session = self.vm.new_session(&self.db, ctx, gas_meter);

        // system pre_execute
        // we do not charge gas for system_pre_execute function
        session
            .execute_function_call(self.system_pre_execute_functions.clone(), false)
            .expect("system_pre_execute should not fail.");

        // user pre_execute
        session.execute_function_call(pre_execute_functions.clone(), true)?;

        // execute main tx
        let execute_result = session.execute_move_action(action);
        let vm_status = vm_status_of_result(execute_result);

        // user post_execute
        let (mut post_session, status) = match vm_status.keep_or_discard() {
            Ok(status) => {
                let mut post_session = match &status {
                    KeptVMStatus::Executed => session,
                    _error => {
                        //if the execution failed, we need to start a new session, and discard the transaction changes
                        // and increment the sequence number or reduce the gas in new session.
                        let mut s = session.respawn(system_env);
                        //Because the session is respawned, the pre_execute function should be called again.
                        s.execute_function_call(pre_execute_functions, true)?;
                        s
                    }
                };
                post_session.execute_function_call(post_execute_functions, true)?;
                (post_session, status)
            }
            Err(discard_status) => {
                //This should not happen, if it happens, it means that the VM or verifer has a bug
                // bail!("Discard status: {:?}", discard_status);
                panic!("Discard status: {:?}", discard_status);
            }
        };

        // update txn result to TxContext
        let gas_used = post_session.query_gas_used();
        //TODO is it a good approach to add tx_result to TxContext?
        let tx_result = TxResult::new(&status, gas_used);
        post_session
            .storage_context_mut()
            .tx_context
            .add(tx_result)
            .expect("Add tx_result to TxContext should always success");

        // system post_execute
        // we do not charge gas for system_post_execute function
        post_session
            .execute_function_call(self.system_post_execute_functions.clone(), false)
            .expect("system_post_execute should not fail.");

        let (_ctx, output) = post_session.finish_with_extensions(status)?;
        Ok(output)
    }

    pub fn execute_and_apply(
        &mut self,
        tx: VerifiedMoveOSTransaction,
    ) -> Result<(H256, TransactionOutput)> {
        let output = self.execute(tx)?;
        let state_root = self.apply_transaction_output(output.clone())?;
        Ok((state_root, output))
    }

    fn apply_transaction_output(&mut self, output: TransactionOutput) -> Result<H256> {
        //TODO move apply change set to a suitable place, and make MoveOS stateless?
        let TransactionOutput {
            status: _,
            changeset,
            state_changeset,
            events,
            gas_used: _,
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
        self.db
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
        Ok(new_state_root)
    }

    /// Execute readonly view function
    pub fn execute_view_function(&self, function_call: FunctionCall) -> FunctionResult {
        //TODO allow user to specify the sender
        let tx_context = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        //TODO verify the view function
        self.execute_readonly_function(&tx_context, function_call)
    }

    fn execute_readonly_function(
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
