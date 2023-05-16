// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    vm::{
        move_vm_ext::{MoveVmExt, SessionExt},
        MoveResolverExt,
    },
    TransactionExecutor, TransactionValidator,
};
use anyhow::{bail, Result};
use move_binary_format::errors::{Location, PartialVMError, VMResult};
use move_core_types::{
    account_address::AccountAddress,
    identifier::{IdentStr, Identifier},
    language_storage::{ModuleId, TypeTag},
    value::MoveValue,
    vm_status::{KeptVMStatus, StatusCode},
};
use move_vm_runtime::session::SerializedReturnValues;
use move_vm_types::gas::UnmeteredGasMeter;
use moveos_statedb::StateDB;
use moveos_stdlib::addresses::ROOCH_FRAMEWORK_ADDRESS;
use moveos_stdlib::natives::moveos_stdlib::raw_table::NativeTableContext;
use moveos_types::transaction::{MoveAction, MoveOSTransaction};
use moveos_types::tx_context::TxContext;
use moveos_types::{h256::H256, transaction::Function};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionOutput {
    /// The new state root after the transaction execution.
    pub state_root: H256,
    pub status: KeptVMStatus,
}

pub static VALIDATE_FUNCTION: Lazy<(ModuleId, Identifier)> = Lazy::new(|| {
    (
        ModuleId::new(
            *ROOCH_FRAMEWORK_ADDRESS,
            Identifier::new("account").unwrap(),
        ),
        Identifier::new("validate").unwrap(),
    )
});

pub struct MoveOS {
    vm: MoveVmExt,
    db: StateDB,
}

impl MoveOS {
    pub fn new(db: StateDB) -> Result<Self> {
        let vm = MoveVmExt::new()?;
        let is_genesis = db.is_genesis();
        let mut moveos = Self { vm, db };
        if is_genesis {
            let genesis_txn = Self::build_genesis_txn()?;
            moveos.execute(genesis_txn)?;
        }
        Ok(moveos)
    }

    pub fn state(&self) -> &StateDB {
        &self.db
    }

    //TODO move to a suitable place
    pub fn build_genesis_txn() -> Result<MoveOSTransaction> {
        let genesis_txn =
            MoveAction::ModuleBundle(moveos_stdlib::Framework::build()?.into_module_bundles()?);
        Ok(MoveOSTransaction::new_for_test(
            *ROOCH_FRAMEWORK_ADDRESS,
            genesis_txn,
        ))
    }

    pub fn validate<T: Into<MoveOSTransaction>, A: Into<Vec<u8>>>(
        &mut self,
        tx: T,
        authenticator: A,
    ) -> Result<MoveOSTransaction> {
        let moveos_tx: MoveOSTransaction = tx.into();
        let tx_context = TxContext::new(moveos_tx.sender, moveos_tx.tx_hash);
        let session = self.vm.new_session(&self.db);
        let (module, function_name) = VALIDATE_FUNCTION.clone();
        let function = Function::new(
            module,
            function_name,
            vec![],
            vec![MoveValue::vector_u8(authenticator.into())
                .simple_serialize()
                .unwrap()],
        );
        let result = Self::execute_function_bypass_visibility(session, &tx_context, function);
        match result {
            Ok(_) => {}
            Err(e) => {
                //TODO handle the abort error code
                println!("validate failed: {:?}", e);
                // If the error code is EUnsupportedScheme, then we can try to call the sender's validate function
                // This is the Account Abstraction.
            }
        }
        Ok(moveos_tx)
    }

    pub fn execute(&mut self, tx: MoveOSTransaction) -> Result<TransactionOutput> {
        let MoveOSTransaction {
            sender,
            action,
            tx_hash,
        } = tx;
        let session = self.vm.new_session(&self.db);
        self.execute_transaction(session, sender, tx_hash, action)
    }

