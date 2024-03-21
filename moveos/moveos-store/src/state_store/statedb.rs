// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::state_store::NodeDBStore;
use anyhow::{Error, Result};
use move_core_types::effects::Op;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::moveos_std::object::RootObjectEntity;
use moveos_types::moveos_std::object::{ObjectEntity, RawObject};
use moveos_types::state::KeyState;
use moveos_types::state_resolver::StateKV;
use moveos_types::{h256::H256, state::State};
use moveos_types::{state::StateChangeSet, state_resolver::StateResolver};
use smt::{SMTIterator, SMTree, UpdateSet};
use std::collections::BTreeMap;

/// ObjectEntity with fields State Tree
#[derive(Clone)]
pub struct TreeObject {
    entity: RawObject,
    smt: SMTree<KeyState, State, NodeDBStore>,
}

impl TreeObject {
    pub fn new(node_store: NodeDBStore, entity: RawObject) -> Self {
        let state_root = Some(entity.state_root());
        Self {
            entity,
            smt: SMTree::new(node_store, state_root),
        }
    }

    pub fn get_field(&self, key: KeyState) -> Result<Option<State>> {
        if self.smt.is_genesis() {
            return Ok(None);
        }
        self.smt.get(key.clone())
    }

    pub fn get_field_as_object(&self, id: &ObjectID) -> Result<Option<RawObject>> {
        self.get_field(id.to_key())?
            .map(|state| state.as_raw_object())
            .transpose()
            .map_err(Into::into)
    }

    pub fn list_fields(&self, cursor: Option<KeyState>, limit: usize) -> Result<Vec<StateKV>> {
        self.smt.list(cursor, limit)
    }

    pub fn update_fields<I>(&mut self, update_set: I) -> Result<H256>
    where
        I: Into<UpdateSet<KeyState, State>>,
    {
        let state_root = self.smt.puts(update_set)?;
        self.entity.update_state_root(state_root);
        Ok(state_root)
    }

    pub fn state_root(&self) -> H256 {
        let state_root = self.smt.root_hash();
        if cfg!(debug_assertions) {
            debug_assert!(
                state_root == self.entity.state_root(),
                "state root not match, object state_root: {}, smt state_root: {}",
                self.entity.state_root(),
                state_root
            );
        }
        state_root
    }

    pub fn is_genesis(&self) -> bool {
        let is_genesis = self.smt.is_genesis();
        if cfg!(debug_assertions) {
            let state_root = self.smt.root_hash();
            debug_assert!(
                state_root == self.entity.state_root(),
                "is_genesis not match, object state_root: {}, size: {}, smt state_root: {}",
                self.entity.state_root,
                self.entity.size,
                self.smt.root_hash()
            );
        }
        is_genesis
    }

    pub fn put_changes<I: IntoIterator<Item = (KeyState, Op<State>)>>(
        &mut self,
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
        self.update_fields(update_set)
    }

    pub fn dump(&self) -> Result<Vec<(KeyState, State)>> {
        self.smt.dump()
    }

    pub fn iter(&self) -> Result<SMTIterator<KeyState, State, NodeDBStore>> {
        self.smt.iter(None)
    }
}

/// StateDB provide state storage and state proof
#[derive(Clone)]
pub struct StateDBStore {
    pub node_store: NodeDBStore,
    root_object: TreeObject,
}

impl StateDBStore {
    pub fn new(node_store: NodeDBStore) -> Self {
        Self::new_with_root(node_store, ObjectEntity::genesis_root_object())
    }

    pub fn new_with_root(node_store: NodeDBStore, root: RootObjectEntity) -> Self {
        Self {
            node_store: node_store.clone(),
            root_object: TreeObject::new(node_store, root.to_raw()),
        }
    }

    pub fn get(&self, id: &ObjectID) -> Result<Option<State>> {
        self.root_object.get_field(id.to_key())
    }

    pub fn list(&self, cursor: Option<KeyState>, limit: usize) -> Result<Vec<StateKV>> {
        self.root_object.list_fields(cursor, limit)
    }

    fn get_object(&self, id: &ObjectID) -> Result<Option<TreeObject>> {
        Ok(self
            .root_object
            .get_field_as_object(id)?
            .map(|obj| TreeObject::new(self.node_store.clone(), obj)))
    }

    pub fn get_field(&self, id: &ObjectID, key: KeyState) -> Result<Option<State>> {
        self.get_object(id)
            .and_then(|res| res.map(|obj| obj.get_field(key)).unwrap_or(Ok(None)))
    }

    pub fn list_fields(
        &self,
        id: &ObjectID,
        cursor: Option<KeyState>,
        limit: usize,
    ) -> Result<Vec<StateKV>> {
        let obj = self
            .get_object(id)?
            .ok_or_else(|| anyhow::format_err!("Object with id {} not found", id))?;
        obj.list_fields(cursor, limit)
    }

