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
    identifier::Identifier,
    language_storage::ModuleId,
    resolver::MoveResolver,
    value::MoveTypeLayout,
    vm_status::StatusCode,
};
use move_vm_types::{
    data_store::{DataStore, TransactionCache},
    loaded_data::runtime_types::Type,
    values::{GlobalValue, Struct, Value},
};
use moveos_stdlib::natives::moveos_stdlib::raw_table::{serialize, Table, TableData};
use moveos_types::{
    move_module::MoveModule,
    move_string::MoveString,
    state::{MoveStructState, State, StateChangeSet, TableChange},
    state_resolver::{MoveOSResolver, StateResolver},
};
use parking_lot::Mutex;
use std::collections::btree_map::BTreeMap;
use std::sync::Arc;

use anyhow;
use move_core_types::language_storage::{StructTag, TypeTag};
use moveos_types::object::{NamedTableID, ObjectID};
use moveos_types::state::MoveStructType;
use serde_json;
use std::str::FromStr;

pub struct AccountDataCache {
    module_map: BTreeMap<Identifier, (Vec<u8>, bool)>,
}

impl AccountDataCache {
    fn new() -> Self {
        Self {
            module_map: BTreeMap::new(),
        }
    }
}

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
    account_map: BTreeMap<AccountAddress, AccountDataCache>,
    event_data: Vec<(Vec<u8>, u64, Type, MoveTypeLayout, Value)>,
    table_data: Arc<Mutex<TableData>>,
}

impl<'r, 'l, S: MoveOSResolver> MoveosDataCache<'r, 'l, S> {
    /// Create a `MoveosDataCache` with a `RemoteCache` that provides access to data
    /// not updated in the transaction.
    pub fn new(resolver: &'r S, loader: &'l Loader, table_data: Arc<Mutex<TableData>>) -> Self {
        MoveosDataCache {
            resolver,
            loader,
            account_map: BTreeMap::new(),
            event_data: vec![],
            table_data,
        }
    }

    fn get_mut_or_insert_with<'a, K, V, F>(map: &'a mut BTreeMap<K, V>, k: &K, gen: F) -> &'a mut V
    where
        F: FnOnce() -> (K, V),
        K: Ord,
    {
        if !map.contains_key(k) {
            let (k, v) = gen();
            map.insert(k, v);
        }
        map.get_mut(k).unwrap()
    }
}

impl<'r, 'l, S: MoveOSResolver> TransactionCache for MoveosDataCache<'r, 'l, S> {
    /// Make a write set from the updated (dirty, deleted) global resources along with
    /// published modules.
    ///
    /// Gives all proper guarantees on lifetime of global data as well.
    fn into_effects(self) -> PartialVMResult<(ChangeSet, Vec<Event>)> {
        let mut change_set = ChangeSet::new();
        for (addr, account_data_cache) in self.account_map.into_iter() {
            let mut modules = BTreeMap::new();
            for (module_name, (module_blob, is_republishing)) in account_data_cache.module_map {
                let op = if is_republishing {
                    Op::Modify(module_blob)
                } else {
                    Op::New(module_blob)
                };
                modules.insert(module_name, op);
            }

            // No resources updated in TransactionDataCache as global operations are disabled.
            let resources = BTreeMap::new();
            if !modules.is_empty() || !resources.is_empty() {
                change_set
                    .add_account_changeset(
                        addr,
                        AccountChangeSet::from_modules_resources(modules, resources),
                    )
                    .expect("accounts should be unique");
            }
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
        let total_mutated_accounts: u64 = 1;

        // No accounts mutated in global operations are disabled.
        total_mutated_accounts
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

    fn load_module(&self, module_id: &ModuleId) -> VMResult<Vec<u8>> {
        if let Some(account_cache) = self.account_map.get(module_id.address()) {
            if let Some((blob, _is_republishing)) = account_cache.module_map.get(module_id.name()) {
                return Ok(blob.clone());
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
        let account_cache =
            Self::get_mut_or_insert_with(&mut self.account_map, module_id.address(), || {
                (*module_id.address(), AccountDataCache::new())
            });

        account_cache
            .module_map
            .insert(module_id.name().to_owned(), (blob, is_republishing));

        Ok(())
    }

    fn exists_module(&self, module_id: &ModuleId) -> VMResult<bool> {
        if let Some(account_cache) = self.account_map.get(module_id.address()) {
            if account_cache.module_map.contains_key(module_id.name()) {
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

pub fn into_change_set(table_data: Arc<Mutex<TableData>>) -> PartialVMResult<StateChangeSet> {
    let table_data = Arc::try_unwrap(table_data).map_err(|e| {
        PartialVMError::new(StatusCode::STORAGE_ERROR)
            .with_message("TableData is referenced more than once".to_owned())
    })?;
    let data = table_data.into_inner();
    let (new_tables, removed_tables, tables) = data.into_inner();
    let mut changes = BTreeMap::new();
    for (handle, table) in tables {
        let (_, _, content) = table.into_inner();
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
            changes.insert(handle, TableChange { entries });
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
