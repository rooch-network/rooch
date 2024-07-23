// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::state_store::metrics::StateDBMetrics;
use crate::state_store::NodeDBStore;
use anyhow::{Error, Ok, Result};
use function_name::named;
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
use prometheus::Registry;
use smt::{SMTIterator, TreeChangeSet};
use smt::{SMTree, UpdateSet};
use std::collections::BTreeMap;
use std::sync::Arc;

pub const STATEDB_DUMP_BATCH_SIZE: usize = 5000;

/// StateDB provide state storage and state proof
#[derive(Clone)]
pub struct StateDBStore {
    pub node_store: NodeDBStore,
    smt: SMTree<FieldKey, ObjectState, NodeDBStore>,
    metrics: Arc<StateDBMetrics>,
}

impl StateDBStore {
    pub fn new(node_store: NodeDBStore, registry: &Registry) -> Self {
        Self {
            node_store: node_store.clone(),
            smt: SMTree::new(node_store),
            metrics: Arc::new(StateDBMetrics::new(registry)),
        }
    }

    #[named]
    pub fn update_fields<I>(&self, pre_state_root: H256, update_set: I) -> Result<TreeChangeSet>
    where
        I: Into<UpdateSet<FieldKey, ObjectState>>,
    {
        let fn_name = function_name!();
        let _timer = self
            .metrics
            .state_update_fields_latency
            .with_label_values(&[fn_name])
            .start_timer();
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

    #[named]
    pub fn update_nodes(&self, nodes: BTreeMap<H256, Vec<u8>>) -> Result<()> {
        let fn_name = function_name!();
        let _timer = self
            .metrics
            .state_update_nodes_latency
            .with_label_values(&[fn_name])
            .start_timer();
        let size = nodes.values().map(|v| 32 + v.len()).sum::<usize>();
        self.node_store.write_nodes(nodes)?;
        self.metrics
            .state_update_nodes_bytes
            .with_label_values(&[fn_name])
            .observe(size as f64);
        Ok(())
    }

    fn apply_object_change(
        &self,
        resolver: &dyn StateResolver,
        nodes: &mut BTreeMap<H256, Vec<u8>>,
        update_set: &mut UpdateSet<FieldKey, ObjectState>,
        field_key: FieldKey,
        obj_change: &mut ObjectChange,
    ) -> Result<()> {
        let mut obj = match &obj_change.value {
            Some(op) => match op {
                Op::New(state) | Op::Modify(state) => {
                    ObjectState::new(obj_change.metadata.clone(), state.clone())
                }
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
                obj_state.metadata = obj_change.metadata.clone();
                obj_state
            }
        };
        let mut field_update_set = UpdateSet::new();
        for (child_field_key, child_change) in &mut obj_change.fields {
            self.apply_object_change(
                resolver,
                nodes,
                &mut field_update_set,
                *child_field_key,
                child_change,
            )?;
        }
        let mut tree_change_set = self.update_fields(obj.state_root(), field_update_set)?;
        nodes.append(&mut tree_change_set.nodes);
        let new_state_root = tree_change_set.state_root;
        obj.update_state_root(new_state_root);
        obj_change.update_state_root(new_state_root);
        update_set.put(field_key, obj);

        Ok(())
    }

    #[named]
    pub fn apply_change_set(&self, state_change_set: &mut StateChangeSet) -> Result<()> {
        let fn_name = function_name!();
        let _timer = self
            .metrics
            .state_apply_change_set_latency
            .with_label_values(&[fn_name])
            .start_timer();

        let root = state_change_set.root_metadata();
        let pre_state_root = root.state_root();
        let global_size = root.size;

        let resolver = RootObjectResolver::new(root, self);

        let mut update_set = UpdateSet::new();
        let mut nodes = BTreeMap::new();
        for (field_key, obj_change) in &mut state_change_set.changes {
            self.apply_object_change(
                &resolver,
                &mut nodes,
                &mut update_set,
                *field_key,
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
        let size = nodes.values().map(|v| 32 + v.len()).sum::<usize>();
        self.node_store.write_nodes(nodes)?;
        state_change_set.update_state_root(new_state_root);

        self.metrics
            .state_apply_change_set_bytes
            .with_label_values(&[fn_name])
            .observe(size as f64);
        Ok(())
    }

    #[named]
    pub fn iter(
        &self,
        state_root: H256,
        starting_key: Option<FieldKey>,
    ) -> Result<SMTIterator<FieldKey, ObjectState, NodeDBStore>> {
        let fn_name = function_name!();
        let _timer = self
            .metrics
            .state_iter_latency
            .with_label_values(&[fn_name])
            .start_timer();
        self.smt.iter(state_root, starting_key)
    }
}

impl StatelessResolver for StateDBStore {
    #[named]
    fn get_field_at(&self, state_root: H256, key: &FieldKey) -> Result<Option<ObjectState>, Error> {
        let fn_name = function_name!();
        let _timer = self
            .metrics
            .state_get_field_at_latency
            .with_label_values(&[fn_name])
            .start_timer();

        if state_root == *GENESIS_STATE_ROOT {
            return Ok(None);
        }
        // self.smt.get(state_root, *key)
        let result = self.smt.get(state_root, *key)?;
        if log::log_enabled!(log::Level::Trace) {
            let result_info = match &result {
                Some(state) => format!("Some({})", state.metadata.object_type),
                None => "None".to_string(),
            };
            log::trace!(
                "get_field_at state_root: {} key: {}, result: {:?}",
                state_root,
                key,
                result_info
            );
        }
        //TODO Add perf mode, only output bytes statistics in perf mode,
        // there will be performance loss
        // let size = result.map(|v| v.to_bytes()?.len()).unwrap_or(0);
        // self.metrics
        //     .state_get_field_at_bytes
        //     .with_label_values(&[fn_name])
        //     .observe(size as f64);
        Ok(result)
    }

    #[named]
    fn list_fields_at(
        &self,
        state_root: H256,
        cursor: Option<FieldKey>,
        limit: usize,
    ) -> Result<Vec<StateKV>> {
        let fn_name = function_name!();
        let _timer = self
            .metrics
            .state_list_fields_at_latency
            .with_label_values(&[fn_name])
            .start_timer();
        let result = self.smt.list(state_root, cursor, limit)?;

        //TODO Add perf mode, only output bytes statistics in perf mode,
        // there will be performance loss
        // let size = result.iter().map(|(_k,v)| {
        //     let k_len = AccountAddress::LENGTH;
        //     let v_len = v.to_bytes()?.len();
        //     k_len + v_len
        // }).sum::<usize>();
        // self.metrics
        //     .state_list_fields_at_bytes
        //     .with_label_values(&[fn_name])
        //     .observe(size as f64);
        Ok(result)
    }
}
