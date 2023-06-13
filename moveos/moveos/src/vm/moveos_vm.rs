// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::tx_argument_resolver::TxArgumentResolver;
use anyhow::ensure;
use move_binary_format::{
    compatibility::Compatibility,
    errors::{Location, VMError, VMResult},
    file_format::AbilitySet,
    CompiledModule,
};
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    value::MoveTypeLayout,
    vm_status::{KeptVMStatus, VMStatus},
};
use move_vm_runtime::{
    config::VMConfig,
    move_vm::MoveVM,
    native_extensions::NativeContextExtensions,
    native_functions::NativeFunction,
    session::{LoadedFunctionInstantiation, SerializedReturnValues, Session},
};
use move_vm_types::{
    data_store::DataStore,
    gas::{GasMeter, UnmeteredGasMeter},
    loaded_data::runtime_types::{CachedStructIndex, StructType, Type},
};
use moveos_stdlib::natives::moveos_stdlib::raw_table::NativeTableContext;
use moveos_types::{
    event::{Event, EventID},
    function_return_value::FunctionReturnValue,
    move_types::FunctionId,
    object::ObjectID,
    state_resolver::MoveOSResolver,
    storage_context::StorageContext,
    transaction::{FunctionCall, MoveAction, TransactionOutput, VerifiedMoveAction},
};
use moveos_verifier::verifier::INIT_FN_NAME_IDENTIFIER;
use std::{borrow::Borrow, sync::Arc};

/// MoveOSVM is a wrapper of MoveVM with MoveOS specific features.
pub struct MoveOSVM {
    inner: MoveVM,
    finalize_function: Option<FunctionId>,
}

impl MoveOSVM {
    pub fn new(
        natives: impl IntoIterator<Item = (AccountAddress, Identifier, Identifier, NativeFunction)>,
        vm_config: VMConfig,
        finalize_function: Option<FunctionId>,
    ) -> VMResult<Self> {
        Ok(Self {
            inner: MoveVM::new_with_config(natives, vm_config)?,
            finalize_function,
        })
    }

    pub fn new_session<'r, S: MoveOSResolver, G: GasMeter>(
        &self,
        remote: &'r S,
        ctx: StorageContext,
        gas_meter: G,
    ) -> MoveOSSession<'r, '_, S, G> {
        MoveOSSession::new(
            &self.inner,
            remote,
            ctx,
            self.finalize_function.clone(),
            gas_meter,
            false,
        )
    }

    pub fn new_genesis_session<'r, S: MoveOSResolver>(
        &self,
        remote: &'r S,
        ctx: StorageContext,
    ) -> MoveOSSession<'r, '_, S, UnmeteredGasMeter> {
        //Do not charge gas for genesis session
        let gas_meter = UnmeteredGasMeter;
        // Genesis session do not need to execute finalize function
        MoveOSSession::new(&self.inner, remote, ctx, None, gas_meter, false)
    }

    pub fn new_readonly_session<'r, S: MoveOSResolver, G: GasMeter>(
        &self,
        remote: &'r S,
        ctx: StorageContext,
        gas_meter: G,
    ) -> MoveOSSession<'r, '_, S, G> {
        MoveOSSession::new(&self.inner, remote, ctx, None, gas_meter, true)
    }
}

/// MoveOSSession is a wrapper of MoveVM session with MoveOS specific features.
/// It is used to execute a transaction, every transaction should be executed in a new session.
/// Every session has a TxContext, if the transaction have multiple actions, the TxContext is shared.
pub struct MoveOSSession<'r, 'l, S, G> {
    vm: &'l MoveVM,
    remote: &'r S,
    session: Session<'r, 'l, S>,
    ctx: StorageContext,
    finalize_function: Option<FunctionId>,
    gas_meter: G,
    read_only: bool,
}

