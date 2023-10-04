// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_vm_runtime::loader::Loader;

use move_binary_format::errors::{Location, PartialVMError, PartialVMResult, VMResult};
use move_core_types::{
    account_address::AccountAddress,
    effects::{AccountChangeSet, ChangeSet, Event, Op},
    gas_algebra::NumBytes,
    language_storage::ModuleId,
    value::MoveTypeLayout,
    vm_status::StatusCode,
};
use move_vm_types::{
    data_store::{DataStore, TransactionCache},
    loaded_data::runtime_types::Type,
    values::{GlobalValue, Reference, Struct, Value},
};
use moveos_stdlib::natives::moveos_stdlib::raw_table::{serialize, TableData, TableRuntimeValue};
use moveos_types::{
    move_module::MoveModule,
    move_string::MoveString,
    state::{MoveStructState, State, StateChangeSet, TableChange},
    state_resolver::{module_name_to_key, MoveOSResolver},
};
use parking_lot::RwLock;
use std::collections::{btree_map::BTreeMap, BTreeSet};
use std::sync::Arc;

use move_core_types::language_storage::TypeTag;
use moveos_types::object::NamedTableID;
use moveos_types::state::MoveStructType;

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
    table_data: Arc<RwLock<TableData>>,
    accounts: BTreeSet<AccountAddress>,
}

impl<'r, 'l, S: MoveOSResolver> MoveosDataCache<'r, 'l, S> {
    /// Create a `MoveosDataCache` with a `RemoteCache` that provides access to data
    /// not updated in the transaction.
    pub fn new(resolver: &'r S, loader: &'l Loader, table_data: Arc<RwLock<TableData>>) -> Self {
        MoveosDataCache {
            resolver,
            loader,
            event_data: vec![],
            table_data,
            accounts: BTreeSet::new(),
        }
    }

    /// Returns the key and value type tag of Rooch module table.
    fn module_table_typetag() -> (TypeTag, TypeTag) {
        // Key type: std::string::String
        let key_typetag = TypeTag::Struct(Box::new(MoveString::struct_tag()));

        // value type: moveos_std::move_module::MoveModule
        let value_typetag = TypeTag::Struct(Box::new(MoveModule::struct_tag()));
        (key_typetag, value_typetag)
    }

    fn module_key_bytes(&self, module_id: &ModuleId) -> VMResult<Vec<u8>> {
        let key = module_name_to_key(module_id.name());
        Ok(key)
    }
}

impl<'r, 'l, S: MoveOSResolver> TransactionCache for MoveosDataCache<'r, 'l, S> {
    /// Make a write set from the updated (dirty, deleted) global resources along with
    /// published modules.
    ///
    /// Gives all proper guarantees on lifetime of global data as well.
    fn into_effects(self) -> PartialVMResult<(ChangeSet, Vec<Event>)> {
        let mut change_set = ChangeSet::new();
        // The accounts are just used for initializing account storage.
        // No modules and resources added here.
        for addr in self.accounts {
            change_set
                .add_account_changeset(
                    addr,
                    AccountChangeSet::from_modules_resources(BTreeMap::new(), BTreeMap::new()),
                )
                .expect("accounts should be unique");
        }

        let mut events = vec![];
        for (guid, seq_num, ty, ty_layout, val) in self.event_data {
            let ty_tag = self.loader.type_to_type_tag(&ty)?;
            let blob = val
                .simple_serialize(&ty_layout)
                .ok_or_else(|| PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR))?;
            events.push((guid, seq_num, ty_tag, blob))
        }

        Ok((change_set, events))
    }

    fn num_mutated_accounts(&self, _sender: &AccountAddress) -> u64 {
        // The sender's account will always be mutated.
        // No accounts mutated in global operations are disabled.
        self.accounts.len() as u64
    }
}

// `DataStore` implementation for the `MoveosDataCache`
impl<'r, 'l, S: MoveOSResolver> DataStore for MoveosDataCache<'r, 'l, S> {
    // Retrieve data from the local cache or loads it from the resolver cache into the local cache.
    // All operations on the global data are based on this API and they all load the data
    // into the cache.
    /// In Rooch, all global operations are disable, so this function is never called.
    fn load_resource(
        &mut self,
        _addr: AccountAddress,
        _ty: &Type,
    ) -> PartialVMResult<(&mut GlobalValue, Option<Option<NumBytes>>)> {
        unreachable!("Global operations are disabled")
    }

