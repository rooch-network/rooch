// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::vm::moveos_vm::MoveOSVM;
use anyhow::{bail, ensure, Result};
use move_binary_format::errors::vm_status_of_result;
use move_binary_format::errors::{Location, PartialVMError};
use move_core_types::vm_status::{KeptVMStatus, VMStatus};
use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, vm_status::StatusCode,
};
use move_vm_runtime::config::VMConfig;
use move_vm_runtime::native_functions::NativeFunction;
use move_vm_types::gas::UnmeteredGasMeter;
use moveos_store::transaction_store::TransactionDB;
use moveos_store::MoveOSDB;
use moveos_store::{event_store::EventStore, state_store::StateDB};
use moveos_types::function_return_value::FunctionReturnValue;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::move_types::FunctionId;
use moveos_types::state_resolver::MoveOSResolverProxy;
use moveos_types::storage_context::StorageContext;
use moveos_types::transaction::{MoveOSTransaction, TransactionOutput, VerifiedMoveOSTransaction};
use moveos_types::tx_context::TxContext;
use moveos_types::{h256::H256, transaction::FunctionCall};

pub struct MoveOSConfig {
    pub vm_config: VMConfig,
    /// if the finalize_function is set, the MoveOS will call the function after the transaction is executed.
    /// otherwise, the MoveOS will not call the function.
    pub finalize_function: Option<FunctionId>,
}

//TODO make VMConfig cloneable
impl Clone for MoveOSConfig {
    fn clone(&self) -> Self {
        Self {
            vm_config: VMConfig {
                verifier: self.vm_config.verifier.clone(),
                max_binary_format_version: self.vm_config.max_binary_format_version,
                paranoid_type_checks: self.vm_config.paranoid_type_checks,
            },
            finalize_function: self.finalize_function.clone(),
        }
    }
}

pub struct MoveOS {
    vm: MoveOSVM,
    db: MoveOSResolverProxy<MoveOSDB>,
}

impl MoveOS {
    pub fn new(
        db: MoveOSDB,
        natives: impl IntoIterator<Item = (AccountAddress, Identifier, Identifier, NativeFunction)>,
        config: MoveOSConfig,
    ) -> Result<Self> {
        let vm = MoveOSVM::new(natives, config.vm_config, config.finalize_function)?;
        Ok(Self {
            vm,
            db: MoveOSResolverProxy(db),
        })
    }

    //TODO genesis tx should be in one transaction?
    pub fn init_genesis(&mut self, genesis_txs: Vec<MoveOSTransaction>) -> Result<()> {
        ensure!(
            self.db.0.get_state_store().is_genesis(),
            "genesis already initialized"
        );

        for genesis_tx in genesis_txs {
            self.verify_and_execute_genesis_tx(genesis_tx)?;
        }
        //TODO return the state root genesis TransactionExecutionInfo
        Ok(())
    }

    fn verify_and_execute_genesis_tx(&mut self, tx: MoveOSTransaction) -> Result<()> {
        let MoveOSTransaction {
            ctx: tx_context,
            action,
        } = tx;

        let ctx = StorageContext::new(tx_context);
        let mut session = self.vm.new_genesis_session(&self.db, ctx);
        let verified_action = session.verify_move_action(action)?;
        let execute_result = session.execute_move_action(verified_action);
        let vm_status = vm_status_of_result(execute_result);
        let (_ctx, output) = session.finish_with_extensions(vm_status)?;
        if output.status != KeptVMStatus::Executed {
            bail!("genesis tx should success, error: {:?}", output.status);
        }
        let _state_root = self.apply_transaction_output(output)?;
        Ok(())
    }

    pub fn state(&self) -> &StateDB {
        self.db.0.get_state_store()
    }

    pub fn moveos_resolver(&self) -> &MoveOSResolverProxy<MoveOSDB> {
        &self.db
    }

    pub fn event_store(&self) -> &EventStore {
        self.db.0.get_event_store()
    }

    pub fn transaction_store(&self) -> &TransactionDB {
        self.db.0.get_transaction_store()
    }

    pub fn verify(&self, tx: MoveOSTransaction) -> Result<VerifiedMoveOSTransaction> {
        let MoveOSTransaction {
            ctx: tx_context,
            action,
        } = tx;

        let gas_meter = UnmeteredGasMeter;
        let ctx = StorageContext::new(tx_context.clone());
        let session = self.vm.new_readonly_session(&self.db, ctx, gas_meter);

        let verified_action = session.verify_move_action(action)?;
        let (_, _) = session.finish_with_extensions(VMStatus::Executed)?;
        Ok(VerifiedMoveOSTransaction {
            ctx: tx_context,
            action: verified_action,
        })
    }

    pub fn execute(&mut self, tx: VerifiedMoveOSTransaction) -> Result<(H256, TransactionOutput)> {
        let VerifiedMoveOSTransaction { ctx, action } = tx;
        let tx_hash = ctx.tx_hash();
        if log::log_enabled!(log::Level::Debug) {
            log::debug!(
                "execute tx(sender:{}, hash:{}, action:{})",
                ctx.sender(),
                tx_hash,
                action
            );
        }
        //TODO define the gas meter.
        let gas_meter = UnmeteredGasMeter;
        let ctx = StorageContext::new(ctx);
        let mut session = self.vm.new_session(&self.db, ctx, gas_meter);
        let execute_result = session.execute_move_action(action);
        if execute_result.is_err() {
            log::warn!("execute tx({}) error: {:?}", tx_hash, execute_result);
        }
        let vm_status = vm_status_of_result(execute_result);
        let (_ctx, output) = session.finish_with_extensions(vm_status)?;
        let state_root = self.apply_transaction_output(output.clone())?;
        Ok((state_root, output))
    }

    fn apply_transaction_output(&self, output: TransactionOutput) -> Result<H256> {
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
        Ok(new_state_root)
    }

    /// Execute readonly view function
    pub fn execute_view_function(
        &self,
        function_call: FunctionCall,
    ) -> Result<Vec<FunctionReturnValue>> {
        //TODO allow user to specify the sender and hash
        //View function use a fix address and fix hash
        let tx_context = TxContext::new(AccountAddress::ZERO, H256::zero());
        //TODO verify the view function
        self.execute_readonly_function(&tx_context, function_call)
    }

    fn execute_readonly_function(
        &self,
        tx_context: &TxContext,
        function_call: FunctionCall,
    ) -> Result<Vec<FunctionReturnValue>> {
        let ctx = StorageContext::new(tx_context.clone());
        //TODO limit the view function max gas usage
        let gas_meter = UnmeteredGasMeter;
        let mut session = self.vm.new_readonly_session(&self.db, ctx, gas_meter);

        let result = session.execute_function_bypass_visibility(function_call)?;

        // if execute success, finish the session to check if it change the state
        let (_ctx, _output) = session.finish_with_extensions(VMStatus::Executed)?;

        Ok(result)
    }
}

impl MoveFunctionCaller for MoveOS {
    fn call_function(
        &self,
        ctx: &TxContext,
        function_call: FunctionCall,
    ) -> Result<Vec<FunctionReturnValue>> {
        self.execute_readonly_function(ctx, function_call)
    }
}
