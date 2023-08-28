// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::gas::table::MoveOSGasMeter;
use crate::vm::moveos_vm::MoveOSVM;
use anyhow::{bail, ensure, Result};
use move_binary_format::errors::{vm_status_of_result, Location, PartialVMError, VMResult};
use move_core_types::vm_status::{KeptVMStatus, VMStatus};
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
}

impl MoveOS {
    pub fn new(
        db: MoveOSStore,
        natives: impl IntoIterator<Item = (AccountAddress, Identifier, Identifier, NativeFunction)>,
        config: MoveOSConfig,
    ) -> Result<Self> {
        let vm = MoveOSVM::new(natives, config.vm_config)?;
        Ok(Self {
            vm,
            db: MoveOSResolverProxy(db),
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
        let execute_result = session.execute_move_action(verified_action);
        let vm_status = vm_status_of_result(execute_result);
        let (_ctx, output) = session.finish_with_extensions(vm_status)?;
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

        let gas_meter = MoveOSGasMeter::new_unmetered();
        let session = self
            .vm
            .new_readonly_session(&self.db, ctx.clone(), gas_meter);

        let verified_action = session.verify_move_action(action)?;
        let (_, _) = session.finish_with_extensions(VMStatus::Executed)?;
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
        let gas_meter = MoveOSGasMeter::new_unmetered();
        let mut session = self.vm.new_session(
            &self.db,
            ctx,
            pre_execute_functions,
            post_execute_functions,
            gas_meter,
        );
        let execute_result = session.execute_move_action(action);
        if execute_result.is_err() {
            log::warn!("execute tx({}) error: {:?}", tx_hash, execute_result);
        }
        let vm_status = vm_status_of_result(execute_result);
        let (_ctx, output) = session.finish_with_extensions(vm_status)?;
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
        let gas_meter = MoveOSGasMeter::new_unmetered();
        let mut session = self
            .vm
            .new_readonly_session(&self.db, tx_context.clone(), gas_meter);

        let result = session.execute_function_bypass_visibility(function_call);
        match result {
            Ok(return_values) => {
                // if execute success, finish the session to check if it change the state
                match session.finish_with_extensions(VMStatus::Executed) {
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
