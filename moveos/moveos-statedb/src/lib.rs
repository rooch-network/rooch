// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress,
    effects::{ChangeSet, Op},
    identifier::Identifier,
    language_storage::{ModuleId, StructTag},
    resolver::{ModuleResolver, ResourceResolver},
    value::MoveTypeLayout,
};
use move_table_extension::{TableChangeSet, TableResolver};
use moveos_stdlib::natives::moveos_stdlib::object_extension::{ObjectChangeSet, ObjectResolver};
use moveos_types::object::{MoveObject, NamedTableID, Object, ObjectID, TableObject};
use smt::{InMemoryNodeStore, NodeStore, SMTree, UpdateSet};
use std::collections::BTreeMap;

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

pub struct TreeTable<NS> {
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

    pub fn puts(&self, update_set: UpdateSet<Vec<u8>, Vec<u8>>) -> Result<HashValue> {
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
    smt: SMTree<ObjectID, Object, InMemoryNodeStore>,
}

impl StateDB {
    /// Init stateDB with memory store, just for test
    pub fn new_with_memory_store() -> Self {
        let node_store = InMemoryNodeStore::default();
        let smt = SMTree::new(node_store.clone(), None);
        Self { node_store, smt }
    }

    pub fn get(&self, id: ObjectID) -> Result<Option<Object>> {
        self.smt.get(id)
    }

    pub fn get_as_table(
        &self,
        id: ObjectID,
    ) -> Result<Option<(Object, TreeTable<InMemoryNodeStore>)>> {
        let object = self.get(id)?;
        match object {
            Some(object) => {
                let state_root = object.data.as_table_object()?.state_root;
                Ok(Some((
                    object,
                    TreeTable::new_with_root(self.node_store.clone(), Some(state_root)),
                )))
            }
            None => Ok(None),
        }
    }

    fn get_as_table_or_create(
        &self,
        id: ObjectID,
    ) -> Result<(Object, TreeTable<InMemoryNodeStore>)> {
        Ok(self.get_as_table(id)?.unwrap_or_else(|| {
            let table = TreeTable::new(self.node_store.clone());
            let table_object = TableObject::new(table.state_root(), 0);
            let object = Object::new_table_object(table_object);
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
            let (modules, resources) = account_change_set.into_inner();
            if !modules.is_empty() {
                let object_id = NamedTableID::Module(account).to_object_id();
                let (mut object, module_table) = self.get_as_table_or_create(object_id)?;
                let new_state_root = module_table.put_modules(modules)?;
                let table_object = object.data.as_table_object_mut()?;
                table_object.state_root = new_state_root;
                changed_objects.put(object_id, object);
            }
            if !resources.is_empty() {
                let object_id = NamedTableID::Resource(account).to_object_id();
                let (mut object, resource_table) = self.get_as_table_or_create(object_id)?;
                let new_state_root = resource_table.put_resources(resources)?;
                let table_object = object.data.as_table_object_mut()?;
                table_object.state_root = new_state_root;
                changed_objects.put(object_id, object);
            }
        }

        for (table_handle, table_change) in table_change_set.changes {
            let object_id: ObjectID = table_handle.into();
            let (mut object, table) = self.get_as_table_or_create(object_id)?;
            let new_state_root = table.put_changes(table_change.entries.into_iter())?;
            let table_object = object.data.as_table_object_mut()?;
            table_object.state_root = new_state_root;
            changed_objects.put(object_id, object);
        }

        for table_handle in table_change_set.removed_tables {
            let object_id: ObjectID = table_handle.into();
            changed_objects.remove(object_id);
        }

        self.smt.puts(changed_objects)
    }

    pub fn apply_object_change_set(&self, change_set: ObjectChangeSet) -> Result<HashValue> {
        let mut changed_objects = UpdateSet::new();
        for (object_id, object_info) in change_set.new_objects {
            //TODO should serialize at the extension when make change set
            let contents = object_info
                .value
                .simple_serialize(&MoveTypeLayout::Struct(object_info.layout))
                .unwrap();
            //TODO set version
            changed_objects.put(
                object_id,
                Object::new_move_object(MoveObject::new(object_info.tag, 1, contents)),
            );
        }
        self.smt.puts(changed_objects)
    }

    pub fn is_genesis(&self) -> bool {
        self.smt.is_genesis()
    }
}

impl ResourceResolver for StateDB {
    type Error = anyhow::Error;

    fn get_resource(
        &self,
        address: &AccountAddress,
        tag: &StructTag,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        self.get_with_key(
            NamedTableID::Resource(*address).to_object_id(),
            tag.to_canonical_string().into_bytes(),
        )
    }
}

impl ModuleResolver for StateDB {
    type Error = anyhow::Error;

    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>, Self::Error> {
        self.get_with_key(
            NamedTableID::Module(*module_id.address()).to_object_id(),
            module_id.name().to_string().into_bytes(),
        )
    }
}

impl TableResolver for StateDB {
    fn resolve_table_entry(
        &self,
        handle: &move_table_extension::TableHandle,
        key: &[u8],
    ) -> std::result::Result<Option<Vec<u8>>, anyhow::Error> {
        self.get_with_key((*handle).into(), key.to_vec())
    }
}

impl ObjectResolver for StateDB {
    fn resolve_object(&self, id: ObjectID) -> Result<Option<Object>> {
        self.get(id)
    }
}
