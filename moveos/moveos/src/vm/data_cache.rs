// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_vm_runtime::loader::Loader;
use std::collections::BTreeMap;

use crate::vm::module_cache::MoveOSCodeCache;
use bytes::Bytes;
use move_binary_format::deserializer::DeserializerConfig;
use move_binary_format::errors::{Location, PartialVMError, PartialVMResult, VMResult};
use move_binary_format::file_format::CompiledScript;
use move_binary_format::CompiledModule;
use move_core_types::effects::Changes;
use move_core_types::language_storage::TypeTag;
use move_core_types::{
    account_address::AccountAddress, effects::ChangeSet, gas_algebra::NumBytes,
    language_storage::ModuleId, value::MoveTypeLayout, vm_status::StatusCode,
};
use move_vm_runtime::data_cache::TransactionCache;
use move_vm_runtime::loader::modules::{LegacyModuleStorage, LegacyModuleStorageAdapter};
use move_vm_runtime::logging::expect_no_verification_errors;
use move_vm_runtime::ModuleStorage;
use move_vm_types::{
    loaded_data::runtime_types::Type,
    values::{GlobalValue, Value},
};
use moveos_object_runtime::{runtime::ObjectRuntime, TypeLayoutLoader};
use moveos_types::state::{FieldKey, StateChangeSet};
use moveos_types::{moveos_std::tx_context::TxContext, state_resolver::MoveOSResolver};
use parking_lot::RwLock;
use sha3::{Digest, Sha3_256};
use std::rc::Rc;
use std::sync::Arc;

/// Transaction data cache. Keep updates within a transaction so they can all be published at
/// once when the transaction succeeds.
///
/// It also provides an implementation for the opcodes that refer to storage and gives the
/// proper guarantees of reference lifetime.
///
/// Dirty objects are serialized and returned in make_write_set.
///
/// It is a responsibility of the client to publish changes once the transaction is executed.
///
/// The Move VM takes a `DataStore` in input and this is the default and correct implementation
/// for a data store related to a transaction. Clients should create an instance of this type
/// and pass it to the Move VM.
pub struct MoveosDataCache<'r, 'l, S> {
    resolver: &'r S,
    loader: &'l Loader,
    event_data: Vec<(Vec<u8>, u64, Type, MoveTypeLayout, Value)>,
    object_runtime: Rc<RwLock<ObjectRuntime<'r>>>,

    // Caches to help avoid duplicate deserialization calls.
    compiled_scripts: BTreeMap<[u8; 32], Arc<CompiledScript>>,
    compiled_modules: BTreeMap<ModuleId, (Arc<CompiledModule>, usize, [u8; 32])>,

    code_cache: MoveOSCodeCache<'r, S>,
}

impl<'r, 'l, S: MoveOSResolver> MoveosDataCache<'r, 'l, S> {
    /// Create a `MoveosDataCache` with a `RemoteCache` that provides access to data
    /// not updated in the transaction.
    pub fn new(
        resolver: &'r S,
        loader: &'l Loader,
        object_runtime: Rc<RwLock<ObjectRuntime<'r>>>,
        code_cache: MoveOSCodeCache<'r, S>,
    ) -> Self {
        MoveosDataCache {
            resolver,
            loader,
            event_data: vec![],
            object_runtime,
            compiled_scripts: BTreeMap::new(),
            compiled_modules: BTreeMap::new(),
            code_cache,
        }
    }
}

impl<'r, 'l, S: MoveOSResolver> TransactionCache for MoveosDataCache<'r, 'l, S> {
    /// Make a write set from the updated (dirty, deleted) global resources along with
    /// published modules.
    ///
    /// Gives all proper guarantees on lifetime of global data as well.
    fn into_effects(
        self,
        loader: &Loader,
        module_storage: &dyn ModuleStorage,
    ) -> PartialVMResult<ChangeSet> {
        let resource_converter =
            |value: Value, layout: MoveTypeLayout, _: bool| -> PartialVMResult<Bytes> {
                value
                    .simple_serialize(&layout)
                    .map(Into::into)
                    .ok_or_else(|| {
                        PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                            .with_message(format!("Error when serializing resource {}.", value))
                    })
            };
        self.into_custom_effects(&resource_converter, loader, module_storage)
    }

