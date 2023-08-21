// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{
    data_cache::{into_change_set, MoveosDataCache},
    tx_argument_resolver::TxArgumentResolver,
};
use move_binary_format::{
    compatibility::Compatibility,
    errors::{Location, PartialVMError, VMError, VMResult},
    file_format::AbilitySet,
    CompiledModule,
};

use crate::gas::table::MoveOSGasMeter;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    value::MoveTypeLayout,
    vm_status::{KeptVMStatus, StatusCode, VMStatus},
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
    gas::GasMeter,
    loaded_data::runtime_types::{CachedStructIndex, StructType, Type},
};
use moveos_stdlib::natives::moveos_stdlib::{
    move_module::NativeModuleContext,
    raw_table::{NativeTableContext, TableData},
};
use moveos_types::{
    event::{Event, EventID},
    function_return_value::FunctionReturnValue,
    move_types::FunctionId,
    moveos_std::module_upgrade_flag::ModuleUpgradeFlag,
    object::ObjectID,
    state_resolver::MoveOSResolver,
    storage_context::StorageContext,
    transaction::{FunctionCall, MoveAction, TransactionOutput, VerifiedMoveAction},
    tx_context::TxContext,
};
use moveos_verifier::verifier::INIT_FN_NAME_IDENTIFIER;
use parking_lot::RwLock;
use std::{borrow::Borrow, sync::Arc};

/// MoveOSVM is a wrapper of MoveVM with MoveOS specific features.
pub struct MoveOSVM {
    inner: MoveVM,
}

impl MoveOSVM {
    pub fn new(
        natives: impl IntoIterator<Item = (AccountAddress, Identifier, Identifier, NativeFunction)>,
        vm_config: VMConfig,
    ) -> VMResult<Self> {
        Ok(Self {
            inner: MoveVM::new_with_config(natives, vm_config)?,
        })
    }

    pub fn new_session<'r, S: MoveOSResolver, G: GasMeter>(
        &self,
        remote: &'r S,
        ctx: TxContext,
        pre_execute_functions: Vec<FunctionCall>,
        post_execute_functions: Vec<FunctionCall>,
        gas_meter: G,
    ) -> MoveOSSession<'r, '_, S, G> {
        MoveOSSession::new(
            &self.inner,
            remote,
            ctx,
            pre_execute_functions,
            post_execute_functions,
            gas_meter,
            false,
        )
    }

    pub fn new_genesis_session<'r, S: MoveOSResolver>(
        &self,
        remote: &'r S,
        ctx: TxContext,
    ) -> MoveOSSession<'r, '_, S, MoveOSGasMeter> {
        //Do not charge gas for genesis session
        let gas_meter = MoveOSGasMeter::new();
        // Genesis session do not need to execute pre_execute and post_execute function
        MoveOSSession::new(&self.inner, remote, ctx, vec![], vec![], gas_meter, false)
    }

    pub fn new_readonly_session<'r, S: MoveOSResolver, G: GasMeter>(
        &self,
        remote: &'r S,
        ctx: TxContext,
        gas_meter: G,
    ) -> MoveOSSession<'r, '_, S, G> {
        MoveOSSession::new(&self.inner, remote, ctx, vec![], vec![], gas_meter, true)
    }
}

