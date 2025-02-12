// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::data_cache::{into_change_set, MoveosDataCache};
use crate::moveos::MoveOSCacheManager;
use crate::vm::module_cache::{
    GlobalModuleCache, MoveOSCodeCache, PanicError, RoochModuleExtension, StateValue,
};
use ambassador::delegate_to_methods;
use bytes::Bytes;
use move_binary_format::compatibility::Compatibility;
use move_binary_format::file_format::CompiledScript;
use move_binary_format::normalized;
use move_binary_format::{
    access::ModuleAccess,
    errors::{verification_error, Location, PartialVMError, PartialVMResult, VMError, VMResult},
    file_format::AbilitySet,
    CompiledModule, IndexKind,
};
use move_core_types::identifier::IdentStr;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    value::MoveTypeLayout,
    vm_status::{KeptVMStatus, StatusCode},
};
use move_model::script_into_module;
use move_vm_runtime::data_cache::TransactionCache;
use move_vm_runtime::module_traversal::{TraversalContext, TraversalStorage};
use move_vm_runtime::{
    config::VMConfig, move_vm::MoveVM, native_extensions::NativeContextExtensions,
    native_functions::NativeFunction, session::Session, CodeStorage, LoadedFunction, Module,
    ModuleStorage, RuntimeEnvironment, Script, WithRuntimeEnvironment,
};
use move_vm_types::code::{ambassador_impl_ScriptCache, WithBytes, WithHash};
use move_vm_types::code::{Code, ModuleCode};
use move_vm_types::code::{ModuleCache, ScriptCache, UnsyncModuleCache, UnsyncScriptCache};
use move_vm_types::gas::UnmeteredGasMeter;
use move_vm_types::loaded_data::runtime_types::{StructNameIndex, StructType, Type};
use move_vm_types::sha3_256;
use moveos_common::types::{ClassifiedGasMeter, SwitchableGasMeter};
use moveos_object_runtime::runtime::{ObjectRuntime, ObjectRuntimeContext};
use moveos_object_runtime::TypeLayoutLoader;
use moveos_stdlib::natives::moveos_stdlib::{
    event::NativeEventContext, move_module::NativeModuleContext,
};
use moveos_store::{load_feature_store_object, MoveOSStore, TxnIndex};
use moveos_types::state::ObjectState;
use moveos_types::{addresses, transaction::RawTransactionOutput};
use moveos_types::{
    function_return_value::FunctionReturnValue,
    move_std::string::MoveString,
    move_types::FunctionId,
    moveos_std::copyable_any::Any,
    moveos_std::simple_map::SimpleMap,
    moveos_std::tx_context::TxContext,
    moveos_std::{event::TransactionEvent, module_upgrade_flag::ModuleUpgradeFlag},
    state_resolver::MoveOSResolver,
    transaction::{FunctionCall, MoveAction, VerifiedMoveAction},
};
use moveos_verifier::verifier::INIT_FN_NAME_IDENTIFIER;
use parking_lot::RwLock;
use std::collections::BTreeSet;
use std::rc::Rc;
use std::{borrow::Borrow, sync::Arc};

/// MoveOSVM is a wrapper of MoveVM with MoveOS specific features.
pub struct MoveOSVM {
    inner: MoveVM,
}

impl MoveOSVM {
    pub fn new(moveos_cache_manager: MoveOSCacheManager) -> VMResult<Self> {
        let env = moveos_cache_manager.runtime_environment;
        Ok(Self {
            inner: MoveVM::new_with_runtime_environment(&env),
        })
    }

    pub fn new_session<
        'r,
        S: MoveOSResolver,
        G: SwitchableGasMeter + ClassifiedGasMeter + Clone,
    >(
        &self,
        remote: &'r S,
        ctx: TxContext,
        gas_meter: G,
        global_module_cache: Arc<
            RwLock<GlobalModuleCache<ModuleId, CompiledModule, Module, RoochModuleExtension>>,
        >,
        runtime_environment: &'r RuntimeEnvironment,
    ) -> MoveOSSession<'r, '_, S, G> {
        let root = remote.root();
        let object_runtime = Rc::new(RwLock::new(ObjectRuntime::new(ctx, root.clone(), remote)));
        MoveOSSession::new(
            &self.inner,
            remote,
            object_runtime,
            gas_meter,
            false,
            global_module_cache.clone(),
            runtime_environment,
        )
    }

