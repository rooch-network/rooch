// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress,
    effects::{ChangeSet, Op},
    identifier::{IdentStr, Identifier},
    language_storage::{ModuleId, StructTag},
    resolver::{ModuleResolver, ResourceResolver},
};
use moveos_stdlib::natives::moveos_stdlib::raw_table::{
    TableChangeSet, TableHandle, TableResolver,
};
use moveos_types::{h256::H256, state::{State, MoveState}, move_module::MoveModule};
use moveos_types::{
    object::{AccountStorage, NamedTableID, Object, ObjectID, RawObject, TableInfo},
    storage_context,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use smt::{InMemoryNodeStore, NodeStore, SMTree, UpdateSet};
use std::collections::BTreeMap;

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

pub struct AccountStorageTables<NS> {
    pub resources: (Object<TableInfo>, TreeTable<NS>),
    pub modules: (Object<TableInfo>, TreeTable<NS>),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum StateType {
    /// The Move module
    MoveModule,
    /// The Move resource, include Object and Table Resource
    MoveResource(StructTag),
}

pub struct TreeTable<NS> {
    smt: SMTree<Vec<u8>, State, NS>,
}

impl<NS> TreeTable<NS>
where
    NS: NodeStore,
{
    pub fn new(node_store: NS) -> Self {
        Self::new_with_root(node_store, None)
    }

    pub fn new_with_root(node_store: NS, state_root: Option<H256>) -> Self {
        Self {
            smt: SMTree::new(node_store, state_root),
        }
    }

    pub fn get(&self, key: Vec<u8>) -> Result<Option<State>> {
        self.smt.get(key)
    }

    pub fn puts<I>(&self, update_set: I) -> Result<H256>
    where
        I: Into<UpdateSet<Vec<u8>, State>>,
    {
        self.smt.puts(update_set)
    }

    pub fn state_root(&self) -> H256 {
        self.smt.root_hash()
    }

    pub fn put_modules(&self, modules: BTreeMap<Identifier, Op<Vec<u8>>>) -> Result<H256> {
        self.put_changes(
            modules
                .into_iter()
                .map(|(k, v)| (module_name_to_key(k.as_ident_str()), v.map(|v|MoveModule::new(v).into())))
        )
    }

    pub fn put_resources(&self, modules: BTreeMap<StructTag, Op<Vec<u8>>>) -> Result<H256> {
        self.put_changes(modules.into_iter().map(|(k, v)| (tag_to_key(&k), v.map(|v|
        State::new(v, k)
        ))))
    }

    pub fn put_changes<I: IntoIterator<Item = (Vec<u8>, Op<State>)>>(
        &self,
        changes: I,
    ) -> Result<H256> {
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
    global_table: TreeTable<InMemoryNodeStore>,
}

impl StateDB {
    /// Init stateDB with memory store, just for test
    pub fn new_with_memory_store() -> Self {
        let node_store = InMemoryNodeStore::default();
        Self {
            node_store: node_store.clone(),
            global_table: TreeTable::new(node_store),
        }
    }

    pub fn get(&self, id: ObjectID) -> Result<Option<State>> {
        self.global_table.get(id.to_bytes())
    }

    fn get_as_object<T: MoveState>(&self, id: ObjectID) -> Result<Option<Object<T>>> {
        self.get(id)?
            .map(|state| state.as_object::<T>())
            .transpose()
            .map_err(Into::into)
    }

    pub fn get_as_raw_object(&self, id: ObjectID) -> Result<Option<RawObject>> {
        self.get(id)?
            .map(|state| state.as_raw_object())
            .transpose()
            .map_err(Into::into)
    }

    fn get_as_account_storage(
        &self,
        account: AccountAddress,
    ) -> Result<Option<Object<AccountStorage>>> {
        self.get_as_object::<AccountStorage>(account.into())
    }

    fn get_as_account_storage_or_create(
        &self,
        account: AccountAddress,
    ) -> Result<(
        Object<AccountStorage>,
        AccountStorageTables<InMemoryNodeStore>,
    )> {
        let account_storage = self
            .get_as_account_storage(account)?
            .unwrap_or_else(|| Object::new_account_storage_object(account));
        let storage_tables = AccountStorageTables {
            resources: self.get_as_table_or_create(account_storage.value.resources)?,
            modules: self.get_as_table_or_create(account_storage.value.modules)?,
        };
        Ok((account_storage, storage_tables))
    }

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
                    TreeTable::new_with_root(
                        self.node_store.clone(),
                        Some(H256(state_root.into())),
                    ),
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
            let table_info = TableInfo::new(AccountAddress::new(table.state_root().into()));
            let object = Object::new_table_object(id, table_info);
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
    ) -> Result<H256> {
        let mut changed_objects = UpdateSet::new();

        for (account, account_change_set) in change_set.into_inner() {
            let (account_storage, storage_tables) =
                self.get_as_account_storage_or_create(account)?;

            let (modules, resources) = account_change_set.into_inner();
            if !modules.is_empty() {
                let (mut object, module_table) = storage_tables.modules;
                let new_state_root = module_table.put_modules(modules)?;
                object.value.state_root = AccountAddress::new(new_state_root.into());
                changed_objects.put(account_storage.value.modules.to_bytes(), object.into());
            }
            if !resources.is_empty() {
                let (mut object, resource_table) = storage_tables.resources;
                let new_state_root = resource_table.put_resources(resources)?;
                object.value.state_root = AccountAddress::new(new_state_root.into());
                changed_objects.put(
                    account_storage.value.resources.to_bytes(),
                    object.into(),
                );
            }
            //TODO check if the account_storage and table is changed, if not changed, don't put it
            changed_objects.put(
                ObjectID::from(account).to_bytes(),
                account_storage.into(),
            )
        }

        for (table_handle, table_change) in table_change_set.changes {
            // handle global object
            if table_handle.0 == storage_context::GLOBAL_OBJECT_STORAGE_HANDLE {
                self.global_table
                    .put_changes(table_change.entries.into_iter())?;
            } else {
                let object_id: ObjectID = table_handle.into();
                let (mut object, table) = self.get_as_table_or_create(object_id)?;
                let new_state_root = table.put_changes(table_change.entries.into_iter())?;
                object.value.state_root = AccountAddress::new(new_state_root.into());
                changed_objects.put(object_id.to_bytes(), object.into());
            }
        }

        for table_handle in table_change_set.removed_tables {
            let object_id: ObjectID = table_handle.into();
            changed_objects.remove(object_id.to_bytes());
        }

        self.global_table.puts(changed_objects)
    }

    pub fn is_genesis(&self) -> bool {
        self.global_table.smt.is_genesis()
    }

    //Only for unit test and integration test runner
    pub fn create_account_storage(&self, account: AccountAddress) -> Result<()> {
        let account_storage = Object::new_account_storage_object(account);
        self.global_table.puts((
            ObjectID::from(account).to_bytes(),
            account_storage.to_bytes(),
        ))?;
        Ok(())
    }
}

impl ResourceResolver for StateDB {
    type Error = anyhow::Error;

    fn get_resource(
        &self,
        address: &AccountAddress,
        tag: &StructTag,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        let resource_table_id = NamedTableID::Resource(*address).to_object_id();
        let key = tag_to_key(tag);
        let resource = self.get_with_key(resource_table_id, key)?;
        // We do not need to unbox value at here, because the resource must be a struct,
        // ValueBox<T> 's bcs serialized format is the same as T's bcs serialized format.
        //resource.map(|v|unbox_value(v.as_slice())).transpose()
        Ok(resource)
    }
}

impl ModuleResolver for StateDB {
    type Error = anyhow::Error;

    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>, Self::Error> {
        let module_table_id = NamedTableID::Module(*module_id.address()).to_object_id();
        let key = module_name_to_key(module_id.name());
        let module = self.get_with_key(module_table_id, key)?;
        module.map(|v| unbox_value(v.as_slice())).transpose()
    }
}

fn tag_to_key(tag: &StructTag) -> Vec<u8> {
    // The key is bcs serialize format string, not String::into_bytes.
    bcs::to_bytes(&tag.to_canonical_string()).expect("bcs to_bytes String must success.")
}

fn module_name_to_key(name: &IdentStr) -> Vec<u8> {
    // The key is bcs serialize format string, not String::into_bytes.
    bcs::to_bytes(&name.to_string()).expect("bcs to_bytes String must success.")
}

// Wrap value to a Box, because the table deserialize value to a Box struct
// see moveos_std::raw_table::Box
fn box_value<T: Serialize>(value: T) -> Vec<u8> {
    let value_box = ValueBox { value };
    bcs::to_bytes(&value_box).expect("bcs to_bytes ValueBox must success.")
}

// Unwrap value from a Box, because the table deserialize value to a Box struct
// If we get value from Table API, like raw_table::borrow, the raw_table native auto unbox the value.
// But if we get value from ModuleResolver or ResourceResolver, we need to unbox the value by ourselves.
fn unbox_value<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    let value_box: ValueBox<T> = bcs::from_bytes(bytes)?;
    Ok(value_box.value)
}

impl TableResolver for StateDB {
    fn resolve_table_entry(
        &self,
        handle: &TableHandle,
        key: &[u8],
    ) -> std::result::Result<Option<Vec<u8>>, anyhow::Error> {
        if handle.0 == storage_context::GLOBAL_OBJECT_STORAGE_HANDLE {
            self.global_table.get(key.to_vec())
        } else {
            self.get_with_key((*handle).into(), key.to_vec())
        }
    }
}