    fn num_mutated_accounts(&self, _sender: &AccountAddress) -> u64 {
        //TODO load from table data
        todo!("num_mutated_accounts")
    }

    // Retrieve data from the local cache or loads it from the resolver cache into the local cache.
    // All operations on the global data are based on this API and they all load the data
    // into the cache.
    /// In Rooch, all global operations are disable, so this function is never called.
    fn load_resource(
        &mut self,
        _loader: &Loader,
        _module_storage: &dyn ModuleStorage,
        _addr: AccountAddress,
        _ty: &Type,
        _module_store: &LegacyModuleStorageAdapter,
    ) -> PartialVMResult<(&mut GlobalValue, Option<NumBytes>)> {
        unreachable!("Global operations are disabled")
    }

    /// Get the serialized format of a `CompiledModule` given a `ModuleId`.
    fn load_module(&self, module_id: &ModuleId) -> PartialVMResult<Bytes> {
        //if we use object_runtime.write() here, it will cause a deadlock
        //TODO refactor DataCache and ObjectRuntime to avoid this deadlock
        let object_runtime = self.object_runtime.read();

        let result = object_runtime
            .get_loaded_module(module_id)
            .and_then(|module| match module {
                Some(move_module) => {
                    if tracing::enabled!(tracing::Level::TRACE) {
                        tracing::trace!("Loaded module {:?} from ObjectRuntime", module_id);
                    }
                    Ok(Some(move_module.byte_codes))
                }
                None => match self.resolver.get_module(module_id) {
                    Ok(bytes_opt) => match bytes_opt {
                        None => Ok(None),
                        Some(v) => Ok(Some(v.to_vec())),
                    },
                    Err(e) => {
                        let msg = format!("Unexpected storage error: {:?}", e);
                        Err(
                            PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                                .with_message(msg),
                        )
                    }
                },
            });

        match result {
            Ok(Some(code)) => Ok(Bytes::from(code)),
            Ok(None) => {
                let field_key = FieldKey::derive_module_key(module_id.name());
                Err(
                    PartialVMError::new(StatusCode::LINKER_ERROR).with_message(format!(
                        "Cannot find module {:?}(key:{}) in ObjectRuntime and Storage",
                        module_id, field_key,
                    )),
                )
            }
            Err(err) => {
                if tracing::enabled!(tracing::Level::DEBUG) {
                    tracing::warn!("Error loading module {:?}: {:?}", module_id, err);
                }
                Err(err)
            }
        }
    }

    /// Publish a module.
    fn publish_module(
        &mut self,
        module_id: &ModuleId,
        blob: Vec<u8>,
        is_republishing: bool,
    ) -> VMResult<()> {
        let mut object_runtime = self.object_runtime.write();
        object_runtime
            .publish_module(self, self.resolver, module_id, blob, is_republishing)
            .map_err(|e| e.finish(Location::Module(module_id.clone())))
    }

    /// Check if this module exists.
    fn exists_module(&self, module_id: &ModuleId) -> VMResult<bool> {
        let object_runtime = self.object_runtime.read();
        if object_runtime
            .exists_loaded_module(module_id)
            .map_err(|e| e.finish(Location::Module(module_id.clone())))?
        {
            return Ok(true);
        }

        Ok(self
            .resolver
            .get_module(module_id)
            .map_err(|_| {
                PartialVMError::new(StatusCode::STORAGE_ERROR).finish(Location::Undefined)
            })?
            .is_some())
    }

    /*
    fn emit_event(
        &mut self,
        loader: &Loader,
        guid: Vec<u8>,
        seq_num: u64,
        ty: Type,
        val: Value,
    ) -> PartialVMResult<()> {
        let ty_layout = loader.type_to_type_layout(&ty)?;
        self.event_data.push((guid, seq_num, ty, ty_layout, val));
        Ok(())
    }
     */