    pub fn new_genesis_session<'r, S: MoveOSResolver>(
        &self,
        remote: &'r S,
        ctx: TxContext,
        genesis_objects: Vec<(ObjectState, MoveTypeLayout)>,
        global_module_cache: Arc<
            RwLock<GlobalModuleCache<ModuleId, CompiledModule, Module, RoochModuleExtension>>,
        >,
        runtime_environment: &'r RuntimeEnvironment,
    ) -> MoveOSSession<'r, '_, S, UnmeteredGasMeter> {
        let root = remote.root();
        let object_runtime = Rc::new(RwLock::new(ObjectRuntime::genesis(
            ctx,
            root.clone(),
            remote,
            genesis_objects,
        )));
        //Do not charge gas for genesis session
        // Genesis session do not need to execute pre_execute and post_execute function
        MoveOSSession::new(
            &self.inner,
            remote,
            object_runtime,
            UnmeteredGasMeter,
            false,
            global_module_cache.clone(),
            runtime_environment,
        )
    }

    pub fn new_readonly_session<
        'r,
        S: MoveOSResolver,
        G: SwitchableGasMeter + ClassifiedGasMeter + Clone,
    >(
        &self,
        remote: &'r S,
        ctx: TxContext,
        gas_meter: G,
        global_module_cache: Arc<
            RwLock<GlobalModuleCache<ModuleId, CompiledModule, Module, RoochModuleExtension>>,
        >,
        runtime_environment: &'r RuntimeEnvironment,
    ) -> MoveOSSession<'r, '_, S, G> {
        let root = remote.root();
        let object_runtime = Rc::new(RwLock::new(ObjectRuntime::new(ctx, root.clone(), remote)));
        MoveOSSession::new(
            &self.inner,
            remote,
            object_runtime,
            gas_meter,
            true,
            global_module_cache.clone(),
            runtime_environment,
        )
    }

    pub fn mark_loader_cache_as_invalid(&self) {
        self.inner.mark_loader_cache_as_invalid()
    }

    pub fn inner(&self) -> &MoveVM {
        &self.inner
    }
}

fn into_module_cache_iterator(
    compiled_modules: &Vec<CompiledModule>,
    module_bundle: &Vec<Vec<u8>>,
    _runtime_environment: &RuntimeEnvironment,
) -> impl Iterator<
    Item = (
        ModuleId,
        Arc<ModuleCode<CompiledModule, Module, RoochModuleExtension>>,
    ),
> {
    let mut module_code_list = vec![];
    for (idx, m) in compiled_modules.iter().enumerate() {
        let bytes = module_bundle[idx].clone();
        let extension = Arc::new(RoochModuleExtension::new(StateValue::new_legacy(
            Bytes::copy_from_slice(bytes.as_slice()),
        )));
        let module = ModuleCode::from_deserialized(m.clone(), extension);
        let module_code = Arc::new(module);

        module_code_list.push((m.self_id(), module_code));
    }

    module_code_list.into_iter()
}

/// MoveOSSession is a wrapper of MoveVM session with MoveOS specific features.
/// It is used to execute a transaction, every transaction should be executed in a new session.
/// Every session has a TxContext, if the transaction have multiple actions, the TxContext is shared.
pub struct MoveOSSession<'r, 'l, S, G> {
    pub(crate) vm: &'l MoveVM,
    pub(crate) remote: &'r S,
    pub(crate) session: Session<'r, 'l, MoveosDataCache<'r, 'l, S>>,
    pub(crate) object_runtime: Rc<RwLock<ObjectRuntime<'r>>>,
    pub(crate) gas_meter: G,
    pub(crate) code_cache: MoveOSCodeCache<'r, S>,
    pub(crate) read_only: bool,
}

