// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Result, anyhow};
use move_core_types::{
    account_address::AccountAddress,
    effects::{ChangeSet, Op},
    identifier::Identifier,
    language_storage::{ModuleId, StructTag},
    resolver::{ModuleResolver, ResourceResolver},
};
use moveos_stdlib::natives::moveos_stdlib::raw_table::{TableChangeSet, TableResolver, TableHandle};
use moveos_types::{object::{Object, ObjectID, TableInfo, RawObject, AccountStorage}, storage_context};
use serde::de::DeserializeOwned;
use smt::{InMemoryNodeStore, NodeStore, SMTree, UpdateSet};
use std::{collections::BTreeMap};

pub use smt::HashValue;

#[cfg(test)]
mod tests;

/// StateDB query path
/// 1. /account_address/resource_type|module_id
/// 2. /table_handle/key
/// 3. /object_id/child_id
pub struct AccessPath {}

pub trait StateWriter {
    //TODO define batch struct
    fn write_batch(&self, batch: Vec<(AccessPath, Vec<u8>)>) -> Result<()>;
}

pub struct AccountStorageTables<NS>{
    pub resources:(Object<TableInfo>, TreeTable<NS>),
    pub modules: (Object<TableInfo>, TreeTable<NS>),
}

pub struct TreeTable<NS> {
    //TODO move ValueBox to moveos_types
    smt: SMTree<Vec<u8>, Vec<u8>, NS>,
}

impl<NS> TreeTable<NS>
where
    NS: NodeStore,
{
    pub fn new(node_store: NS) -> Self {
        Self::new_with_root(node_store, None)
    }

    pub fn new_with_root(node_store: NS, state_root: Option<HashValue>) -> Self {
        Self {
            smt: SMTree::new(node_store, state_root),
        }
    }

    pub fn get(&self, key: Vec<u8>) -> Result<Option<Vec<u8>>> {
        self.smt.get(key)
    }

    pub fn puts<I>(&self, update_set: I) -> Result<HashValue> where I: Into<UpdateSet<Vec<u8>, Vec<u8>>>{
        self.smt.puts(update_set)
    }

    pub fn state_root(&self) -> HashValue {
        self.smt.root_hash()
    }

    pub fn put_modules(&self, modules: BTreeMap<Identifier, Op<Vec<u8>>>) -> Result<HashValue> {
        self.put_changes(
            modules
                .into_iter()
                .map(|(k, v)| (k.to_string().into_bytes(), v)),
        )
    }

    pub fn put_resources(&self, modules: BTreeMap<StructTag, Op<Vec<u8>>>) -> Result<HashValue> {
        self.put_changes(
            modules
                .into_iter()
                .map(|(k, v)| (k.to_canonical_string().into_bytes(), v)),
        )
    }

    pub fn put_changes<I: IntoIterator<Item = (Vec<u8>, Op<Vec<u8>>)>>(
        &self,
        changes: I,
    ) -> Result<HashValue> {
        let mut update_set = UpdateSet::new();
        for (key, op) in changes {
            match op {
                Op::Modify(value) => {
                    update_set.put(key, value);
                }
                Op::Delete => {
                    update_set.remove(key);
                }
                Op::New(value) => {
                    update_set.put(key, value);
                }
            }
        }
        self.puts(update_set)
    }
}

/// StateDB provide state storage and state proof
pub struct StateDB {
    node_store: InMemoryNodeStore,
    global_table: TreeTable<InMemoryNodeStore>
}

impl StateDB {
    /// Init stateDB with memory store, just for test
    pub fn new_with_memory_store() -> Self {
        let node_store = InMemoryNodeStore::default();
        Self { node_store:node_store.clone(), global_table: TreeTable::new(node_store) }
    }

    pub fn get(&self, id: ObjectID) -> Result<Option<Vec<u8>>> {
        self.global_table.get(id.to_vec())
    }

    fn get_as_object<T: DeserializeOwned>(&self, id: ObjectID) -> Result<Option<Object<T>>>{
        self.get(id)?.map(|bytes|{
            bcs::from_bytes::<Object<T>>(&bytes)
        }).transpose().map_err(Into::into)
    }

    pub fn get_as_raw_object(&self, id: ObjectID) -> Result<Option<RawObject>>{
        self.get(id)?.map(|bytes|{
            bcs::from_bytes::<RawObject>(&bytes)
        }).transpose().map_err(Into::into)
    }

    fn get_as_account_storage(&self, account: AccountAddress) -> Result<Object<AccountStorage>>{
        let object = self.get_as_object::<AccountStorage>(account.into())?.ok_or_else(||anyhow!("Can not find account by address:{}", account))?;
        Ok(object)
    } 

    //TODO should remove this
    // fn get_as_account_storage_with_table(&self, account: AccountAddress) -> Result<(Object<AccountStorage>,AccountStorageTables<InMemoryNodeStore>)>{
    //     let object = self.get_as_account_storage(account)?;
    //     let account_storage_tables = AccountStorageTables{
    //         resources: self.get_as_table_or_create(object.value.resources.into())?,
    //         modules: self.get_as_table_or_create(object.value.modules.into())?,    
    //     };
    //     Ok((object, account_storage_tables))
    // } 

