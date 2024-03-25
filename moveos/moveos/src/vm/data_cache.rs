// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_vm_runtime::loader::Loader;

use move_binary_format::errors::{Location, PartialVMError, PartialVMResult, VMResult};
use move_core_types::language_storage::TypeTag;
use move_core_types::{
    account_address::AccountAddress,
    effects::{ChangeSet, Event},
    gas_algebra::NumBytes,
    language_storage::ModuleId,
    value::MoveTypeLayout,
    vm_status::StatusCode,
};
use move_vm_runtime::data_cache::TransactionCache;
use move_vm_types::{
    loaded_data::runtime_types::Type,
    values::{GlobalValue, Struct, Value},
};
use moveos_stdlib::natives::moveos_stdlib::raw_table::{
    serialize, ObjectRuntime, TypeLayoutLoader,
};
use moveos_types::state::KeyState;
use moveos_types::{
    moveos_std::tx_context::TxContext, state::StateChangeSet, state_resolver::MoveOSResolver,
};
use parking_lot::RwLock;
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
    object_runtime: Arc<RwLock<ObjectRuntime>>,
}

impl<'r, 'l, S: MoveOSResolver> MoveosDataCache<'r, 'l, S> {
    /// Create a `MoveosDataCache` with a `RemoteCache` that provides access to data
    /// not updated in the transaction.
    pub fn new(
        resolver: &'r S,
        loader: &'l Loader,
        object_runtime: Arc<RwLock<ObjectRuntime>>,
    ) -> Self {
        MoveosDataCache {
            resolver,
            loader,
            event_data: vec![],
            object_runtime,
        }
    }
}

impl<'r, 'l, S: MoveOSResolver> TransactionCache for MoveosDataCache<'r, 'l, S> {
    /// Make a write set from the updated (dirty, deleted) global resources along with
    /// published modules.
    ///
    /// Gives all proper guarantees on lifetime of global data as well.
    fn into_effects(self, loader: &Loader) -> PartialVMResult<(ChangeSet, Vec<Event>)> {
        let mut events = vec![];
        for (guid, seq_num, ty, ty_layout, val) in self.event_data {
            let ty_tag = loader.type_to_type_tag(&ty)?;
            let blob = val
                .simple_serialize(&ty_layout)
                .ok_or_else(|| PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR))?;
            events.push((guid, seq_num, ty_tag, blob))
        }

        Ok((ChangeSet::new(), events))
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
        _addr: AccountAddress,
        _ty: &Type,
    ) -> PartialVMResult<(&mut GlobalValue, Option<NumBytes>)> {
        unreachable!("Global operations are disabled")
    }

    /// Get the serialized format of a `CompiledModule` given a `ModuleId`.
    fn load_module(&self, module_id: &ModuleId) -> VMResult<Vec<u8>> {
        //if we use object_runtime.write() here, it will cause a deadlock
        //TODO refactor DataCache and ObjectRuntime to avoid this deadlock
        let object_runtime = self.object_runtime.read();

        let result = object_runtime
            .get_loaded_module(module_id)
            .and_then(|module| match module {
                Some(move_module) => {
                    if log::log_enabled!(log::Level::Trace) {
                        log::trace!("Loaded module {:?} from ObjectRuntime", module_id);
                    }
                    Ok(Some(move_module.byte_codes))
                }
                None => self.resolver.get_module(module_id).map_err(|e| {
                    let msg = format!("Unexpected storage error: {:?}", e);
                    PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                        .with_message(msg)
                }),
            });

        match result {
            Ok(Some(code)) => Ok(code),
            Ok(None) => {
                let key_state = KeyState::from_module_id(module_id);
                Err(PartialVMError::new(StatusCode::LINKER_ERROR)
                    .with_message(format!(
                        "Cannot find module {:?}(key:{:?}) in ObjectRuntime and Storage",
                        module_id, key_state,
                    ))
                    .finish(Location::Undefined))
            }
            Err(err) => {
                if log::log_enabled!(log::Level::Debug) {
                    log::warn!("Error loading module {:?}: {:?}", module_id, err);
                }
                Err(err.finish(Location::Undefined))
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

    fn events(&self) -> &Vec<(Vec<u8>, u64, Type, MoveTypeLayout, Value)> {
        &self.event_data
    }
}

pub fn into_change_set(
    object_runtime: Arc<RwLock<ObjectRuntime>>,
) -> PartialVMResult<(TxContext, StateChangeSet)> {
    let object_runtime = Arc::try_unwrap(object_runtime).map_err(|_| {
        PartialVMError::new(StatusCode::STORAGE_ERROR)
            .with_message("ObjectRuntime is referenced more than once".to_owned())
    })?;
    let data = object_runtime.into_inner();
    let (tx_context, change_set) = data.into_change_set()?;
    // let mut changes = BTreeMap::new();
    // for (handle, table) in tables {
    //     let (_, content) = table.into_inner();
    //     let mut entries = BTreeMap::new();
    //     for (key, table_value) in content {
    //         let (value_layout, value_type, op) = match table_value.into_effect() {
    //             Some((value_layout, value_type, op)) => (value_layout, value_type, op),
    //             None => continue,
    //         };
    //         match op {
    //             Op::New(box_val) => {
    //                 let bytes = unbox_and_serialize(&value_layout, box_val)?;
    //                 entries.insert(
    //                     KeyState::new(key.key, key.key_type),
    //                     Op::New(State {
    //                         value_type,
    //                         value: bytes,
    //                     }),
    //                 );
    //             }
    //             Op::Modify(val) => {
    //                 let bytes = unbox_and_serialize(&value_layout, val)?;
    //                 entries.insert(
    //                     KeyState::new(key.key, key.key_type),
    //                     Op::Modify(State {
    //                         value_type,
    //                         value: bytes,
    //                     }),
    //                 );
    //             }
    //             Op::Delete => {
    //                 entries.insert(KeyState::new(key.key, key.key_type), Op::Delete);
    //             }
    //         }
    //     }
    //     if !entries.is_empty() {
    //         changes.insert(handle, TableChange { entries });
    //     }
    // }
    Ok((tx_context, change_set))
}

// Unbox a value of `moveos_std::raw_table::Box<V>` to V and serialize it.
fn unbox_and_serialize(layout: &MoveTypeLayout, box_val: Value) -> PartialVMResult<Vec<u8>> {
    let mut fields = box_val.value_as::<Struct>()?.unpack()?;
    let val = fields.next().ok_or_else(|| {
        PartialVMError::new(StatusCode::VM_EXTENSION_ERROR)
            .with_message("Box<V> should have one field of type V".to_owned())
    })?;
    serialize(layout, &val)
}

impl<'r, 'l, S: MoveOSResolver> TypeLayoutLoader for MoveosDataCache<'r, 'l, S> {
    fn get_type_layout(&self, type_tag: &TypeTag) -> PartialVMResult<MoveTypeLayout> {
        self.loader
            .get_type_layout(type_tag, self)
            .map_err(|e| e.to_partial())
    }

    fn type_to_type_layout(&self, ty: &Type) -> PartialVMResult<MoveTypeLayout> {
        self.loader.type_to_type_layout(ty)
    }

    fn type_to_type_tag(&self, ty: &Type) -> PartialVMResult<TypeTag> {
        self.loader.type_to_type_tag(ty)
    }
}