    pub fn apply_change_set(
        &mut self,
        mut state_change_set: StateChangeSet,
    ) -> Result<(H256, u64)> {
        let mut changed_objects = BTreeMap::new();

        let mut update_set = UpdateSet::new();

        let global_change = state_change_set.changes.remove(&ObjectID::root());
        let mut global_size = self.root_object.entity.size;
        if let Some(global_change) = global_change {
            for change in global_change.entries {
                match change {
                    (key, Op::New(state)) => {
                        changed_objects.insert(
                            key.clone(),
                            TreeObject::new(self.node_store.clone(), state.as_raw_object()?),
                        );
                        global_size += 1;
                    }
                    (key, Op::Modify(state)) => {
                        changed_objects.insert(
                            key,
                            TreeObject::new(self.node_store.clone(), state.as_raw_object()?),
                        );
                    }
                    (key, Op::Delete) => {
                        update_set.remove(key);
                        global_size -= 1;
                    }
                }
            }
        }
        for (object_id, object_change) in state_change_set.changes {
            let key = object_id.to_key();
            let mut obj = match changed_objects.remove(&key) {
                Some(obj) => obj,
                None => self
                    .get_object(&object_id)?
                    .ok_or_else(|| anyhow::format_err!("Object with id {} not found", object_id))?,
            };

            obj.put_changes(object_change.entries.into_iter())?;
            update_set.put(key, obj.entity.into_state())
        }

        for table_handle in state_change_set.removed_tables {
            update_set.remove(table_handle.to_key());
        }

        for (key, value) in changed_objects {
            update_set.put(key, value.entity.into_state());
        }

        let update_set_size = update_set.len();
        let state_root = self.root_object.update_fields(update_set)?;
        if log::log_enabled!(log::Level::Debug) {
            log::debug!("apply_change_set state_root: {:?}, update_set_size: {}, pre_global_size: {}, new_global_size: {}", state_root, update_set_size, self.root_object.entity.size, global_size);
        }
        self.root_object.entity.size = global_size;
        Ok((state_root, global_size))
    }

    pub fn is_genesis(&self) -> bool {
        self.root_object.is_genesis()
    }

    pub fn resolve_state(&self, handle: &ObjectID, key: &KeyState) -> Result<Option<State>, Error> {
        if handle == &ObjectID::root() {
            //TODO provide a better way to get global object
            if key == &ObjectID::root().to_key() {
                return Ok(Some(self.root_object.entity.into_state()));
            }
            self.root_object.get_field(key.clone())
        } else {
            self.get_field(handle, key.clone())
        }
    }

    pub fn resolve_list_state(
        &self,
        handle: &ObjectID,
        cursor: Option<KeyState>,
        limit: usize,
    ) -> Result<Vec<StateKV>, Error> {
        if handle == &ObjectID::root() {
            self.root_object.list_fields(cursor, limit)
        } else {
            self.list_fields(handle, cursor, limit)
        }
    }

    //TODO support dump and dump_iter and apply

    // rebuild statedb via TableStateSet from dump
    // pub fn apply(&self, table_state_set: TableStateSet) -> Result<H256> {
    //     let mut state_root = H256::zero();
    //     for (k, v) in table_state_set.table_state_sets.into_iter() {
    //         if k == ObjectID::root() {
    //             state_root = self.root_object.puts(v.entries)?
    //         } else {
    //             // must force create table
    //             let table_store = TreeObject::new(self.node_store.clone());
    //             state_root = table_store.puts(v.entries)?
    //         }
    //     }
    //     Ok(state_root)
    // }

    // pub fn dump_iter(
    //     &self,
    //     handle: &ObjectID,
    // ) -> Result<Option<SMTIterator<Vec<u8>, State, NodeDBStore>>> {
    //     if handle == &ObjectID::root() {
    //         self.root_object.iter().map(|v| Some(v))
    //     } else {
    //         self.get_as_table(handle.clone())
    //             .and_then(|res| res.map_or(Ok(None), |(_, table)| table.iter().map(|v| Some(v))))
    //     }
    // }

    // dump all states
    // pub fn dump(&self) -> Result<TableStateSet> {
    //     let global_states = self.root_object.dump()?;
    //     let mut table_state_set = TableStateSet::default();
    //     let mut golbal_table_state = TableState::default();
    //     for (key, state) in global_states.into_iter() {
    //         // If the state is an Object, and the T's struct_tag of Object<T> is Table
    //         if ObjectID::struct_tag_match(&as_struct_tag(key.key_type.clone())?) {
    //             let mut table_state = TableState::default();
    //             let table_handle = ObjectID::from_bytes(key.key.clone())?;
    //             let result = self.get_object(table_handle)?;
    //             if result.is_none() {
    //                 continue;
    //             };
    //             let obj = result.unwrap();
    //             let states = obj.dump()?;
    //             for (inner_key, inner_state) in states.into_iter() {
    //                 table_state.entries.put(inner_key, inner_state);
    //             }
    //             table_state_set
    //                 .table_state_sets
    //                 .insert(table_handle, table_state);
    //         }

    //         golbal_table_state.entries.put(key, state);
    //     }
    //     table_state_set
    //         .table_state_sets
    //         .insert(ObjectID::root(), golbal_table_state);

    //     Ok(table_state_set)
    // }

    // update global root object
    pub fn update_root(&mut self, root: RootObjectEntity) -> Result<()> {
        self.root_object = TreeObject::new(self.node_store.clone(), root.to_raw());
        Ok(())
    }
}

impl StateResolver for StateDBStore {
    fn resolve_table_item(
        &self,
        handle: &ObjectID,
        key: &KeyState,
    ) -> std::result::Result<Option<State>, Error> {
        self.resolve_state(handle, key)
    }

    fn list_table_items(
        &self,
        handle: &ObjectID,
        cursor: Option<KeyState>,
        limit: usize,
    ) -> std::result::Result<Vec<StateKV>, Error> {
        self.resolve_list_state(handle, cursor, limit)
    }
}