    // TODO should be return the execute result
    fn execute_transaction<S>(
        &self,
        mut session: SessionExt<S>,
        sender: AccountAddress,
        tx_hash: H256,
        action: MoveAction,
    ) -> Result<TransactionOutput>
    where
        S: MoveResolverExt,
    {
        let mut gas_meter = UnmeteredGasMeter;
        let tx_context = TxContext::new(sender, tx_hash);
        let execute_result = match action {
            MoveAction::Script(script) => {
                let loaded_function =
                    session.load_script(script.code.as_slice(), script.ty_args.clone())?;

                let args = session
                    .resolve_args(&tx_context, loaded_function, script.args)
                    .map_err(|e| e.finish(Location::Undefined))?;
                session
                    .execute_script(script.code, script.ty_args, args, &mut gas_meter)
                    .map(|ret| {
                        debug_assert!(
                            ret.return_values.is_empty(),
                            "Script function should not return values"
                        );
                    })
            }
            MoveAction::Function(function) => {
                let loaded_function = session.load_function(
                    &function.module,
                    &function.function,
                    function.ty_args.as_slice(),
                )?;
                let args = session
                    .resolve_args(&tx_context, loaded_function, function.args)
                    .map_err(|e| e.finish(Location::Undefined))?;
                session
                    .execute_entry_function(
                        &function.module,
                        &function.function,
                        function.ty_args,
                        args,
                        &mut gas_meter,
                    )
                    .map(|ret| {
                        debug_assert!(
                            ret.return_values.is_empty(),
                            "Entry function should not return values"
                        );
                    })
            }
            MoveAction::ModuleBundle(modules) => {
                //TODO check the modules package address with the sender
                session.publish_module_bundle(modules, sender, &mut gas_meter)
            }
        };

        let (change_set, _events, mut extensions) = session.finish_with_extensions()?;

        //TODO handle events

        let table_context: NativeTableContext = extensions.remove();
        let table_change_set = table_context
            .into_change_set()
            .map_err(|e| e.finish(Location::Undefined))?;

        let vm_status = move_binary_format::errors::vm_status_of_result(execute_result);
        match vm_status.keep_or_discard() {
            Ok(status) => {
                //TODO move apply change set to a suitable place, and make MoveOS stateless.
                let new_state_root = self
                    .db
                    .apply_change_set(change_set, table_change_set)
                    .map_err(|e| {
                        PartialVMError::new(StatusCode::STORAGE_ERROR)
                            .with_message(e.to_string())
                            .finish(Location::Undefined)
                    })?;
                Ok(TransactionOutput {
                    state_root: new_state_root,
                    status,
                })
            }
            Err(discard) => {
                bail!("VM discard the transaction: {:?}", discard)
            }
        }
    }

    fn execute_function_bypass_visibility(
        mut session: SessionExt<impl MoveResolverExt>,
        tx_context: &TxContext,
        function: Function,
    ) -> VMResult<SerializedReturnValues> {
        let mut gas_meter = UnmeteredGasMeter;
        let loaded_function = session.load_function(
            &function.module,
            &function.function,
            function.ty_args.as_slice(),
        )?;
        let args = session
            .resolve_args(tx_context, loaded_function, function.args)
            .map_err(|e| e.finish(Location::Undefined))?;
        session.execute_function_bypass_visibility(
            &function.module,
            &function.function,
            function.ty_args,
            args,
            &mut gas_meter,
        )
    }

    /// Execute readonly view function
    pub fn execute_view_function(
        &self,
        module: &ModuleId,
        function_name: &IdentStr,
        ty_args: Vec<TypeTag>,
        args: Vec<impl Borrow<[u8]>>,
    ) -> Result<SerializedReturnValues> {
        let mut session = self.vm.new_session(&self.db);
        //TODO limit the view function max gas usage
        let mut gas_meter = UnmeteredGasMeter;
        let result = session.execute_function_bypass_visibility(
            module,
            function_name,
            ty_args,
            args,
            &mut gas_meter,
        )?;
        let (change_set, events, mut extensions) = session.finish_with_extensions()?;

        let table_context: NativeTableContext = extensions.remove();
        let table_change_set = table_context
            .into_change_set()
            .map_err(|e| e.finish(Location::Undefined))?;

        assert!(
            change_set.accounts().is_empty(),
            "Change set should be empty when execute view function"
        );
        assert!(
            events.is_empty(),
            "Events should be empty when execute view function"
        );
        assert!(
            table_change_set.changes.is_empty(),
            "Table change set should be empty when execute view function"
        );
        Ok(result)
    }
}

impl TransactionValidator for MoveOS {
    fn validate_transaction<T>(&self, _transaction: T) -> crate::ValidatorResult {
        todo!()
    }
}

impl TransactionExecutor for MoveOS {
    fn execute_transaction<T>(&self, _transaction: T) -> crate::ExecutorResult {
        todo!()
    }
}
