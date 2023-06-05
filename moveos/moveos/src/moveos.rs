// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::vm::{
    move_vm_ext::{MoveVmExt, SessionExt},
    tx_argument_resolver::{as_struct_no_panic, is_storage_context},
    MoveResolverExt,
};
use anyhow::{anyhow, bail, ensure, Result};
use move_binary_format::{
    errors::{Location, PartialVMError, VMResult},
    CompiledModule,
};
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    value::MoveValue,
    vm_status::{KeptVMStatus, StatusCode},
};
use move_vm_runtime::session::SerializedReturnValues;
use move_vm_types::gas::UnmeteredGasMeter;
use moveos_stdlib::natives::moveos_stdlib::raw_table::NativeTableContext;
use moveos_store::MoveOSDB;
use moveos_store::{event_store::EventStore, state_store::StateDB};
use moveos_types::event::{Event, EventID};
use moveos_types::object::ObjectID;
use moveos_types::transaction::{AuthenticatableTransaction, MoveAction, MoveOSTransaction};
use moveos_types::tx_context::TxContext;
use moveos_types::{addresses::ROOCH_FRAMEWORK_ADDRESS, move_types::FunctionId};
use moveos_types::{h256::H256, transaction::FunctionCall};
use moveos_verifier::verifier::{verify_init_function, INIT_FN_NAME_IDENTIFIER};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionOutput {
    /// The new state root after the transaction execution.
    pub state_root: H256,
    pub status: KeptVMStatus,
}

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
    // db: StateDB,
    db: MoveOSDB,
}

impl MoveOS {
    pub fn new(db: MoveOSDB) -> Result<Self> {
        let vm = MoveVmExt::new()?;
        let is_genesis = db.get_state_store().is_genesis();
        let mut moveos = Self { vm, db };
        if is_genesis {
            let genesis_txn = Self::build_genesis_txn()?;
            moveos.execute(genesis_txn)?;
        }
        Ok(moveos)
    }

    pub fn state(&self) -> &StateDB {
        self.db.get_state_store()
    }

