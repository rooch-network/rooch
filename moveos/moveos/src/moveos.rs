// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::vm::move_vm_ext::MoveVmExt;
use anyhow::{bail, Result};
use move_binary_format::errors::vm_status_of_result;
use move_binary_format::errors::{Location, PartialVMError};
use move_core_types::vm_status::VMStatus;
use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, language_storage::ModuleId,
    value::MoveValue, vm_status::StatusCode,
};
use move_vm_types::gas::UnmeteredGasMeter;
use moveos_store::MoveOSDB;
use moveos_store::{event_store::EventStore, state_store::StateDB};
use moveos_types::function_return_value::FunctionReturnValue;
use moveos_types::state_resolver::MoveOSResolverProxy;
use moveos_types::storage_context::StorageContext;
use moveos_types::transaction::{
    AuthenticatableTransaction, MoveAction, MoveOSTransaction, TransactionOutput,
    VerifiedMoveOSTransaction,
};
use moveos_types::tx_context::TxContext;
use moveos_types::{addresses::ROOCH_FRAMEWORK_ADDRESS, move_types::FunctionId};
use moveos_types::{h256::H256, transaction::FunctionCall};
use once_cell::sync::Lazy;

pub static VALIDATE_FUNCTION: Lazy<FunctionId> = Lazy::new(|| {
    FunctionId::new(
        ModuleId::new(
            *ROOCH_FRAMEWORK_ADDRESS,
            Identifier::new("account").unwrap(),
        ),
        Identifier::new("validate").unwrap(),
    )
});

pub struct MoveOS {
    vm: MoveVmExt,
    db: MoveOSResolverProxy<MoveOSDB>,
}

impl MoveOS {
    pub fn new(db: MoveOSDB) -> Result<Self> {
        let vm = MoveVmExt::new()?;
        let is_genesis = db.get_state_store().is_genesis();
        let mut moveos = Self {
            vm,
            db: MoveOSResolverProxy(db),
        };
        if is_genesis {
            //TODO the genesis
            let genesis_tx = Self::build_genesis_tx()?;
            let verified_tx = moveos.verify(genesis_tx)?;
            moveos.execute(verified_tx)?;
        }
        Ok(moveos)
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

    //TODO move to a suitable place
    pub fn build_genesis_tx() -> Result<MoveOSTransaction> {
        let genesis_tx =
            MoveAction::ModuleBundle(moveos_stdlib::Framework::build()?.into_module_bundles()?);
        Ok(MoveOSTransaction::new_for_test(
            *ROOCH_FRAMEWORK_ADDRESS,
            genesis_tx,
        ))
    }

    pub fn validate<T: AuthenticatableTransaction>(
        &mut self,
        tx: T,
    ) -> Result<VerifiedMoveOSTransaction> {
        //TODO ensure the validate function's sender should be the genesis address?
        let tx_context = TxContext::new(*ROOCH_FRAMEWORK_ADDRESS, tx.tx_hash());

        let authenticator = tx.authenticator_info();

        let function_id = VALIDATE_FUNCTION.clone();
        let call = FunctionCall::new(
            function_id,
            vec![],
            vec![MoveValue::vector_u8(
                bcs::to_bytes(&authenticator).expect("serialize authenticator should success"),
            )
            .simple_serialize()
            .unwrap()],
        );
        let result = self.execute_readonly_function(tx_context, call);
        match result {
            Ok(return_values) => {
                let validate_result = &return_values
                    .get(0)
                    .expect("the validate function should return the validate result.")
                    .value;
                let auth_result = bcs::from_bytes::<T::AuthenticatorResult>(validate_result)?;
                //TODO verify should share session with validate
                Ok(self.verify(tx.construct_moveos_transaction(auth_result)?)?)
            }
            Err(e) => {
                //TODO handle the abort error code
                //let status = explain_vm_status(self.db.get_state_store(), e.into_vm_status())?;
                println!("validate failed: {:?}", e);
                // If the error code is EUnsupportedScheme, then we can try to call the sender's validate function
                // This is the Account Abstraction.
                bail!("validate failed: {:?}", e)
            }
        }
    }

    pub fn verify(&mut self, tx: MoveOSTransaction) -> Result<VerifiedMoveOSTransaction> {
        let MoveOSTransaction {
            ctx: tx_context,
            action,
        } = tx;

        let gas_meter = UnmeteredGasMeter;
        let ctx = StorageContext::new(tx_context.clone());
        let session = self.vm.new_session(&self.db, ctx, gas_meter);

        let verified_action = session.verify_move_action(action)?;
        Ok(VerifiedMoveOSTransaction {
            ctx: tx_context,
            action: verified_action,
        })
    }

    pub fn execute(&mut self, tx: VerifiedMoveOSTransaction) -> Result<(H256, TransactionOutput)> {
        let VerifiedMoveOSTransaction { ctx, action } = tx;
        //TODO define the gas meter.
        let gas_meter = UnmeteredGasMeter;
        let ctx = StorageContext::new(ctx);
        let mut session = self.vm.new_session(&self.db, ctx, gas_meter);

        let execute_result = session.execute_move_action(action);
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
            state_changeset: table_changeset,
            events,
            gas_used: _,
        } = output;

        let new_state_root = self
            .db
            .0
            .get_state_store()
            .apply_change_set(changeset, table_changeset)
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
        //View function use a fix address and fix hash
        let tx_context = TxContext::new(AccountAddress::ZERO, H256::zero());
        //TODO verify the view function
        self.execute_readonly_function(tx_context, function_call)
    }

    fn execute_readonly_function(
        &self,
        tx_context: TxContext,
        function_call: FunctionCall,
    ) -> Result<Vec<FunctionReturnValue>> {
        let ctx = StorageContext::new(tx_context);
        //TODO limit the view function max gas usage
        let gas_meter = UnmeteredGasMeter;
        let mut session = self.vm.new_readonly_session(&self.db, ctx, gas_meter);

        let result = session.execute_function_bypass_visibility(function_call)?;

        // if execute success, finish the session to check if it change the state
        let (_ctx, _output) = session.finish_with_extensions(VMStatus::Executed)?;

        Ok(result)
    }
}