    fn into_custom_effects<Resource>(
        self,
        _resource_converter: &dyn Fn(Value, MoveTypeLayout, bool) -> PartialVMResult<Resource>,
        _loader: &Loader,
        _module_storage: &dyn ModuleStorage,
    ) -> PartialVMResult<Changes<Bytes, Resource>>
    where
        Self: Sized,
    {
        Ok(Changes::<Bytes, Resource>::new())
    }

    fn num_mutated_resources(&self, _sender: &AccountAddress) -> u64 {
        0
    }

    fn load_compiled_script_to_cache(
        &self,
        script_blob: &[u8],
        _hash_value: [u8; 32],
    ) -> VMResult<Arc<CompiledScript>> {
        let script = match CompiledScript::deserialize_with_config(
            script_blob,
            &DeserializerConfig::default(),
        ) {
            Ok(script) => script,
            Err(err) => {
                let msg = format!("[VM] deserializer for script returned error: {:?}", err);
                return Err(PartialVMError::new(StatusCode::CODE_DESERIALIZATION_ERROR)
                    .with_message(msg)
                    .finish(Location::Script));
            }
        };
        Ok(Arc::new(script))
    }

    fn load_compiled_module_to_cache(
        &self,
        id: ModuleId,
        allow_loading_failure: bool,
    ) -> VMResult<(Arc<CompiledModule>, usize, [u8; 32])> {
        let bytes = match self
            .load_module(&id)
            .map_err(|err| err.finish(Location::Undefined))
        {
            Ok(bytes) => bytes,
            Err(err) if allow_loading_failure => return Err(err),
            Err(err) => {
                return Err(expect_no_verification_errors(err));
            }
        };

        let module =
            CompiledModule::deserialize_with_config(&bytes, &DeserializerConfig::default())
                .map_err(|err| {
                    let msg = format!("Deserialization error: {:?}", err);
                    PartialVMError::new(StatusCode::CODE_DESERIALIZATION_ERROR)
                        .with_message(msg)
                        .finish(Location::Module(id.clone()))
                })
                .map_err(expect_no_verification_errors)?;

        let mut sha3_256 = Sha3_256::new();
        sha3_256.update(&bytes);
        let hash_value: [u8; 32] = sha3_256.finalize().into();

        Ok((Arc::new(module), bytes.len(), hash_value))
    }
}

pub fn into_change_set(
    object_runtime: Rc<RwLock<ObjectRuntime>>,
) -> PartialVMResult<(TxContext, StateChangeSet)> {
    let object_runtime = Rc::try_unwrap(object_runtime).map_err(|_| {
        PartialVMError::new(StatusCode::STORAGE_ERROR)
            .with_message("ObjectRuntime is referenced more than once".to_owned())
    })?;
    let data = object_runtime.into_inner();
    let (tx_context, change_set) = data.into_change_set()?;
    Ok((tx_context, change_set))
}

impl<'r, 'l, S: MoveOSResolver> TypeLayoutLoader for MoveosDataCache<'r, 'l, S> {
    fn get_type_layout(&self, type_tag: &TypeTag) -> PartialVMResult<MoveTypeLayout> {
        let legacy_module_cache =
            Arc::new(self.code_cache.legacy_module_cache.clone()) as Arc<dyn LegacyModuleStorage>;
        let legacy_module_storage = &LegacyModuleStorageAdapter::new(legacy_module_cache);
        self.loader
            .get_type_layout(type_tag, self, legacy_module_storage, &self.code_cache)
            .map_err(|e| e.to_partial())
    }

    fn type_to_type_layout(&self, ty: &Type) -> PartialVMResult<MoveTypeLayout> {
        let legacy_module_cache =
            Arc::new(self.code_cache.legacy_module_cache.clone()) as Arc<dyn LegacyModuleStorage>;
        let legacy_module_storage = &LegacyModuleStorageAdapter::new(legacy_module_cache);
        self.loader
            .type_to_type_layout(ty, legacy_module_storage, &self.code_cache)
    }

    fn type_to_type_tag(&self, ty: &Type) -> PartialVMResult<TypeTag> {
        self.loader.type_to_type_tag(ty, &self.code_cache)
    }
}
