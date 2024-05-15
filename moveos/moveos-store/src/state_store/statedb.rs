// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::state_store::NodeDBStore;
use anyhow::{Error, Result};
use move_core_types::effects::Op;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::moveos_std::object::GENESIS_STATE_ROOT;
use moveos_types::state::FieldChange;
use moveos_types::state::KeyState;
use moveos_types::state::ObjectChange;
use moveos_types::state::StateChangeSet;
use moveos_types::state_resolver::RootObjectResolver;
use moveos_types::state_resolver::StateKV;
use moveos_types::state_resolver::StateResolver;
use moveos_types::state_resolver::StatelessResolver;
use moveos_types::{h256::H256, state::State};
use smt::{SMTIterator, TreeChangeSet};
use smt::{SMTree, UpdateSet};
use std::collections::BTreeMap;

pub const STATEDB_DUMP_BATCH_SIZE: usize = 5000;

/// StateDB provide state storage and state proof
#[derive(Clone)]
pub struct StateDBStore {
    pub node_store: NodeDBStore,
    smt: SMTree<KeyState, State, NodeDBStore>,
}

impl StateDBStore {
    pub fn new(node_store: NodeDBStore) -> Self {
        Self {
            node_store: node_store.clone(),
            smt: SMTree::new(node_store),
        }
    }

    pub fn update_fields<I>(&self, pre_state_root: H256, update_set: I) -> Result<TreeChangeSet>
    where
        I: Into<UpdateSet<KeyState, State>>,
    {
        let update_set: UpdateSet<KeyState, State> = update_set.into();
        let change_set = self.smt.puts(pre_state_root, update_set)?;
        if log::log_enabled!(log::Level::Trace) {
            log::trace!(
                "update_fields pre_state_root: {}, new_state_root: {}",
                pre_state_root,
                change_set.state_root,
            );
        }
        Ok(change_set)
    }

    pub fn update_nodes(&self, nodes: BTreeMap<H256, Vec<u8>>) -> Result<()> {
        self.node_store.write_nodes(nodes)?;
        Ok(())
    }

    fn apply_object_change(
        &self,
        resolver: &dyn StateResolver,
        nodes: &mut BTreeMap<H256, Vec<u8>>,
        update_set: &mut UpdateSet<KeyState, State>,
        object_id: ObjectID,
        obj_change: ObjectChange,
    ) -> Result<()> {
        let mut obj = match obj_change.op {
            Some(op) => match op {
                Op::New(state) | Op::Modify(state) => state.as_raw_object()?,
                Op::Delete => {
                    //TODO clean up the removed object fields
                    update_set.remove(object_id.to_key());
                    return Ok(());
                }
            },
            None => {
                // The VM do not change the value of ObjectEntity
                resolver
                    .get_object(&object_id)?
                    .ok_or_else(|| anyhow::format_err!("Object with id {} not found", object_id))?
            }
        };
        let mut field_update_set = UpdateSet::new();
        for (key, change) in obj_change.fields {
            match change {
                FieldChange::Normal(field) => match field.op {
                    Op::New(state) | Op::Modify(state) => {
                        field_update_set.put(key, state);
                    }
                    Op::Delete => {
                        field_update_set.remove(key);
                    }
                },
                FieldChange::Object(obj_change) => {
                    self.apply_object_change(
                        resolver,
                        nodes,
                        &mut field_update_set,
                        key.as_object_id()?,
                        obj_change,
                    )?;
                }
            }
        }
        let mut tree_change_set = self.update_fields(obj.state_root(), field_update_set)?;
        nodes.append(&mut tree_change_set.nodes);
        obj.update_state_root(tree_change_set.state_root);
        update_set.put(object_id.to_key(), obj.into_state());
        Ok(())
    }

    pub fn apply_change_set(&self, state_change_set: StateChangeSet) -> Result<(H256, u64)> {
        let root = state_change_set.root_object();
        let resolver = RootObjectResolver::new(root, self);
        let pre_state_root = state_change_set.state_root;
        let mut update_set = UpdateSet::new();
        let mut nodes = BTreeMap::new();
        for (object_id, obj_change) in state_change_set.changes {
            self.apply_object_change(
                &resolver,
                &mut nodes,
                &mut update_set,
                object_id,
                obj_change,
            )?;
        }
        let global_size = state_change_set.global_size;

        let mut tree_change_set = self.update_fields(pre_state_root, update_set)?;
        let new_state_root = tree_change_set.state_root;
        nodes.append(&mut tree_change_set.nodes);
        if log::log_enabled!(log::Level::Debug) {
            log::debug!(
                "apply_change_set new_state_root: {:?}, smt nodes: {}, new_global_size: {}",
                new_state_root,
                nodes.len(),
                global_size
            );
        }
        self.node_store.write_nodes(nodes)?;
        Ok((new_state_root, global_size))
    }

    pub fn iter(
        &self,
        state_root: H256,
        starting_key: Option<KeyState>,
    ) -> Result<SMTIterator<KeyState, State, NodeDBStore>> {
        self.smt.iter(state_root, starting_key)
    }
}

impl StatelessResolver for StateDBStore {
    fn get_field_at(
        &self,
        state_root: H256,
        key: &KeyState,
    ) -> std::result::Result<Option<State>, Error> {
        if state_root == *GENESIS_STATE_ROOT {
            return Ok(None);
        }
        let result = self.smt.get(state_root, key.clone());
        if log::log_enabled!(log::Level::Trace) {
            let result_info = match &result {
                Ok(Some(state)) => format!("Some({})", state.value_type),
                Ok(None) => "None".to_string(),
                Err(e) => format!("Error({:?})", e),
            };
            log::trace!(
                "get_field_at state_root: {} key: {}, result: {:?}",
                state_root,
                key,
                result_info
            );
        }
        result
    }

    fn list_fields_at(
        &self,
        state_root: H256,
        cursor: Option<KeyState>,
        limit: usize,
    ) -> Result<Vec<StateKV>> {
        self.smt.list(state_root, cursor, limit)
    }
}