    pub fn event_store(&self) -> &EventStore {
        self.db.get_event_store()
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

    pub fn validate<T: AuthenticatableTransaction>(&mut self, tx: T) -> Result<MoveOSTransaction> {
        let session = self.vm.new_session(&self.db);
        self.validate_transaction(session, tx)
    }

    fn validate_transaction<S, T: AuthenticatableTransaction>(
        &self,
        mut session: SessionExt<S>,
        tx: T,
    ) -> Result<MoveOSTransaction>
    where
        S: MoveResolverExt,
    {
        let authenticator = tx.authenticator_info();

        //TODO ensure the validate function's sender should be the genesis address?
        let tx_context = TxContext::new(*ROOCH_FRAMEWORK_ADDRESS, tx.tx_hash());
        let mut gas_meter = UnmeteredGasMeter;
        let function_id = VALIDATE_FUNCTION.clone();
        let function = FunctionCall::new(
            function_id,
            vec![],
            vec![MoveValue::vector_u8(
                bcs::to_bytes(&authenticator).expect("serialize authenticator should success"),
            )
            .simple_serialize()
            .unwrap()],
        );
        let result = Self::execute_function_bypass_visibility(
            &mut session,
            &tx_context,
            &mut gas_meter,
            function,
        );
        match result {
            Ok(return_values) => {
                let (validate_result, _layout) = return_values
                    .return_values
                    .get(0)
                    .expect("the validate function should return the validate result.");
                let auth_result = bcs::from_bytes::<T::AuthenticatorResult>(validate_result)?;
                tx.construct_moveos_transaction(auth_result)
            }
            Err(e) => {
                //TODO handle the abort error code
                println!("validate failed: {:?}", e);
                // If the error code is EUnsupportedScheme, then we can try to call the sender's validate function
                // This is the Account Abstraction.
                bail!("validate failed: {:?}", e)
            }
        }
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
                let func = session.load_script(script.code.as_slice(), script.ty_args.clone())?;
                moveos_verifier::verifier::verify_entry_function(func, session.runtime_session())?;

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
                let func =
                    session.load_function(&function.function_id, function.ty_args.as_slice())?;
                moveos_verifier::verifier::verify_entry_function(func, session.runtime_session())?;

                let loaded_function =
                    session.load_function(&function.function_id, function.ty_args.as_slice())?;
                let args = session
                    .resolve_args(&tx_context, loaded_function, function.args)
                    .map_err(|e| e.finish(Location::Undefined))?;

                session
                    .execute_entry_function(
                        &function.function_id,
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
                // since the Move runtime does not to know about new packages created,
                // the first deployment contract and the upgrade contract need to be handled separately
                let compiled_modules = self.deserialize_modules(&modules)?;

                //TODO check the modules package address with the sender
                let result = session.publish_module_bundle(modules, sender, &mut gas_meter);
                self.check_and_execute_init_modules(
                    &mut session,
                    &tx_context,
                    &mut gas_meter,
                    &compiled_modules,
                )?;
                result
            }
        };

        let (change_set, raw_events, mut extensions) = session.finish_with_extensions()?;

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
                    .get_state_store()
                    .apply_change_set(change_set, table_change_set)
                    .map_err(|e| {
                        PartialVMError::new(StatusCode::STORAGE_ERROR)
                            .with_message(e.to_string())
                            .finish(Location::Undefined)
                    })?;

                // handle events
                let events = raw_events
                    .into_iter()
                    .enumerate()
                    .map(|(i, e)| {
                        let event_handle_id = ObjectID::from_bytes(e.0.as_slice()).unwrap();
                        Event::new(EventID::new(event_handle_id, e.1), e.2, e.3, i as u32)
                    })
                    .collect();

                self.db.get_event_store().save_events(events).map_err(|e| {
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

    fn deserialize_modules(&self, module_bytes: &[Vec<u8>]) -> Result<Vec<CompiledModule>> {
        let modules = module_bytes
            .iter()
            .map(|b| CompiledModule::deserialize(b).map_err(|e| e.finish(Location::Undefined)))
            .collect::<VMResult<Vec<CompiledModule>>>()
            .map_err(|e| anyhow!("Failed to deserialize modules: {:?}", e))?;

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
        function_id: &FunctionId,
        ty_args: Vec<TypeTag>,
        _args: Vec<Vec<u8>>,
    ) -> Result<bool>
    where
        S: MoveResolverExt,
    {
        let loaded_function = session.load_function(function_id, ty_args.as_slice())?;
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
        let mut modules_to_init = vec![];
        for module in modules {
            let result = verify_init_function(module, session.runtime_session());
            match result {
                Ok(_) => modules_to_init.push(module.self_id()),
                Err(_) => {}
            }
        }

        for module_id in modules_to_init {
            let function_id = FunctionId::new(module_id.clone(), INIT_FN_NAME_IDENTIFIER.clone());

            // check module init permission
            // if !self.check_module_init_permission(
            //     session,
            //     tx_context,
            //     &function_id,
            //     vec![],
            //     vec![],
            // )? {
            //     continue;
            // };

            let function = FunctionCall::new(function_id, vec![], vec![]);
            let _result =
                Self::execute_function_bypass_visibility(session, tx_context, gas_meter, function)
                    .map_err(|e| {
                        anyhow!(
                            "Failed to execute init function at {:?} err: {:?}",
                            module_id,
                            e
                        )
                    })?;
        }

        Ok(())
    }

    fn execute_function_bypass_visibility(
        session: &mut SessionExt<impl MoveResolverExt>,
        tx_context: &TxContext,
        gas_meter: &mut UnmeteredGasMeter,
        function_call: FunctionCall,
    ) -> VMResult<SerializedReturnValues> {
        let loaded_function =
            session.load_function(&function_call.function_id, function_call.ty_args.as_slice())?;
        let args = session
            .resolve_args(tx_context, loaded_function, function_call.args)
            .map_err(|e| e.finish(Location::Undefined))?;
        session.execute_function_bypass_visibility(
            &function_call.function_id,
            function_call.ty_args,
            args,
            gas_meter,
        )
    }

    /// Execute readonly view function
    pub fn execute_view_function(
        &self,
        function_call: FunctionCall,
    ) -> Result<SerializedReturnValues> {
        let mut session = self.vm.new_session(&self.db);
        //TODO limit the view function max gas usage
        let mut gas_meter = UnmeteredGasMeter;
        //View function use a fix address and fix hash
        let tx_context = TxContext::new(AccountAddress::ZERO, H256::zero());

        let result = Self::execute_function_bypass_visibility(
            &mut session,
            &tx_context,
            &mut gas_meter,
            function_call,
        )?;
        let (change_set, events, mut extensions) = session.finish_with_extensions()?;

        let table_context: NativeTableContext = extensions.remove();
        let table_change_set = table_context
            .into_change_set()
            .map_err(|e| e.finish(Location::Undefined))?;

        ensure!(
            change_set.accounts().is_empty(),
            "Change set should be empty when execute view function"
        );
        ensure!(
            events.is_empty(),
            "Events should be empty when execute view function"
        );
        ensure!(
            table_change_set.changes.is_empty(),
            "Table change set should be empty when execute view function"
        );
        Ok(result)
    }
}
