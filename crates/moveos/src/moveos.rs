// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::transaction::{AbstractTransaction, MoveTransaction},
    vm::{move_vm_ext::MoveVmExt, MoveResolverExt},
    TransactionExecutor, TransactionValidator,
};
use anyhow::Result;
use move_binary_format::errors::Location;
use move_table_extension::NativeTableContext;
use move_vm_runtime::session::Session;
use move_vm_types::gas::UnmeteredGasMeter;
use statedb::StateDB;

pub struct MoveOS {
    vm: MoveVmExt,
    db: StateDB,
}

impl MoveOS {
    pub fn new(db: StateDB) -> Self {
        let vm = MoveVmExt::new().unwrap();
        Self { vm, db }
    }

    pub fn execute<T>(&self, txn: T) -> Result<()>
    where
        T: AbstractTransaction,
    {
        let session_id = txn.txn_hash();
        let move_txn = txn.into_move_transaction();
        let mut session = self.vm.new_session(&self.db, session_id);
        self.execute_transaction(session, move_txn)
    }

    fn execute_transaction<S>(&self, mut session: Session<S>, txn: MoveTransaction) -> Result<()>
    where
        S: MoveResolverExt,
    {
        let mut gas_meter = UnmeteredGasMeter;
        match txn {
            MoveTransaction::Script(script) => {
                //session.execute_script(script.code, script.ty_args, script.args);
            }
            MoveTransaction::Function(function) => {
                let result = session.execute_entry_function(
                    &function.module,
                    &function.function,
                    function.ty_args,
                    function.args,
                    &mut gas_meter,
                )?;
            }
            MoveTransaction::ModuleBundle(module_bundle) => {
                //session.publish_module(module_bundle);
            }
        }
        let (change_set, events, mut extensions) = session.finish_with_extensions()?;

        let table_context: NativeTableContext = extensions.remove();
        let table_change_set = table_context
            .into_change_set()
            .map_err(|e| e.finish(Location::Undefined))?;

        self.db.apply_change_set(change_set, table_change_set)?;
        Ok(())
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