impl<'r, 'l, S, G> MoveOSSession<'r, 'l, S, G>
where
    S: MoveOSResolver,
    G: GasMeter,
{
    pub fn new(
        vm: &'l MoveVM,
        remote: &'r S,
        ctx: StorageContext,
        finalize_function: Option<FunctionId>,
        gas_meter: G,
        read_only: bool,
    ) -> Self {
        Self {
            vm,
            remote,
            session: Self::new_inner_session(vm, remote),
            ctx,
            finalize_function,
            gas_meter,
            read_only,
        }
    }

    fn new_inner_session(vm: &'l MoveVM, remote: &'r S) -> Session<'r, 'l, S> {
        let mut extensions = NativeContextExtensions::default();

        extensions.add(NativeTableContext::new(remote));

        // The VM code loader has bugs around module upgrade. After a module upgrade, the internal
        // cache needs to be flushed to work around those bugs.
        vm.flush_loader_cache_if_invalidated();
        vm.new_session_with_extensions(remote, extensions)
    }

    /// Verify a move action.
    /// The caller should call this function when validate a transaction.
    /// If the result is error, the transaction should be rejected.
    pub fn verify_move_action(
        &self,
        action: MoveAction,
    ) -> Result<VerifiedMoveAction, anyhow::Error> {
        match action {
            MoveAction::Script(script) => {
                let loaded_function = self
                    .session
                    .load_script(script.code.as_slice(), script.ty_args.clone())?;
                moveos_verifier::verifier::verify_entry_function(&loaded_function, &self.session)?;
                let resolved_args = self
                    .ctx
                    .resolve_argument(&self.session, &loaded_function, script.args.clone())
                    .map_err(|e| e.finish(Location::Undefined))?;
                Ok(VerifiedMoveAction::Script {
                    call: script,
                    resolved_args,
                })
            }
            MoveAction::Function(function) => {
                let loaded_function = self.session.load_function(
                    &function.function_id.module_id,
                    &function.function_id.function_name,
                    function.ty_args.as_slice(),
                )?;
                moveos_verifier::verifier::verify_entry_function(&loaded_function, &self.session)?;
                let resolved_args = self
                    .ctx
                    .resolve_argument(&self.session, &loaded_function, function.args.clone())
                    .map_err(|e| e.finish(Location::Undefined))?;
                Ok(VerifiedMoveAction::Function {
                    call: function,
                    resolved_args,
                })
            }
            MoveAction::ModuleBundle(module_bundle) => {
                let compiled_modules = deserialize_modules(&module_bundle)?;

                let mut init_function_modules = vec![];
                for module in &compiled_modules {
                    let result = moveos_verifier::verifier::verify_module(module);
                    match result {
                        Ok(res) => {
                            if res {
                                init_function_modules.push(module.self_id())
                            }
                        }
                        Err(err) => return Err(err.into()),
                    }
                }

                //TODO add more module verifier.
                Ok(VerifiedMoveAction::ModuleBundle {
                    module_bundle,
                    init_function_modules,
                })
            }
        }
    }

    /// Execute a move action.
    /// The caller should ensure call verify_move_action before execute.
    /// Once we start executing transactions, we must ensure that the transaction execution has a result, regardless of success or failure,
    /// and we need to save the result and deduct gas
    pub fn execute_move_action(&mut self, action: VerifiedMoveAction) -> VMResult<()> {
        match action {
            VerifiedMoveAction::Script {
                call,
                resolved_args,
            } => self
                .session
                .execute_script(call.code, call.ty_args, resolved_args, &mut self.gas_meter)
                .map(|ret| {
                    debug_assert!(
                        ret.return_values.is_empty(),
                        "Script function should not return values"
                    );
                    self.update_storage_context_via_return_values(&ret);
                }),
            VerifiedMoveAction::Function {
                call,
                resolved_args,
            } => self
                .session
                .execute_entry_function(
                    &call.function_id.module_id,
                    &call.function_id.function_name,
                    call.ty_args.clone(),
                    resolved_args,
                    &mut self.gas_meter,
                )
                .map(|ret| {
                    debug_assert!(
                        ret.return_values.is_empty(),
                        "Entry function should not return values"
                    );
                    self.update_storage_context_via_return_values(&ret);
                }),
            VerifiedMoveAction::ModuleBundle {
                module_bundle,
                init_function_modules,
            } => {
                //TODO check the modules package address with the sender
                let sender = self.ctx.tx_context.sender();
                //TODO check the compatiblity
                let compat_config = Compatibility::no_check();
                self.session.publish_module_bundle_with_compat_config(
                    module_bundle,
                    sender,
                    &mut self.gas_meter,
                    compat_config,
                )?;
                self.execute_init_modules(init_function_modules)
            }
        }
    }

    // Because the StorageContext can be mut argument, if the function change the StorageContext,
    // we need to update the StorageContext via return values, and pass the updated StorageContext to the next function.
    fn update_storage_context_via_return_values(
        &mut self,
        _return_values: &SerializedReturnValues,
    ) {
        //TODO get StorageContext from _return_values.mutable_reference_outputs
    }

    pub fn execute_function_bypass_visibility(
        &mut self,
        call: FunctionCall,
    ) -> VMResult<Vec<FunctionReturnValue>> {
        let loaded_function = self.session.load_function(
            &call.function_id.module_id,
            &call.function_id.function_name,
            call.ty_args.as_slice(),
        )?;
        let resolved_args = self
            .ctx
            .resolve_argument(&self.session, &loaded_function, call.args)
            .map_err(|e| e.finish(Location::Undefined))?;

        let return_values = self.session.execute_function_bypass_visibility(
            &call.function_id.module_id,
            &call.function_id.function_name,
            call.ty_args,
            resolved_args,
            &mut self.gas_meter,
        )?;
        self.update_storage_context_via_return_values(&return_values);
        return_values
            .return_values
            .into_iter()
            .zip(loaded_function.return_.iter())
            .map(|((v, _layout), ty)| {
                // We can not use
                // let type_tag :TypeTag = TryInto::try_into(&layout)?
                // to get TypeTag from MoveTypeLayout, because this MoveTypeLayout not MoveLayoutType::WithTypes
                // Invalid MoveTypeLayout -> StructTag conversion--needed MoveLayoutType::WithTypes
                let type_tag = self.session.get_type_tag(ty)?;
                Ok(FunctionReturnValue::new(type_tag, v))
            })
            .collect()
    }

    fn execute_init_modules(
        &mut self,
        init_function_modules: Vec<ModuleId>,
    ) -> Result<(), VMError> {
        for module_id in init_function_modules {
            let function_id = FunctionId::new(module_id.clone(), INIT_FN_NAME_IDENTIFIER.clone());
            let call = FunctionCall::new(function_id, vec![], vec![]);

            self.execute_function_bypass_visibility(call)
                .map(|result| {
                    debug_assert!(result.is_empty(), "Init function must not return value")
                })?;
        }

        Ok(())
    }

    pub fn finish_with_extensions(
        self,
        vm_status: VMStatus,
    ) -> Result<(StorageContext, TransactionOutput), anyhow::Error> {
        let (finalized_session, status) = match vm_status.keep_or_discard() {
            Ok(status) => self.finalizer(status),
            Err(discard_status) => {
                //This should not happen, if it happens, it means that the VM or verifer has a bug
                //TODO try to handle this error
                panic!("Discard status: {:?}", discard_status);
            }
        };

        let (changeset, raw_events, mut extensions) =
            finalized_session.session.finish_with_extensions()?;
        let table_context: NativeTableContext = extensions.remove();
        let state_changeset = table_context
            .into_change_set()
            .map_err(|e| e.finish(Location::Undefined))?;

        if finalized_session.read_only {
            ensure!(
                changeset.accounts().is_empty(),
                "ChangeSet should be empty when execute readonly function"
            );
            ensure!(
                raw_events.is_empty(),
                "Events should be empty when execute readonly function"
            );
            ensure!(
                state_changeset.changes.is_empty(),
                "Table change set should be empty when execute readonly function"
            );
        }

        let events = raw_events
            .into_iter()
            .enumerate()
            .map(|(i, e)| {
                let event_handle_id = ObjectID::from_bytes(e.0.as_slice())
                    .expect("the event handle id must be ObjectID");
                Event::new(EventID::new(event_handle_id, e.1), e.2, e.3, i as u64)
            })
            .collect();
        //TODO calculate the gas_used with gas_meter
        let gas_used = 0;
        Ok((
            finalized_session.ctx,
            TransactionOutput {
                status,
                changeset,
                state_changeset,
                events,
                gas_used,
            },
        ))
    }

    fn finalizer(self, status: KeptVMStatus) -> (Self, KeptVMStatus) {
        if self.read_only {
            (self, status)
        } else {
            let mut finalized_session = match &status {
                KeptVMStatus::Executed => self,
                _error => {
                    //if the execution failed, we need to start a new session, and discard the transaction changes
                    // and increment the sequence number or reduce the gas in new session.
                    let MoveOSSession {
                        vm,
                        remote,
                        session: _,
                        ctx,
                        finalize_function,
                        gas_meter,
                        read_only,
                    } = self;
                    Self::new(vm, remote, ctx, finalize_function, gas_meter, read_only)
                }
            };
            if let Some(finalize_function) = &finalized_session.finalize_function {
                finalized_session
                    .execute_function_bypass_visibility(FunctionCall::new(
                        finalize_function.clone(),
                        vec![],
                        vec![],
                    ))
                    .expect("finalize function should always success");
            }
            (finalized_session, status)
        }
    }

    /// Load a script and all of its types into cache
    pub fn load_script(
        &self,
        script: impl Borrow<[u8]>,
        ty_args: Vec<TypeTag>,
    ) -> VMResult<LoadedFunctionInstantiation> {
        self.session.load_script(script, ty_args)
    }

    /// Load a module, a function, and all of its types into cache
    pub fn load_function(
        &self,
        function_id: &FunctionId,
        type_arguments: &[TypeTag],
    ) -> VMResult<LoadedFunctionInstantiation> {
        self.session.load_function(
            &function_id.module_id,
            &function_id.function_name,
            type_arguments,
        )
    }

    pub fn load_type(&self, type_tag: &TypeTag) -> VMResult<Type> {
        self.session.load_type(type_tag)
    }

    pub fn get_type_layout(&self, type_tag: &TypeTag) -> VMResult<MoveTypeLayout> {
        self.session.get_type_layout(type_tag)
    }

    pub fn get_fully_annotated_type_layout(&self, type_tag: &TypeTag) -> VMResult<MoveTypeLayout> {
        self.session.get_fully_annotated_type_layout(type_tag)
    }

    pub fn get_type_tag(&self, ty: &Type) -> VMResult<TypeTag> {
        self.session.get_type_tag(ty)
    }

    pub fn get_struct_type(&self, index: CachedStructIndex) -> Option<Arc<StructType>> {
        self.session.get_struct_type(index)
    }

    pub fn get_type_abilities(&self, ty: &Type) -> VMResult<AbilitySet> {
        self.session.get_type_abilities(ty)
    }

    pub fn get_data_store(&mut self) -> &mut dyn DataStore {
        self.session.get_data_store()
    }

    pub fn get_native_extensions(&self) -> &NativeContextExtensions<'r> {
        self.session.get_native_extensions()
    }

    pub fn runtime_session(&self) -> &Session<'r, 'l, S> {
        self.session.borrow()
    }
}

impl AsRef<MoveVM> for MoveOSVM {
    fn as_ref(&self) -> &MoveVM {
        &self.inner
    }
}

fn deserialize_modules(module_bytes: &[Vec<u8>]) -> Result<Vec<CompiledModule>, VMError> {
    module_bytes
        .iter()
        .map(|b| CompiledModule::deserialize(b).map_err(|e| e.finish(Location::Undefined)))
        .collect::<VMResult<Vec<CompiledModule>>>()
}
