// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use move_core_types::{
    account_address::AccountAddress,
    effects::{ChangeSet, Op},
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
};
use moveos_types::move_types::is_table;
use moveos_types::state::StateSet;
use moveos_types::state_resolver::ListState;
use moveos_types::{
    h256::H256,
    move_module::MoveModule,
    state::{MoveStructState, State},
};
use moveos_types::{
    object::{AccountStorage, Object, ObjectID, RawObject, TableInfo},
    storage_context,
};
use moveos_types::{
    state::StateChangeSet,
    state_resolver::{self, module_name_to_key, resource_tag_to_key, StateResolver},
};
use smt::{NodeStore, SMTree, UpdateSet};
use std::collections::BTreeMap;

use crate::state_store::NodeDBStore;

struct AccountStorageTables<NS> {
    pub resources: (Object<TableInfo>, TreeTable<NS>),
    pub modules: (Object<TableInfo>, TreeTable<NS>),
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

    pub fn list(&self, cursor: Option<Vec<u8>>, limit: usize) -> Result<Vec<Option<ListState>>> {
        self.smt.list(cursor, limit)
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
        //We wrap the modules to `MoveModule`
        //For distinguish `vector<u8>` and MoveModule in Move.
        self.put_changes(modules.into_iter().map(|(k, v)| {
            (
                module_name_to_key(k.as_ident_str()),
                v.map(|v| MoveModule::new(v).into()),
            )
        }))
    }

    pub fn put_resources(&self, modules: BTreeMap<StructTag, Op<Vec<u8>>>) -> Result<H256> {
        self.put_changes(modules.into_iter().map(|(k, v)| {
            (
                resource_tag_to_key(&k),
                v.map(|v| State::new(v, TypeTag::Struct(Box::new(k)))),
            )
        }))
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

    pub fn dump(&self) -> Result<Vec<(Vec<u8>, State)>> {
        self.smt.dump()
    }
}

/// StateDB provide state storage and state proof
pub struct StateDBStore {
    pub node_store: NodeDBStore,
    global_table: TreeTable<NodeDBStore>,
}

impl StateDBStore {
    pub fn new(node_store: NodeDBStore) -> Self {
        Self {
            node_store: node_store.clone(),
            global_table: TreeTable::new(node_store),
        }
    }

    pub fn new_with_root(node_store: NodeDBStore, state_root: Option<H256>) -> Self {
        Self {
            node_store: node_store.clone(),
            global_table: TreeTable::new_with_root(node_store, state_root),
        }
    }

    pub fn get(&self, id: ObjectID) -> Result<Option<State>> {
        self.global_table.get(id.to_bytes())
    }

    pub fn list(&self, cursor: Option<Vec<u8>>, limit: usize) -> Result<Vec<Option<ListState>>> {
        self.global_table.list(cursor, limit)
    }

    fn get_as_object<T: MoveStructState>(&self, id: ObjectID) -> Result<Option<Object<T>>> {
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
    ) -> Result<(Object<AccountStorage>, AccountStorageTables<NodeDBStore>)> {
        let account_storage = self
            .get_as_account_storage(account)?
            .unwrap_or_else(|| Object::new_account_storage_object(account));
        let storage_tables = AccountStorageTables {
            resources: self.get_as_table_or_create(account_storage.value.resources)?,
            modules: self.get_as_table_or_create(account_storage.value.modules)?,
        };
        Ok((account_storage, storage_tables))
    }

    fn get_as_table(
        &self,
        id: ObjectID,
    ) -> Result<Option<(Object<TableInfo>, TreeTable<NodeDBStore>)>> {
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
    ) -> Result<(Object<TableInfo>, TreeTable<NodeDBStore>)> {
        Ok(self.get_as_table(id)?.unwrap_or_else(|| {
            let table = TreeTable::new(self.node_store.clone());
            let table_info = TableInfo::new(AccountAddress::new(table.state_root().into()));
            let object = Object::new_table_object(id, table_info);
            (object, table)
        }))
    }

    pub fn get_with_key(&self, id: ObjectID, key: Vec<u8>) -> Result<Option<State>> {
        self.get_as_table(id)
            .and_then(|res| res.map(|(_, table)| table.get(key)).unwrap_or(Ok(None)))
    }