#[allow(clippy::arc_with_non_send_sync)]
impl<'r, 'l, S, G> MoveOSSession<'r, 'l, S, G>
where
    S: MoveOSResolver,
    G: SwitchableGasMeter + ClassifiedGasMeter,
{
    pub fn new(
        vm: &'l MoveVM,
        remote: &'r S,
        object_runtime: Rc<RwLock<ObjectRuntime<'r>>>,
        gas_meter: G,
        read_only: bool,
        global_module_cache: Arc<
            RwLock<GlobalModuleCache<ModuleId, CompiledModule, Module, RoochModuleExtension>>,
        >,
        runtime_environment: &'r RuntimeEnvironment,
    ) -> Self {
        Self {
            vm,
            remote,
            session: Self::new_inner_session(
                vm,
                remote,
                object_runtime.clone(),
                global_module_cache.clone(),
                runtime_environment,
            ),
            object_runtime,
            gas_meter,
            code_cache: MoveOSCodeCache::new(global_module_cache.clone(), runtime_environment, remote),
            read_only,
        }
    }

    /// Re spawn a new session with the same context.
    pub fn respawn(self, env: SimpleMap<MoveString, Any>) -> Self {
        let ctx = self.object_runtime.read().tx_context().clone();
        let new_ctx = ctx.spawn(env);
        // We get the root object from the remote, because the root object may be changed during the transaction execution
        let root = self.remote.root().clone();
        let object_runtime = Rc::new(RwLock::new(ObjectRuntime::new(new_ctx, root, self.remote)));
        Self {
            session: Self::new_inner_session(
                self.vm,
                self.remote,
                object_runtime.clone(),
                self.code_cache.global_module_cache.clone(),
                self.code_cache.runtime_environment,
            ),
            object_runtime,
            ..self
        }
    }

    fn new_inner_session(
        vm: &'l MoveVM,
        remote: &'r S,
        object_runtime: Rc<RwLock<ObjectRuntime<'r>>>,
        global_module_cache: Arc<
            RwLock<GlobalModuleCache<ModuleId, CompiledModule, Module, RoochModuleExtension>>,
        >,
        runtime_environment: &'r RuntimeEnvironment,
    ) -> Session<'r, 'l, MoveosDataCache<'r, 'l, S>> {
        let mut extensions = NativeContextExtensions::default();

        let feature_store = load_feature_store_object(remote);
        extensions.add(ObjectRuntimeContext::new(
            object_runtime.clone(),
            feature_store,
        ));
        extensions.add(NativeModuleContext::new(remote));
        extensions.add(NativeEventContext::default());

        // The VM code loader has bugs around module upgrade. After a module upgrade, the internal
        // cache needs to be flushed to work around those bugs.
        // vm.mark_loader_cache_as_invalid();
        // vm.flush_loader_cache_if_invalidated();
        let loader = vm.runtime.loader();
        let data_store: MoveosDataCache<'r, 'l, S> = MoveosDataCache::new(
            remote,
            loader,
            object_runtime,
            MoveOSCodeCache::new(global_module_cache.clone(), runtime_environment, remote),
        );
        vm.new_session_with_extensions_legacy(data_store, extensions)
    }

    pub(crate) fn tx_context(&self) -> TxContext {
        self.object_runtime.read().tx_context().clone()
    }

    /// Verify a move action.
    /// The caller should call this function when validate a transaction.
    /// If the result is error, the transaction should be rejected.
    pub fn verify_move_action(&mut self, action: MoveAction) -> VMResult<VerifiedMoveAction> {
        match action {
            MoveAction::Script(call) => {
                let loaded_function = self.session.load_script(
                    &self.code_cache,
                    call.code.as_slice(),
                    call.ty_args.as_slice(),
                )?;
                let location = Location::Script;
                moveos_verifier::verifier::verify_entry_function(
                    &loaded_function,
                    &self.session,
                    &self.code_cache,
                )
                .map_err(|e| e.finish(location.clone()))?;
                let _serialized_args = self.resolve_argument(
                    &loaded_function,
                    call.args.clone(),
                    location,
                    false,
                    &self.code_cache,
                )?;

                let compiled_script_opt = CompiledScript::deserialize(call.code.as_slice());
                let compiled_script = match compiled_script_opt {
                    Ok(v) => v,
                    Err(err) => return Err(err.finish(Location::Undefined)),
                };
                let script_module = script_into_module(compiled_script, "");
                let modules = vec![script_module];
                let result = moveos_verifier::verifier::verify_modules(&modules, self.remote);
                match result {
                    Ok(_) => {}
                    Err(err) => return Err(err),
                }

                Ok(VerifiedMoveAction::Script { call })
            }
            MoveAction::Function(call) => {
                let loaded_function = self.session.load_function(
                    &self.code_cache,
                    &call.function_id.module_id,
                    &call.function_id.function_name,
                    call.ty_args.as_slice(),
                )?;
                let location = Location::Module(call.function_id.module_id.clone());
                moveos_verifier::verifier::verify_entry_function(
                    &loaded_function,
                    &self.session,
                    &self.code_cache,
                )
                .map_err(|e| e.finish(location.clone()))?;
                let _resolved_args = self.resolve_argument(
                    &loaded_function,
                    call.args.clone(),
                    location,
                    false,
                    &self.code_cache,
                )?;
                Ok(VerifiedMoveAction::Function {
                    call,
                    bypass_visibility: false,
                })
            }
            MoveAction::ModuleBundle(module_bundle) => {
                let sender = self.tx_context().sender();
                // Publishing modules through `MoveAction::ModuleBundle` is only allowed for
                // system reserved addresses. Developers can publish modules through Move function
                // `moveos_std::move_module::publish_modules`
                if !addresses::is_system_reserved_address(sender) {
                    return Err(PartialVMError::new(StatusCode::INVALID_MODULE_PUBLISHER)
                        .finish(Location::Undefined));
                };
                let compiled_modules = deserialize_modules(&module_bundle)?;

                //#TODO: disable the verification process temporary
                /*
                let result =
                    moveos_verifier::verifier::verify_modules(&compiled_modules, self.remote);
                match result {
                    Ok(_) => {}
                    Err(err) => return Err(err),
                }
                 */

                /*
                self.vm
                    .runtime
                    .loader()
                    .verify_module_bundle_for_publication(
                        compiled_modules.as_slice(),
                        &self.session.data_cache,
                    )?;
                 */

                let mut init_function_modules = vec![];

                for module in &compiled_modules {
                    let result = moveos_verifier::verifier::verify_init_function(module);
                    match result {
                        Ok(res) => {
                            if res {
                                init_function_modules.push(module.self_id())
                            }
                        }
                        Err(err) => return Err(err),
                    }
                }

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
        let traversal_storage = TraversalStorage::new();
        let mut traversal_context = TraversalContext::new(&traversal_storage);

        let action_result = match action {
            VerifiedMoveAction::Script { call } => {
                let loaded_function = self.session.load_script(
                    &self.code_cache,
                    call.code.as_slice(),
                    call.ty_args.as_slice(),
                )?;
                let location: Location = Location::Script;
                let serialized_args = self.resolve_argument(
                    &loaded_function,
                    call.args,
                    location,
                    true,
                    &self.code_cache,
                )?;

                self.session.execute_script(
                    call.code,
                    call.ty_args,
                    serialized_args,
                    &mut self.gas_meter,
                    &mut traversal_context,
                    &self.code_cache,
                )
            }
            VerifiedMoveAction::Function {
                call,
                bypass_visibility,
            } => {
                let loaded_function = self.session.load_function(
                    &self.code_cache,
                    &call.function_id.module_id,
                    &call.function_id.function_name,
                    call.ty_args.as_slice(),
                )?;
                let location = Location::Module(call.function_id.module_id.clone());
                let serialized_args = self.resolve_argument(
                    &loaded_function,
                    call.args.clone(),
                    location,
                    true,
                    &self.code_cache,
                )?;
                if bypass_visibility {
                    // bypass visibility call is system call, such as execute L1 block transaction
                    self.session
                        .execute_function_bypass_visibility(
                            &call.function_id.module_id,
                            &call.function_id.function_name,
                            call.ty_args.clone(),
                            serialized_args,
                            &mut self.gas_meter,
                            &mut traversal_context,
                            &self.code_cache,
                        )
                        .map(|ret| {
                            debug_assert!(
                                ret.return_values.is_empty(),
                                "Function should not return values"
                            );
                        })
                } else {
                    let loaded_function = self.session.load_function(
                        &self.code_cache,
                        &call.function_id.module_id,
                        &call.function_id.function_name,
                        call.ty_args.as_slice(),
                    )?;
                    self.session.execute_entry_function(
                        loaded_function,
                        serialized_args,
                        &mut self.gas_meter,
                        &mut traversal_context,
                        &self.code_cache,
                    )
                }
            }
            VerifiedMoveAction::ModuleBundle {
                module_bundle,
                init_function_modules,
            } => {
                let sender = self.tx_context().sender();
                // Check if module is first published. Only the first published module can run init function
                let modules_with_init = init_function_modules
                    .into_iter()
                    .filter(|m| self.session.get_data_store().exists_module(m) == Ok(false))
                    .collect();

                // The following code is copy from session.publish_module_bundle_with_compat_config
                // We need do some modification to support skip the check if the sender is a system reserved address
                // self.session.publish_module_bundle_with_compat_config(
                //     module_bundle,
                //     sender,
                //     &mut self.gas_meter,
                //     compat,
                // )?;

                // deserialize the modules. Perform bounds check. After this indexes can be
                // used with the `[]` operator
                let compiled_modules = match module_bundle
                    .iter()
                    .map(|blob| CompiledModule::deserialize(blob))
                    .collect::<PartialVMResult<Vec<_>>>()
                {
                    Ok(modules) => modules,
                    Err(err) => {
                        return Err(err
                            .append_message_with_separator(
                                '\n',
                                "[VM] module deserialization failed".to_string(),
                            )
                            .finish(Location::Undefined));
                    }
                };

                // Check if the sender address matches the module address
                // skip the check if the sender is a system reserved address
                if !addresses::is_system_reserved_address(sender) {
                    for module in &compiled_modules {
                        if module.address() != &sender {
                            return Err(verification_error(
                                StatusCode::MODULE_ADDRESS_DOES_NOT_MATCH_SENDER,
                                IndexKind::AddressIdentifier,
                                module.self_handle_idx().0,
                            )
                            .finish(Location::Undefined));
                        }
                    }
                }

                for (idx, m) in compiled_modules.iter().enumerate() {
                    let bytes = module_bundle[idx].clone();
                    let extension = Arc::new(RoochModuleExtension::new(StateValue::new_legacy(
                        Bytes::copy_from_slice(bytes.as_slice()),
                    )));
                    self.code_cache.module_cache.insert_deserialized_module(
                        m.self_id(),
                        m.clone(),
                        extension,
                        Some(1),
                    )?
                }

                /*
                let module_code_list = into_module_cache_iterator(
                    &compiled_modules,
                    &module_bundle,
                    self.code_cache.runtime_environment,
                );
                self.insert_global_module_cache(module_code_list)?;

                 */

                // Collect ids for modules that are published together
                let mut bundle_unverified = BTreeSet::new();

                let data_store = self.session.get_data_store();

                let compat = Compatibility::full_check();
                for module in &compiled_modules {
                    let module_id = module.self_id();

                    if data_store.exists_module(&module_id)? && compat.need_check_compat() {
                        let old_module = self.vm.load_module(&module_id, self.remote)?;
                        compat
                            .check(&old_module, &module)
                            .map_err(|e| e.finish(Location::Undefined))?;
                    }
                    if !bundle_unverified.insert(module_id) {
                        return Err(PartialVMError::new(StatusCode::DUPLICATE_MODULE_NAME)
                            .finish(Location::Undefined));
                    }
                }

                /*
                // Perform bytecode and loading verification. Modules must be sorted in topological order.
                self.vm
                    .runtime
                    .loader()
                    .verify_module_bundle_for_publication(&compiled_modules, data_store)?;
                 */

                for (module, blob) in compiled_modules.into_iter().zip(module_bundle.into_iter()) {
                    let is_republishing = data_store.exists_module(&module.self_id())?;
                    if is_republishing {
                        // This is an upgrade, so invalidate the loader cache, which still contains the
                        // old module.
                        self.vm.mark_loader_cache_as_invalid();
                    }
                    data_store.publish_module(&module.self_id(), blob, is_republishing)?;
                }

                self.execute_init_modules(modules_with_init)
            }
        };

        if action_result.is_ok() {
            self.resolve_pending_init_functions()?;
            // Check if there are modules upgrading
            let module_flag = self.tx_context().get::<ModuleUpgradeFlag>().map_err(|e| {
                PartialVMError::new(StatusCode::UNKNOWN_VALIDATION_STATUS)
                    .with_message(e.to_string())
                    .finish(Location::Undefined)
            })?;
            let is_upgrade = module_flag.map_or(false, |flag| flag.is_upgrade);
            if is_upgrade {
                // the V1 calling of Loader must be disabled
                //self.vm.mark_loader_cache_as_invalid();
            };
        }

        action_result
    }

    fn insert_global_module_cache(
        &mut self,
        modules: impl Iterator<
            Item = (
                ModuleId,
                Arc<ModuleCode<CompiledModule, Module, RoochModuleExtension>>,
            ),
        >,
    ) -> VMResult<()> {
        let mut write_guard = self.code_cache.global_module_cache.write();
        write_guard
            .insert_verified(modules)
            .expect("write_guard.insert_verified failed");

        Ok(())
    }

    /// Resolve pending init functions request registered via the NativeModuleContext.
    fn resolve_pending_init_functions(&mut self) -> VMResult<()> {
        let ctx = self
            .session
            .get_native_extensions_mut()
            .get_mut::<NativeModuleContext>();
        let init_functions = ctx.init_functions.clone();
        if !init_functions.is_empty() {
            self.execute_init_modules(init_functions.into_iter().collect())
        } else {
            Ok(())
        }
    }

    pub fn execute_function_bypass_visibility(
        &mut self,
        call: FunctionCall,
    ) -> VMResult<Vec<FunctionReturnValue>> {
        let traversal_storage = TraversalStorage::new();
        let mut traversal_context = TraversalContext::new(&traversal_storage);

        let loaded_function = self.session.load_function(
            &self.code_cache,
            &call.function_id.module_id,
            &call.function_id.function_name,
            call.ty_args.as_slice(),
        )?;
        let location = Location::Module(call.function_id.module_id.clone());
        let serialized_args = self.resolve_argument(
            &loaded_function,
            call.args,
            location,
            true,
            &self.code_cache,
        )?;
        let return_values = self.session.execute_function_bypass_visibility(
            &call.function_id.module_id,
            &call.function_id.function_name.as_ident_str(),
            call.ty_args,
            serialized_args,
            &mut self.gas_meter,
            &mut traversal_context,
            &self.code_cache,
        )?;
        return_values
            .return_values
            .into_iter()
            .zip(loaded_function.return_tys().iter())
            .map(|((v, _layout), ty)| {
                // We can not use
                // let type_tag :TypeTag = TryInto::try_into(&layout)?
                // to get TypeTag from MoveTypeLayout, because this MoveTypeLayout not MoveLayoutType::WithTypes
                // Invalid MoveTypeLayout -> StructTag conversion--needed MoveLayoutType::WithTypes

                let type_tag = match ty {
                    Type::Reference(ty) | Type::MutableReference(ty) => {
                        self.session.get_type_tag(ty, &self.code_cache)?
                    }
                    _ => self.session.get_type_tag(ty, &self.code_cache)?,
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
            if tracing::enabled!(tracing::Level::TRACE) {
                tracing::trace!(
                    "Execute init function for module: {:?}",
                    module_id.to_string()
                );
            }
            self.execute_function_bypass_visibility(call)
                .map(|result| {
                    debug_assert!(result.is_empty(), "Init function must not return value")
                })?;
        }

        Ok(())
    }

    pub fn finish_with_extensions(
        self,
        status: KeptVMStatus,
    ) -> VMResult<(TxContext, RawTransactionOutput)> {
        let gas_used = self.query_gas_used();
        // let is_read_only_execution = self.read_only;
        let MoveOSSession {
            vm: _,
            remote: _,
            session,
            object_runtime,
            gas_meter: _,
            read_only,
            code_cache: _,
        } = self;
        let (changeset, mut extensions) = session.finish_with_extensions(&self.code_cache)?;
        //We do not use the event API from data_cache. Instead, we use the NativeEventContext
        //debug_assert!(raw_events.is_empty());
        //We do not use the account, resource, and module API from data_cache. Instead, we use the ObjectRuntimeContext
        debug_assert!(changeset.accounts().is_empty());
        drop(changeset);
        //drop(raw_events);

        let event_context = extensions.remove::<NativeEventContext>();
        let raw_events = event_context.into_events();
        drop(extensions);

        let (ctx, state_changeset) =
            into_change_set(object_runtime).map_err(|e| e.finish(Location::Undefined))?;

        if read_only {
            if !state_changeset.changes.is_empty() {
                return Err(PartialVMError::new(StatusCode::UNKNOWN_VALIDATION_STATUS)
                    .with_message(
                        "State change set should be empty when execute readonly function."
                            .to_owned(),
                    )
                    .finish(Location::Undefined));
            }

            if ctx.ids_created > 0 {
                return Err(PartialVMError::new(StatusCode::UNKNOWN_VALIDATION_STATUS)
                    .with_message("TxContext::ids_created should be zero as never used.".to_owned())
                    .finish(Location::Undefined));
            }
            //We do not check the event, we allow read_only session to emit event now.
        }

        let events: Vec<_> = raw_events
            .into_iter()
            .enumerate()
            .map(|(i, (struct_tag, event_data))| {
                Ok(TransactionEvent::new(struct_tag, event_data, i as u64))
            })
            .collect::<VMResult<_>>()?;

        // Check if there are modules upgrading
        let module_flag = ctx.get::<ModuleUpgradeFlag>().map_err(|e| {
            PartialVMError::new(StatusCode::UNKNOWN_VALIDATION_STATUS)
                .with_message(e.to_string())
                .finish(Location::Undefined)
        })?;
        let is_upgrade = module_flag.map_or(false, |flag| flag.is_upgrade);

        //TODO cleanup
        // match gas_meter.charge_change_set(&state_changeset) {
        //     Ok(_) => {}
        //     Err(partial_vm_error) => {
        //         return Err(partial_vm_error
        //             .with_message(
        //                 "An error occurred during the charging of the change set".to_owned(),
        //             )
        //             .finish(Location::Undefined));
        //     }
        // }

        // Temporary behavior, will enable this in the future.
        /*
        match gas_meter.charge_event(events.as_slice()) {
            Ok(_) => {}
            Err(partial_vm_error) => {
                return Err(partial_vm_error
                    .with_message("An error occurred during the charging of the events".to_owned())
                    .finish(Location::Undefined));
            }
        }

        match gas_meter.check_constrains(ctx.tx_context.max_gas_amount) {
            Ok(_) => {}
            Err(partial_vm_err) => {
                return Err(partial_vm_err.finish(Location::Undefined));
            }
        };

        let mut gas_statement = gas_meter.gas_statement();
        if is_read_only_execution {
            gas_statement.execution_gas_used = 0;
            gas_statement.storage_gas_used = 0;
        }
         */

        Ok((
            ctx,
            RawTransactionOutput {
                status,
                changeset: state_changeset,
                events,
                gas_used,
                is_upgrade,
                is_gas_upgrade: false,
            },
        ))
    }

    pub fn execute_function_call(
        &mut self,
        functions: Vec<FunctionCall>,
        meter_gas: bool,
    ) -> VMResult<()> {
        if !meter_gas {
            self.gas_meter.stop_metering();
        }
        for function_call in functions {
            let result = self.execute_function_bypass_visibility(function_call);
            match result {
                Ok(return_values) => {
                    // This function is only used in crates. No return values are expected.
                    assert!(
                        return_values.is_empty(),
                        "Function should not return values"
                    )
                }
                Err(e) => {
                    if !meter_gas {
                        self.gas_meter.start_metering();
                    }
                    return Err(e);
                }
            }
        }
        if !meter_gas {
            self.gas_meter.start_metering();
        }
        Ok(())
    }

    pub(crate) fn query_gas_used(&self) -> u64 {
        let max_gas_amount = self.tx_context().max_gas_amount;
        let gas_left: u64 = self.gas_meter.balance_internal().into();
        max_gas_amount.checked_sub(gas_left).unwrap_or_else(
                || panic!("gas_left({gas_left}) should always be less than or equal to max gas amount({max_gas_amount})")
            )
    }

    /// Load a script and all of its types into cache
    pub fn load_script(
        &mut self,
        script: impl Borrow<[u8]>,
        ty_args: Vec<TypeTag>,
    ) -> VMResult<LoadedFunction> {
        self.session
            .load_script(&self.code_cache, script, ty_args.as_slice())
    }

    /// Load a module, a function, and all of its types into cache
    pub fn load_function(
        &mut self,
        function_id: &FunctionId,
        type_arguments: &[TypeTag],
    ) -> VMResult<LoadedFunction> {
        self.session.load_function(
            &self.code_cache,
            &function_id.module_id,
            &function_id.function_name,
            type_arguments,
        )
    }

    /// Try to get the type tag of a type, but return None if the type is not a struct or struct instantiation
    /// This function also support struct reference and mutable reference
    pub fn get_type_tag_option(&self, t: &Type) -> Option<TypeTag> {
        match t {
            Type::Struct { idx: _, ability: _ }
            | Type::StructInstantiation {
                idx: _,
                ty_args: _,
                ability: _,
            } => self.session.get_type_tag(t, &self.code_cache).ok(),
            Type::Reference(r) => self.get_type_tag_option(r),
            Type::MutableReference(r) => self.get_type_tag_option(r),
            _ => None,
        }
    }

    pub fn get_type_tag(&self, ty: &Type) -> VMResult<TypeTag> {
        self.session.get_type_tag(ty, &self.code_cache)
    }

    pub fn load_type(&mut self, type_tag: &TypeTag) -> VMResult<Type> {
        self.session.load_type(type_tag, &self.code_cache)
    }

    pub fn get_fully_annotated_type_layout(
        &mut self,
        type_tag: &TypeTag,
    ) -> VMResult<MoveTypeLayout> {
        self.session
            .get_fully_annotated_type_layout(type_tag, &self.code_cache)
    }

    pub fn get_struct_type(&self, index: StructNameIndex) -> Option<Arc<StructType>> {
        self.session.fetch_struct_ty_by_idx(index, &self.code_cache)
    }

    pub fn get_type_abilities(&self, ty: &Type) -> VMResult<AbilitySet> {
        match ty.abilities() {
            Ok(v) => Ok(v),
            Err(e) => Err(e.finish(Location::Undefined)),
        }
    }

    pub fn get_data_store(&mut self) -> &mut dyn TransactionCache {
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
