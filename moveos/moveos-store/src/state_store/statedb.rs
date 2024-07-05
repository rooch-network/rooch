// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::state_store::NodeDBStore;
use anyhow::{Error, Result};
use move_core_types::effects::Op;
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::GENESIS_STATE_ROOT;
use moveos_types::state::FieldKey;
use moveos_types::state::ObjectChange;
use moveos_types::state::ObjectState;
use moveos_types::state::StateChangeSet;
use moveos_types::state_resolver::RootObjectResolver;
use moveos_types::state_resolver::StateKV;
use moveos_types::state_resolver::StateResolver;
use moveos_types::state_resolver::StatelessResolver;
use smt::{SMTIterator, TreeChangeSet};
use smt::{SMTree, UpdateSet};
use std::collections::BTreeMap;

pub const STATEDB_DUMP_BATCH_SIZE: usize = 5000;

/// StateDB provide state storage and state proof
#[derive(Clone)]
pub struct StateDBStore {
    pub node_store: NodeDBStore,
    smt: SMTree<FieldKey, ObjectState, NodeDBStore>,
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
        I: Into<UpdateSet<FieldKey, ObjectState>>,
    {
        let update_set: UpdateSet<FieldKey, ObjectState> = update_set.into();
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
        update_set: &mut UpdateSet<FieldKey, ObjectState>,
        field_key: FieldKey,
        obj_change: ObjectChange,
    ) -> Result<()> {
        let mut obj = match obj_change.value {
            Some(op) => match op {
                Op::New(state) | Op::Modify(state) => ObjectState::new(obj_change.metadata, state),
                Op::Delete => {
                    //TODO clean up the removed object fields
                    update_set.remove(field_key);
                    return Ok(());
                }
            },
            None => {
                let object_id = obj_change.metadata.id.clone();
                // The VM do not change the value of Object
                let mut obj_state = resolver
                    .get_object(&object_id)?
                    .ok_or_else(|| anyhow::format_err!("Object with id {} not found", object_id))?;
                //The object value is not changed, but the metadata may be changed
                obj_state.metadata = obj_change.metadata;
                obj_state
            }
        };
        let mut field_update_set = UpdateSet::new();
        for (child_field_key, child_change) in obj_change.fields {
            self.apply_object_change(
                resolver,
                nodes,
                &mut field_update_set,
                child_field_key,
                child_change,
            )?;
        }
        let mut tree_change_set = self.update_fields(obj.state_root(), field_update_set)?;
        nodes.append(&mut tree_change_set.nodes);
        obj.update_state_root(tree_change_set.state_root);
        update_set.put(field_key, obj);
        Ok(())
    }

    pub fn apply_change_set(&self, state_change_set: StateChangeSet) -> Result<(H256, u64)> {
        let root = state_change_set.root_object();
        let pre_state_root = root.metadata.state_root();
        let global_size = root.metadata.size;

        let resolver = RootObjectResolver::new(root, self);

        let mut update_set = UpdateSet::new();
        let mut nodes = BTreeMap::new();
        for (field_key, obj_change) in state_change_set.changes {
            self.apply_object_change(
                &resolver,
                &mut nodes,
                &mut update_set,
                field_key,
                obj_change,
            )?;
        }

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
        starting_key: Option<FieldKey>,
    ) -> Result<SMTIterator<FieldKey, ObjectState, NodeDBStore>> {
        self.smt.iter(state_root, starting_key)
    }
}

impl StatelessResolver for StateDBStore {
    fn get_field_at(
        &self,
        state_root: H256,
        key: &FieldKey,
    ) -> std::result::Result<Option<ObjectState>, Error> {
        if state_root == *GENESIS_STATE_ROOT {
            return Ok(None);
        }
        let result = self.smt.get(state_root, *key);
        if log::log_enabled!(log::Level::Trace) {
            let result_info = match &result {
                Ok(Some(state)) => format!("Some({})", state.metadata.value_type),
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
        cursor: Option<FieldKey>,
        limit: usize,
    ) -> Result<Vec<StateKV>> {
        self.smt.list(state_root, cursor, limit)
    }
}