    pub fn list_with_key(
        &self,
        id: ObjectID,
        cursor: Option<Vec<u8>>,
        limit: usize,
    ) -> Result<Vec<Option<ListState>>> {
        self.get_as_table(id).and_then(|res| {
            res.map(|(_, table)| table.list(cursor, limit))
                .unwrap_or(Ok(vec![]))
        })
    }

    pub fn apply_change_set(
        &self,
        change_set: ChangeSet,
        state_change_set: StateChangeSet,
    ) -> Result<H256> {
        let mut changed_objects = UpdateSet::new();
        //TODO
        //We want deprecate the global storage instructions https://github.com/rooch-network/rooch/issues/248
        //So the ChangeSet should be empty, but the module publish still need it
        //We need to figure out a way to make the module publish use raw table's StateChangeSet
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
                changed_objects.put(account_storage.value.resources.to_bytes(), object.into());
            }
            //TODO check if the account_storage and table is changed, if not changed, don't put it
            changed_objects.put(ObjectID::from(account).to_bytes(), account_storage.into())
        }

        for (table_handle, table_change) in state_change_set.changes {
            // handle global object
            if table_handle == storage_context::GLOBAL_OBJECT_STORAGE_HANDLE {
                self.global_table
                    .put_changes(table_change.entries.into_iter())?;
            } else {
                let (mut object, table) = self.get_as_table_or_create(table_handle)?;
                let new_state_root = table.put_changes(table_change.entries.into_iter())?;
                object.value.state_root = AccountAddress::new(new_state_root.into());
                changed_objects.put(table_handle.to_bytes(), object.into());
            }
        }

        for table_handle in state_change_set.removed_tables {
            changed_objects.remove(table_handle.to_bytes());
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
            State::from(account_storage),
        ))?;
        Ok(())
    }

    pub fn resolve_state(&self, handle: &ObjectID, key: &[u8]) -> Result<Option<State>, Error> {
        if handle == &state_resolver::GLOBAL_OBJECT_STORAGE_HANDLE {
            self.global_table.get(key.to_vec())
        } else {
            self.get_with_key(*handle, key.to_vec())
        }
    }

    pub fn resolve_list_state(
        &self,
        handle: &ObjectID,
        cursor: Option<Vec<u8>>,
        limit: usize,
    ) -> Result<Vec<Option<ListState>>, Error> {
        if handle == &state_resolver::GLOBAL_OBJECT_STORAGE_HANDLE {
            self.global_table.list(cursor, limit)
        } else {
            self.list_with_key(*handle, cursor, limit)
        }
    }

    pub fn dump(&self) -> Result<StateSet> {
        let global_states = self.global_table.dump()?;
        let mut state_set = StateSet::default();
        let mut golbal_update_set = UpdateSet::new();
        for (key, state) in global_states.into_iter() {
            // If the state is an Object, and the T's struct_tag of Object<T> is Table
            let struct_tag = state.get_object_struct_tag();
            if struct_tag.is_some() && is_table(&struct_tag.unwrap()) {
                let table_handle = ObjectID::from_bytes(key.as_slice())?;
                let result = self.get_as_table(table_handle)?;
                if result.is_none() {
                    continue;
                }
                let table_states = result.unwrap().1.dump()?;
                let mut update_set = UpdateSet::new();
                for (inner_key, inner_state) in table_states.into_iter() {
                    update_set.put(inner_key, inner_state);
                }
                state_set.state_sets.insert(table_handle, update_set);
            }

            golbal_update_set.put(key, state);
        }
        state_set.state_sets.insert(
            storage_context::GLOBAL_OBJECT_STORAGE_HANDLE,
            golbal_update_set,
        );

        Ok(state_set)
    }
}

impl StateResolver for StateDBStore {
    fn resolve_state(
        &self,
        handle: &ObjectID,
        key: &[u8],
    ) -> std::result::Result<Option<State>, Error> {
        self.resolve_state(handle, key)
    }

    fn resolve_list_state(
        &self,
        handle: &ObjectID,
        cursor: Option<Vec<u8>>,
        limit: usize,
    ) -> std::result::Result<Vec<Option<ListState>>, Error> {
        self.resolve_list_state(handle, cursor, limit)
    }
}