    /// Get the serialized format of a `CompiledModule` given a `ModuleId`.
    fn load_module(&self, module_id: &ModuleId) -> VMResult<Vec<u8>> {
        let table_data = self.table_data.read();
        let sender = module_id.address();
        let table_handle = NamedTableID::Module(*sender).to_object_id();
        let (_, value_type) = Self::module_table_typetag();
        // TODO: check or ensure the module table exists.
        if table_data.exist_table(&table_handle) {
            let table = table_data
                .borrow_table(&table_handle)
                .map_err(|e| e.finish(Location::Undefined))?;

            let key_bytes = self.module_key_bytes(module_id)?;
            if let Some(global_value) = table.get_global_value(&key_bytes) {
                let byte_codes = load_module_from_table_runtime_value(global_value, value_type)
                    .map_err(|e| e.finish(Location::Undefined))?;
                return Ok(byte_codes);
            }
        }

        match self.resolver.get_module(module_id) {
            Ok(Some(bytes)) => Ok(bytes),
            Ok(None) => Err(PartialVMError::new(StatusCode::LINKER_ERROR)
                .with_message(format!("Cannot find {:?} in data cache", module_id))
                .finish(Location::Undefined)),
            Err(err) => {
                let msg = format!("Unexpected storage error: {:?}", err);
                Err(
                    PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                        .with_message(msg)
                        .finish(Location::Undefined),
                )
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
        let sender = module_id.address();
        let table_handle = NamedTableID::Module(*sender).to_object_id();

        // Key type: std::string::String
        // value type: moveos_std::move_module::MoveModule
        let (_, value_type) = Self::module_table_typetag();

        let key_layout = MoveTypeLayout::Struct(MoveString::struct_layout());
        let mut table_data = self.table_data.write();
        // TODO: check or ensure the module table exists.
        let table = table_data
            .get_or_create_table_with_key_layout(table_handle, key_layout)
            .map_err(|e| e.finish(Location::Module(module_id.clone())))?;

        let key_bytes = self.module_key_bytes(module_id)?;
        let (tv, _) = table
            .get_or_create_global_value_with_layout_fn(self.resolver, key_bytes, |t| {
                self.loader.get_type_layout(t, self).map_err(|e| {
                    PartialVMError::new(StatusCode::STORAGE_ERROR).with_message(e.to_string())
                })
            })
            .map_err(|e| e.finish(Location::Module(module_id.clone())))?;
        let module_layout = MoveTypeLayout::Struct(MoveModule::struct_layout());

        let byte_codes = Value::vector_u8(blob);
        let module_value = Value::struct_(Struct::pack(vec![byte_codes]));
        // wrap with moveos_std::raw_table::Box
        let box_value = Value::struct_(Struct::pack(vec![module_value]));
        if is_republishing {
            let _old_value = tv
                .move_from(value_type.clone())
                .map_err(|e: PartialVMError| e.finish(Location::Module(module_id.clone())))?;
        }
        match tv.move_to(box_value, module_layout, value_type) {
            Ok(_) => {
                self.accounts.insert(*sender);
                Ok(())
            }
            Err((err, _)) => Err(err.finish(Location::Module(module_id.clone()))),
        }
    }

    /// Check if this module exists.
    fn exists_module(&self, module_id: &ModuleId) -> VMResult<bool> {
        let table_data = self.table_data.read();
        let sender = module_id.address();
        let table_handle = NamedTableID::Module(*sender).to_object_id();
        if table_data.exist_table(&table_handle) {
            let table = table_data
                .borrow_table(&table_handle)
                .map_err(|e| e.finish(Location::Undefined))?;

            let key_bytes = self.module_key_bytes(module_id)?;
            if table.contains_key(&key_bytes) {
                return Ok(true);
            }
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
        guid: Vec<u8>,
        seq_num: u64,
        ty: Type,
        val: Value,
    ) -> PartialVMResult<()> {
        let ty_layout = self.loader.type_to_type_layout(&ty)?;
        self.event_data.push((guid, seq_num, ty, ty_layout, val));
        Ok(())
    }

    fn events(&self) -> &Vec<(Vec<u8>, u64, Type, MoveTypeLayout, Value)> {
        &self.event_data
    }
}

pub fn into_change_set(table_data: Arc<RwLock<TableData>>) -> PartialVMResult<StateChangeSet> {
    let table_data = Arc::try_unwrap(table_data).map_err(|_| {
        PartialVMError::new(StatusCode::STORAGE_ERROR)
            .with_message("TableData is referenced more than once".to_owned())
    })?;
    let data = table_data.into_inner();
    let (new_tables, removed_tables, tables) = data.into_inner();
    let mut changes = BTreeMap::new();
    for (handle, table) in tables {
        let (_, _, content, size_increment) = table.into_inner();
        let mut entries = BTreeMap::new();
        for (key, table_value) in content {
            let (value_layout, value_type, op) = match table_value.into_effect() {
                Some((value_layout, value_type, op)) => (value_layout, value_type, op),
                None => continue,
            };
            match op {
                Op::New(box_val) => {
                    let bytes = unbox_and_serialize(&value_layout, box_val)?;
                    entries.insert(
                        key,
                        Op::New(State {
                            value_type,
                            value: bytes,
                        }),
                    );
                }
                Op::Modify(val) => {
                    let bytes = unbox_and_serialize(&value_layout, val)?;
                    entries.insert(
                        key,
                        Op::Modify(State {
                            value_type,
                            value: bytes,
                        }),
                    );
                }
                Op::Delete => {
                    entries.insert(key, Op::Delete);
                }
            }
        }
        if !entries.is_empty() {
            changes.insert(
                handle,
                TableChange {
                    entries,
                    size_increment,
                },
            );
        } else {
            debug_assert!(size_increment == 0);
        }
    }
    Ok(StateChangeSet {
        new_tables,
        removed_tables,
        changes,
    })
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

// load module bytes stored in `moveos_std::raw_table::Box<moveos_std::move_module::MoveModule>`
fn load_module_from_table_runtime_value(
    global_value: &TableRuntimeValue,
    value_type: TypeTag,
) -> PartialVMResult<Vec<u8>> {
    let blob = global_value.borrow_global(value_type)?;
    let box_value = blob
        .value_as::<Reference>()?
        .read_ref()?
        .value_as::<Struct>()?;

    let mut fields = box_value.unpack()?.collect::<Vec<Value>>();
    debug_assert!(fields.len() == 1, "Fields of Box struct must be 1");
    let module_value = fields.pop().unwrap();
    let mut module_fields = module_value
        .value_as::<Struct>()?
        .unpack()?
        .collect::<Vec<Value>>();
    debug_assert!(
        module_fields.len() == 1,
        "Fields of Module struct must be 1, actual: {}",
        module_fields.len()
    );
    let module = module_fields.pop().unwrap();

    let byte_codes = module.value_as::<Vec<u8>>()?;
    Ok(byte_codes)
}
