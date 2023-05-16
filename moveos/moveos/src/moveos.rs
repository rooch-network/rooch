// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    vm::{
        move_vm_ext::{MoveVmExt, SessionExt},
        tx_argument_resolver::{as_struct_no_panic, is_storage_context},
        MoveResolverExt,
    },
    TransactionExecutor, TransactionValidator, INIT_FN_NAME,
};
use anyhow::{anyhow, Result};
use move_binary_format::access::ModuleAccess;
use move_binary_format::{
    errors::{Location, VMResult},
    file_format::Visibility,
    CompiledModule,
};
use move_core_types::{
    account_address::AccountAddress,
    identifier::IdentStr,
    language_storage::{ModuleId, TypeTag},
};
use move_vm_runtime::session::SerializedReturnValues;
use move_vm_types::gas::UnmeteredGasMeter;
use moveos_statedb::StateDB;
use moveos_stdlib::addresses::ROOCH_FRAMEWORK_ADDRESS;
use moveos_stdlib::natives::moveos_stdlib::raw_table::NativeTableContext;
use moveos_types::h256::H256;
use moveos_types::transaction::{AbstractTransaction, MoveTransaction, SimpleTransaction};
use moveos_types::tx_context::TxContext;
use std::borrow::Borrow;
// use moveos_types::error::MoveOSError::{VMModuleDeserializationError};

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
        let genesis_txn = MoveTransaction::ModuleBundle(
            moveos_stdlib::Framework::build()?.into_module_bundles()?,
        );
        Ok(SimpleTransaction::new(
            *ROOCH_FRAMEWORK_ADDRESS,
            genesis_txn,
        ))
    }

    pub fn execute<T>(&self, txn: T) -> Result<()>
    where
        T: AbstractTransaction,
    {
        let tx_hash = txn.txn_hash();
        let senders = txn.senders();
        let move_txn = txn.into_move_transaction();
        let session = self.vm.new_session(&self.db);
        self.execute_transaction(session, senders, tx_hash, move_txn)
    }

    // TODO should be return the execute result
    fn execute_transaction<S>(
        &self,
        mut session: SessionExt<S>,
        mut senders: Vec<AccountAddress>,
        tx_hash: H256,
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
                // since the Move runtime does not to know about new packages created,
                // the first deployment contract and the upgrade contract need to be handled separately
                let compiled_modules = self.deserialize_modules(&modules)?;

                //TODO check the modules package address with the sender
                session.publish_module_bundle(modules, sender, &mut gas_meter)?;
                self.check_and_execute_init_modules(
                    &mut session,
                    &tx_context,
                    &mut gas_meter,
                    &compiled_modules,
                )?;
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

        // let object_context: NativeObjectContext = extensions.remove();
        // let object_change_set = object_context
        //     .into_change_set()
        //     .map_err(|e| e.finish(Location::Undefined))?;
        // self.db.apply_object_change_set(object_change_set)?;

        Ok(())
    }

    fn deserialize_modules(&self, module_bytes: &[Vec<u8>]) -> Result<Vec<CompiledModule>> {
        let modules = module_bytes
            .iter()
            .map(|b| CompiledModule::deserialize(b).map_err(|e| e.finish(Location::Undefined)))
            .collect::<VMResult<Vec<CompiledModule>>>()
            .map_err(|e| anyhow!("Failed to deserialize modules: {:?}", e))?;

        assert!(
            !modules.is_empty(),
            "input checker ensures package is not empty"
        );

        Ok(modules)
    }

    /// The initializer function must have the following properties in order to be executed at publication:
    /// - Name init
    /// - Single parameter of &mut TxContext type
    /// - No return values
    /// - Private
    fn check_module_init_permission<S>(
        &self,
        session: &mut SessionExt<S>,
        _tx_context: &TxContext,
        module_id: &ModuleId,
        function_name: &IdentStr,
        ty_args: Vec<TypeTag>,
        _args: Vec<Vec<u8>>,
    ) -> Result<bool>
    where
        S: MoveResolverExt,
    {
        let loaded_function =
            session.load_function(module_id, function_name, ty_args.as_slice())?;
        let Some((_i, _t)) = loaded_function.parameters.iter().enumerate().find(|(i, t)| {
            let struct_type = as_struct_no_panic(session, t);
            (*i as u32 == 0u32) && Option::is_some(&struct_type) && is_storage_context(&(struct_type.unwrap()))
        }) else {
            return Ok(false)
        };

        if !(loaded_function.return_.is_empty()) {
            return Ok(false);
        }
        Ok(true)
    }

    fn check_and_execute_init_modules<S>(
        &self,
        session: &mut SessionExt<S>,
        tx_context: &TxContext,
        gas_meter: &mut UnmeteredGasMeter,
        modules: &[CompiledModule],
    ) -> Result<()>
    where
        S: MoveResolverExt,
    {
        let modules_to_init = modules.iter().filter_map(|module| {
            for fdef in &module.function_defs {
                let fhandle = module.function_handle_at(fdef.function);
                let fname = module.identifier_at(fhandle.name);
                if fname == INIT_FN_NAME {
                    // check function visibility
                    if Visibility::Private == fdef.visibility && !fdef.is_entry {
                        return Some(module.self_id());
                    }
                }
            }
            None
        });

        for module_id in modules_to_init {
            // check module init permission
            if !self
                .check_module_init_permission(
                    session,
                    tx_context,
                    &module_id,
                    INIT_FN_NAME,
                    vec![],
                    vec![],
                )
                .unwrap()
            {
                continue;
            }
            let _return_values = self.execute_move_call_bypass_visibility(
                session,
                tx_context,
                gas_meter,
                &module_id,
                INIT_FN_NAME,
                vec![],
                vec![],
            )?;

            // assert!(
            // return_values.is_empty(),
            // "init should not have return values"
            // )
        }

        Ok(())
    }

    fn execute_move_call_bypass_visibility<S>(
        &self,
        session: &mut SessionExt<S>,
        tx_context: &TxContext,
        gas_meter: &mut UnmeteredGasMeter,
        module: &ModuleId,
        function_name: &IdentStr,
        ty_args: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
    ) -> Result<()>
    where
        S: MoveResolverExt,
    {
        let loaded_function = session.load_function(module, function_name, ty_args.as_slice())?;
        let args = session.resolve_args(&tx_context, loaded_function, args)?;

        let result = session.execute_function_bypass_visibility(
            module,
            function_name,
            ty_args,
            args,
            gas_meter,
        )?;
        assert!(
            result.return_values.is_empty(),
            "Entry function should not return values"
        );

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
