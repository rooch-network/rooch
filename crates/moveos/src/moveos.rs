// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::transaction::{AbstractTransaction, MoveTransaction, SimpleTransaction},
    vm::{move_vm_ext::MoveVmExt, MoveResolverExt},
    TransactionExecutor, TransactionValidator,
};
use anyhow::Result;
use framework::addresses::MOS_FRAMEWORK_ADDRESS;
use mos_types::tx_context::TxContext;
use move_binary_format::errors::{Location, PartialVMError, VMResult};
use move_core_types::{
    account_address::AccountAddress,
    identifier::IdentStr,
    language_storage::{ModuleId, TypeTag},
    move_resource::MoveStructType,
    value::MoveValue,
    vm_status::StatusCode,
};
use move_table_extension::NativeTableContext;
use move_vm_runtime::session::{LoadedFunctionInstantiation, SerializedReturnValues, Session};
use move_vm_types::{gas::UnmeteredGasMeter, loaded_data::runtime_types::Type};
use statedb::{HashValue, StateDB};
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
            MoveTransaction::ModuleBundle(framework::Framework::build()?.into_module_bundles()?);
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
        mut session: Session<S>,
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
                //TODO find a nicer way to fill the signer arguments
                let args = check_and_rearrange_args_by_signer_position(
                    &session,
                    loaded_function,
                    script.args,
                    tx_context,
                )?;
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
                //TODO find a nicer way to fill the signer arguments
                let args = check_and_rearrange_args_by_signer_position(
                    &session,
                    loaded_function,
                    function.args,
                    tx_context,
                )?;
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

//TODO refactor this, provide a TransactionParameterResolver trait, and let extension to provide.
fn check_and_rearrange_args_by_signer_position<S>(
    session: &Session<S>,
    func: LoadedFunctionInstantiation,
    mut args: Vec<Vec<u8>>,
    tx_context: TxContext,
) -> VMResult<Vec<Vec<u8>>>
where
    S: MoveResolverExt,
{
    let has_signer = func
        .parameters
        .iter()
        .position(|i| matches!(i, Type::Signer))
        .map(|pos| {
            if pos != 0 {
                Err(
                    PartialVMError::new(StatusCode::NUMBER_OF_SIGNER_ARGUMENTS_MISMATCH)
                        .with_message(format!(
                            "Expected signer arg is this first arg, but got it at {}",
                            pos + 1
                        ))
                        .finish(Location::Undefined),
                )
            } else {
                Ok(true)
            }
        })
        .unwrap_or(Ok(false))?;

    if has_signer {
        let signer = MoveValue::Signer(tx_context.sender());
        args.push(
            signer
                .simple_serialize()
                .expect("serialize signer should success"),
        );
    }

    let has_tx_context = func
        .parameters
        .iter()
        .position(|i| is_tx_context(session, i))
        .map(|pos| {
            if pos != 0 {
                Err(
                    PartialVMError::new(StatusCode::NUMBER_OF_SIGNER_ARGUMENTS_MISMATCH)
                        .with_message(format!(
                            "Expected TxContext arg is this first arg, but got it at {}",
                            pos + 1
                        ))
                        .finish(Location::Undefined),
                )
            } else {
                Ok(true)
            }
        })
        .unwrap_or(Ok(false))?;

    if has_tx_context {
        args.push(tx_context.to_vec());
    }

    Ok(args)
}

fn is_tx_context<T>(session: &Session<T>, t: &Type) -> bool
where
    T: MoveResolverExt,
{
    match t {
        Type::Struct(s) => match session.get_struct_type(*s) {
            Some(t) => {
                *t.module.address() == *framework::addresses::MOS_STD_ADDRESS
                    && t.module.name() == TxContext::module_identifier().as_ident_str()
                    && t.name == TxContext::struct_identifier()
            }
            None => false,
        },
        Type::Reference(r) => is_tx_context(session, &r),
        Type::MutableReference(r) => is_tx_context(session, &r),
        _ => false,
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