    pub fn get_as_table(
        &self,
        id: ObjectID,
    ) -> Result<Option<(Object<TableInfo>, TreeTable<InMemoryNodeStore>)>> {
        let object = self.get_as_object::<TableInfo>(id)?;
        match object {
            Some(object) => {
                let state_root = object.value.state_root;
                Ok(Some((
                    object,
                    TreeTable::new_with_root(self.node_store.clone(), Some(HashValue::new(state_root.into()))),
                )))
            }
            None => Ok(None),
        }
    }

    fn get_as_table_or_create(
        &self,
        id: ObjectID,
    ) -> Result<(Object<TableInfo>, TreeTable<InMemoryNodeStore>)> {
        Ok(self.get_as_table(id)?.unwrap_or_else(|| {
            let table = TreeTable::new(self.node_store.clone());
            let table_object = TableInfo::new(AccountAddress::new(table.state_root().into()));
            let object = Object::new_table_object(id, table_object);
            (object, table)
        }))
    }

    pub fn get_with_key(&self, id: ObjectID, key: Vec<u8>) -> Result<Option<Vec<u8>>> {
        self.get_as_table(id)
            .and_then(|res| res.map(|(_, table)| table.get(key)).unwrap_or(Ok(None)))
    }

    pub fn apply_change_set(
        &self,
        change_set: ChangeSet,
        table_change_set: TableChangeSet,
    ) -> Result<HashValue> {
        let mut changed_objects = UpdateSet::new();

        for (account, account_change_set) in change_set.into_inner() {
            let account_storage = self.get_as_account_storage(account)?;

            let (modules, resources) = account_change_set.into_inner();
            if !modules.is_empty() {
                let object_id = account_storage.value.modules.into();
                let (mut object, module_table) = self.get_as_table_or_create(object_id)?;
                let new_state_root = module_table.put_modules(modules)?;
                object.value.state_root = AccountAddress::new(new_state_root.into());
                changed_objects.put(object_id.to_vec(), object.to_vec());
            }
            if !resources.is_empty() {
                let object_id = account_storage.value.resources.into();
                let (mut object, resource_table) = self.get_as_table_or_create(object_id)?;
                let new_state_root = resource_table.put_resources(resources)?;
                object.value.state_root = AccountAddress::new(new_state_root.into());
                changed_objects.put(object_id.to_vec(), object.to_vec());
            }
        }

        for (table_handle, table_change) in table_change_set.changes {
            // handle global object
            if table_handle.0 == storage_context::GLOBAL_OBJECT_STORAGE_HANDLE{
                self.global_table.put_changes(table_change.entries.into_iter())?;
            }else{
                let object_id: ObjectID = table_handle.into();
                let (mut object, table) = self.get_as_table_or_create(object_id)?;
                let new_state_root = table.put_changes(table_change.entries.into_iter())?;
                object.value.state_root = AccountAddress::new(new_state_root.into());
                changed_objects.put(object_id.to_vec(), object.to_vec());
            }
        }

        for table_handle in table_change_set.removed_tables {
            let object_id: ObjectID = table_handle.into();
            changed_objects.remove(object_id.to_vec());
        }

        self.global_table.puts(changed_objects)
    }

    // Only the genesis account need to create by this function 
    pub fn create_account_storage(&self, account: AccountAddress) -> Result<HashValue>{
        let resource_table_id = ObjectID::derive_id(account.to_vec(), 0);
        let module_table_id = ObjectID::derive_id(account.to_vec(), 1);
        let object = Object::new_account_storage_object(account, AccountStorage {
            resources: resource_table_id.into(),
            modules: module_table_id.into(),
        });
        self.global_table.puts((object.id.to_vec(), object.to_vec()))
    }

    // pub fn apply_object_change_set(&self, change_set: ObjectChangeSet) -> Result<HashValue> {
    //     let mut changed_objects = UpdateSet::new();
    //     for (object_id, object_info) in change_set.new_objects {
    //         //TODO should serialize at the extension when make change set
    //         let contents = object_info
    //             .value
    //             .simple_serialize(&object_info.value_layout)
    //             .unwrap();
    //         //TODO set version
    //         changed_objects.put(
    //             object_id,
    //             Object::new_move_object(MoveObjectData::new(object_info.value_tag, 1, contents)),
    //         );
    //     }
    //     self.smt.puts(changed_objects)
    // }

    pub fn is_genesis(&self) -> bool {
        self.global_table.smt.is_genesis()
    }
}

impl ResourceResolver for StateDB {
    type Error = anyhow::Error;

    fn get_resource(
        &self,
        address: &AccountAddress,
        tag: &StructTag,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        let account_storage = self.get_as_account_storage(*address)?;
        self.get_with_key(
            account_storage.value.resources.into(),
            tag.to_canonical_string().into_bytes(),
        )
    }
}

impl ModuleResolver for StateDB {
    type Error = anyhow::Error;

    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>, Self::Error> {
        let account_storage = self.get_as_account_storage(*module_id.address())?;
        self.get_with_key(
            account_storage.value.modules.into(),
            module_id.name().to_string().into_bytes(),
        )
    }
}

impl TableResolver for StateDB {
    fn resolve_table_entry(
        &self,
        handle: &TableHandle,
        key: &[u8],
    ) -> std::result::Result<Option<Vec<u8>>, anyhow::Error> {
        self.get_with_key((*handle).into(), key.to_vec())
    }
}

