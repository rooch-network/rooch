// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::transaction::{AbstractTransaction, MoveTransaction, SimpleTransaction},
    vm::{
        move_vm_ext::{MoveVmExt, SessionExt},
        MoveResolverExt,
    },
    TransactionExecutor, TransactionValidator,
};
use anyhow::Result;
use moveos_stdlib::addresses::MOS_FRAMEWORK_ADDRESS;
use moveos_stdlib::natives::mos_stdlib::object_extension::NativeObjectContext;
use moveos_types::tx_context::TxContext;
use move_binary_format::errors::Location;
use move_core_types::{
    account_address::AccountAddress,
    identifier::IdentStr,
    language_storage::{ModuleId, TypeTag},
};
use move_table_extension::NativeTableContext;
use move_vm_runtime::session::SerializedReturnValues;
use move_vm_types::gas::UnmeteredGasMeter;
use moveos_statedb::{HashValue, StateDB};
use std::borrow::Borrow;

pub struct MoveOS {
    vm: MoveVmExt,
    db: StateDB,
}

impl MoveOS {
    pub fn new(db: StateDB) -> Result<Self> {
        let vm = MoveVmExt::new()?;
        let is_genesis = db.is_genesis();
        let moveos = Self { vm, db };
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
    pub fn build_genesis_txn() -> Result<SimpleTransaction> {
        let genesis_txn =
            MoveTransaction::ModuleBundle(moveos_stdlib::Framework::build()?.into_module_bundles()?);
        Ok(SimpleTransaction::new(*MOS_FRAMEWORK_ADDRESS, genesis_txn))
    }

    pub fn execute<T>(&self, txn: T) -> Result<()>
    where
        T: AbstractTransaction,
    {
        let tx_hash = txn.txn_hash();
        let senders = txn.senders();
        let move_txn = txn.into_move_transaction();
        let session = self.vm.new_session(&self.db, tx_hash);
        self.execute_transaction(session, senders, tx_hash, move_txn)
    }

    fn execute_transaction<S>(
        &self,
        mut session: SessionExt<S>,
        mut senders: Vec<AccountAddress>,
        tx_hash: HashValue,
        txn: MoveTransaction,
    ) -> Result<()>
    where
        S: MoveResolverExt,
    {
        let mut gas_meter = UnmeteredGasMeter;
        //TODO only allow one sender?
        let sender = senders.pop().unwrap();
        let tx_context = TxContext::new(sender, tx_hash);
        match txn {
            MoveTransaction::Script(script) => {
                let loaded_function =
                    session.load_script(script.code.as_slice(), script.ty_args.clone())?;

                let args = session.resolve_args(&tx_context, loaded_function, script.args)?;
                let result =
                    session.execute_script(script.code, script.ty_args, args, &mut gas_meter)?;
                assert!(
                    result.return_values.is_empty(),
                    "Script function should not return values"
                );
            }
            MoveTransaction::Function(function) => {
                let loaded_function = session.load_function(
                    &function.module,
                    &function.function,
                    function.ty_args.as_slice(),
                )?;
                let args = session.resolve_args(&tx_context, loaded_function, function.args)?;
                let result = session.execute_entry_function(
                    &function.module,
                    &function.function,
                    function.ty_args,
                    args,
                    &mut gas_meter,
                )?;
                assert!(
                    result.return_values.is_empty(),
                    "Entry function should not return values"
                );
            }
            MoveTransaction::ModuleBundle(modules) => {
                //TODO check the modules package address with the sender
                session.publish_module_bundle(modules, sender, &mut gas_meter)?;
            }
        }
        let (change_set, _events, mut extensions) = session.finish_with_extensions()?;

        //TODO handle events

        let table_context: NativeTableContext = extensions.remove();
        let table_change_set = table_context
            .into_change_set()
            .map_err(|e| e.finish(Location::Undefined))?;

        //TODO move apply change set to a suitable place, and make MoveOS state less.
        self.db.apply_change_set(change_set, table_change_set)?;

        let object_context: NativeObjectContext = extensions.remove();
        let object_change_set = object_context
            .into_change_set()
            .map_err(|e| e.finish(Location::Undefined))?;
        self.db.apply_object_change_set(object_change_set)?;

        Ok(())
    }

    /// Execute readonly view function
    pub fn execute_view_function(
        &self,
        module: &ModuleId,
        function_name: &IdentStr,
        ty_args: Vec<TypeTag>,
        args: Vec<impl Borrow<[u8]>>,
    ) -> Result<SerializedReturnValues> {
        let session_id = HashValue::random();
        let mut session = self.vm.new_session(&self.db, session_id);
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
    fn validate_transaction<T: AbstractTransaction>(
        &self,
        _transaction: T,
    ) -> crate::ValidatorResult {
        todo!()
    }
}

impl TransactionExecutor for MoveOS {
    fn execute_transaction<T: AbstractTransaction>(
        &self,
        _transaction: T,
    ) -> crate::ExecutorResult {
        todo!()
    }
}