/// MoveOSSession is a wrapper of MoveVM session with MoveOS specific features.
/// It is used to execute a transaction, every transaction should be executed in a new session.
/// Every session has a TxContext, if the transaction have multiple actions, the TxContext is shared.
pub struct MoveOSSession<'r, 'l, S, G> {
    vm: &'l MoveVM,
    remote: &'r S,
    session: Session<'r, 'l, MoveosDataCache<'r, 'l, S>>,
    ctx: StorageContext,
    table_data: Arc<RwLock<TableData>>,
    pre_execute_functions: Vec<FunctionCall>,
    post_execute_functions: Vec<FunctionCall>,
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
        ctx: TxContext,
        pre_execute_functions: Vec<FunctionCall>,
        post_execute_functions: Vec<FunctionCall>,
        gas_meter: G,
        read_only: bool,
    ) -> Self {
        if read_only {
            assert!(
                pre_execute_functions.is_empty(),
                "pre_execute_function is not allowed in read only session"
            );
            assert!(
                post_execute_functions.is_empty(),
                "post_execute_function is not allowed in read only session"
            );
        }
        let ctx = StorageContext::new(ctx);
        let table_data = Arc::new(RwLock::new(TableData::default()));
        let s = Self {
            vm,
            remote,
            session: Self::new_inner_session(vm, remote, table_data.clone()),
            ctx,
            table_data,
            pre_execute_functions,
            post_execute_functions,
            gas_meter,
            read_only,
        };
        s.pre_execute()
    }

    /// Re spawn a new session with the same context.
    pub fn respawn(self) -> Self {
        //FIXME
        //The TxContext::spawn function will reset the ids_created and kv map.
        //But we need some TxContext value in the pre_execute and post_execute function, such as the TxValidateResult.
        //We need to find a solution.
        let ctx = StorageContext::new(self.ctx.tx_context.spawn());
        let s = Self {
            session: Self::new_inner_session(self.vm, self.remote, self.table_data.clone()),
            ctx,
            ..self
        };
        //Because the session is respawned, the pre_execute function should be called again.
        s.pre_execute()
    }

    fn new_inner_session(
        vm: &'l MoveVM,
        remote: &'r S,
        table_data: Arc<RwLock<TableData>>,
    ) -> Session<'r, 'l, MoveosDataCache<'r, 'l, S>> {
        let mut extensions = NativeContextExtensions::default();

        extensions.add(NativeTableContext::new(remote, table_data.clone()));
        extensions.add(NativeModuleContext::new(remote));

        // The VM code loader has bugs around module upgrade. After a module upgrade, the internal
        // cache needs to be flushed to work around those bugs.
        // vm.mark_loader_cache_as_invalid();
        vm.flush_loader_cache_if_invalidated();
        let loader = vm.runtime().loader();
        let data_store: MoveosDataCache<'r, 'l, S> =
            MoveosDataCache::new(remote, loader, table_data);
        vm.new_session_with_cache_and_extensions(data_store, extensions)
    }

    /// Verify a move action.
    /// The caller should call this function when validate a transaction.
    /// If the result is error, the transaction should be rejected.
    pub fn verify_move_action(&self, action: MoveAction) -> VMResult<VerifiedMoveAction> {
        match action {
            MoveAction::Script(call) => {
                let loaded_function = self
                    .session
                    .load_script(call.code.as_slice(), call.ty_args.clone())?;
                moveos_verifier::verifier::verify_entry_function(&loaded_function, &self.session)
                    .map_err(|e| e.finish(Location::Undefined))?;
                let _resolved_args = self
                    .ctx
                    .resolve_argument(&self.session, &loaded_function, call.args.clone())
                    .map_err(|e| e.finish(Location::Undefined))?;
                Ok(VerifiedMoveAction::Script { call })
            }
            MoveAction::Function(call) => {
                let loaded_function = self.session.load_function(
                    &call.function_id.module_id,
                    &call.function_id.function_name,
                    call.ty_args.as_slice(),
                )?;
                moveos_verifier::verifier::verify_entry_function(&loaded_function, &self.session)
                    .map_err(|e| e.finish(Location::Undefined))?;
                let _resolved_args = self
                    .ctx
                    .resolve_argument(&self.session, &loaded_function, call.args.clone())
                    .map_err(|e| e.finish(Location::Undefined))?;
                Ok(VerifiedMoveAction::Function { call })
            }
            MoveAction::ModuleBundle(module_bundle) => {
                let compiled_modules = deserialize_modules(&module_bundle)?;

                let mut init_function_modules = vec![];
                for module in &compiled_modules {
                    let result = moveos_verifier::verifier::verify_module(module, self.remote);
                    match result {
                        Ok(res) => {
                            if res {
                                init_function_modules.push(module.self_id())
                            }
                        }
                        Err(err) => return Err(err),
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
        let action_result = match action {
            VerifiedMoveAction::Script { call } => {
                let loaded_function = self
                    .session
                    .load_script(call.code.as_slice(), call.ty_args.clone())?;

                let resolved_args = self
                    .ctx
                    .resolve_argument(&self.session, &loaded_function, call.args)
                    .map_err(|e| e.finish(Location::Undefined))?;

                self.session
                    .execute_script(call.code, call.ty_args, resolved_args, &mut self.gas_meter)
                    .map(|ret| {
                        debug_assert!(
                            ret.return_values.is_empty(),
                            "Script function should not return values"
                        );
                        self.update_storage_context_via_return_values(&ret);
                    })
            }
            VerifiedMoveAction::Function { call } => {
                let loaded_function = self.session.load_function(
                    &call.function_id.module_id,
                    &call.function_id.function_name,
                    call.ty_args.as_slice(),
                )?;

                let resolved_args = self
                    .ctx
                    .resolve_argument(&self.session, &loaded_function, call.args)
                    .map_err(|e| e.finish(Location::Undefined))?;

                self.session
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
                    })
            }
            VerifiedMoveAction::ModuleBundle {
                module_bundle,
                init_function_modules,
            } => {
                //TODO check the modules package address with the sender
                let sender = self.ctx.tx_context.sender();
                //TODO check the compatiblity
                let compat_config = Compatibility::full_check();
                self.session.publish_module_bundle_with_compat_config(
                    module_bundle,
                    sender,
                    &mut self.gas_meter,
                    compat_config,
                )?;
                self.execute_init_modules(init_function_modules)
            }
        };

        self.resolve_pending_init_functions()?;

        // Check if there are modules upgrading
        let module_flag = self
            .ctx
            .tx_context
            .get::<ModuleUpgradeFlag>()
            .map_err(|e| {
                PartialVMError::new(StatusCode::UNKNOWN_VALIDATION_STATUS)
                    .with_message(e.to_string())
                    .finish(Location::Undefined)
            })?;
        if let Some(flag) = module_flag {
            if flag.is_upgrade {
                self.vm.mark_loader_cache_as_invalid();
            }
        }

        action_result
    }

    /// Resolve pending init functions request registered via the NativeModuleContext.
    fn resolve_pending_init_functions(&mut self) -> VMResult<()> {
        let ctx = self
            .session
            .get_native_extensions_mut()
            .get_mut::<NativeModuleContext>();
        let init_functions = ctx.init_functions.clone();
        if !init_functions.is_empty() {
            self.execute_init_modules(init_functions)
        } else {
            Ok(())
        }
    }

    // Because the StorageContext can be mut argument, if the function change the StorageContext,
    // we need to update the StorageContext via return values, and pass the updated StorageContext to the next function.
    fn update_storage_context_via_return_values(&mut self, return_values: &SerializedReturnValues) {
        //The only mutable reference output is &mut StorageContext
        debug_assert!(
            return_values.mutable_reference_outputs.len() <= 1,
            "The function should not return more than one mutable reference"
        );

        if let Some((_index, value, _layout)) = return_values.mutable_reference_outputs.get(0) {
            //TODO check the type with local index
            let returned_storage_context = StorageContext::from_bytes(value.as_slice())
                .expect("The return mutable reference should be a StorageContext");
            if log::log_enabled!(log::Level::Debug) {
                log::debug!(
                    "The returned storage context is {:?}",
                    returned_storage_context
                );
            }
            self.ctx = returned_storage_context;
        }
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
            .map_err(|e: move_binary_format::errors::PartialVMError| {
                e.finish(Location::Undefined)
            })?;

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

                let type_tag = match ty {
                    Type::Reference(ty) | Type::MutableReference(ty) => {
                        self.session.get_type_tag(ty)?
                    }
                    _ => self.session.get_type_tag(ty)?,
                };

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
    ) -> VMResult<(TxContext, TransactionOutput)> {
        let (finalized_session, status) = match vm_status.keep_or_discard() {
            Ok(status) => self.post_execute(status),
            Err(discard_status) => {
                //This should not happen, if it happens, it means that the VM or verifer has a bug
                //TODO try to handle this error
                panic!("Discard status: {:?}", discard_status);
            }
        };

        let MoveOSSession {
            vm: _,
            remote: _,
            session,
            ctx,
            table_data,
            pre_execute_functions: _,
            post_execute_functions: _,
            gas_meter: _,
            read_only,
        } = finalized_session;
        let (changeset, raw_events, extensions) = session.finish_with_extensions()?;
        drop(extensions);

        let state_changeset =
            into_change_set(table_data).map_err(|e| e.finish(Location::Undefined))?;

        if read_only {
            if !changeset.accounts().is_empty() {
                return Err(PartialVMError::new(StatusCode::UNKNOWN_VALIDATION_STATUS)
                    .with_message("ChangeSet should be empty as never used.".to_owned())
                    .finish(Location::Undefined));
            }

            if !raw_events.is_empty() {
                return Err(PartialVMError::new(StatusCode::UNKNOWN_VALIDATION_STATUS)
                    .with_message("Events should be empty as never used.".to_owned())
                    .finish(Location::Undefined));
            }

            if !state_changeset.changes.is_empty() {
                return Err(PartialVMError::new(StatusCode::UNKNOWN_VALIDATION_STATUS)
                    .with_message("Table change set should be empty as never used.".to_owned())
                    .finish(Location::Undefined));
            }

            if ctx.tx_context.ids_created > 0 {
                return Err(PartialVMError::new(StatusCode::UNKNOWN_VALIDATION_STATUS)
                    .with_message("TxContext::ids_created should be zero as never used.".to_owned())
                    .finish(Location::Undefined));
            }
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
            ctx.tx_context,
            TransactionOutput {
                status,
                changeset,
                state_changeset,
                events,
                gas_used,
            },
        ))
    }

    fn pre_execute(self) -> Self {
        // the read_only function should not execute pre_execute function
        // this ensure via the check in new_session
        let mut pre_execute_session = self;
        for function_call in pre_execute_session.pre_execute_functions.clone() {
            let pre_execute_function_id = function_call.function_id.clone();
            let result = pre_execute_session.execute_function_bypass_visibility(function_call);
            if let Err(e) = result {
                //TODO handle pre_execute function error
                //Because if we allow user to write pre_execute function, we need to handle the error
                log::error!(
                    "pre_execute function {} error: {:?}",
                    pre_execute_function_id,
                    e
                );
                panic!("pre_execute function should success")
            }
        }
        pre_execute_session
    }

    fn post_execute(self, execute_status: KeptVMStatus) -> (Self, KeptVMStatus) {
        if self.read_only {
            (self, execute_status)
        } else {
            let mut post_execute_session = match &execute_status {
                KeptVMStatus::Executed => self,
                _error => {
                    //if the execution failed, we need to start a new session, and discard the transaction changes
                    // and increment the sequence number or reduce the gas in new session.
                    self.respawn()
                }
            };
            for function_call in post_execute_session.post_execute_functions.clone() {
                //TODO handle post_execute function error
                post_execute_session
                    .execute_function_bypass_visibility(function_call)
                    .expect("post_execute function should always success");
            }
            (post_execute_session, execute_status)
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

    pub fn runtime_session(&self) -> &Session<'r, 'l, MoveosDataCache<'r, 'l, S>> {
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
