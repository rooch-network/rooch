// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::MoveOSStore;
use move_core_types::account_address::AccountAddress;
use move_core_types::effects::Op;
use move_core_types::language_storage::TypeTag;
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::{ObjectID, ObjectMeta};
use moveos_types::state::{FieldKey, ObjectChange, ObjectState, StateChangeSet};
use smt::NodeReader;

fn reachable_nodes(
    store: &MoveOSStore,
    root: H256,
    limit: Option<usize>,
) -> std::collections::HashSet<H256> {
    let mut seen = std::collections::HashSet::new();
    let mut stack = vec![root];
    let node_store = store.get_state_node_store();
    while let Some(node_hash) = stack.pop() {
        if !seen.insert(node_hash) {
            continue;
        }
        if let Ok(Some(bytes)) = node_store.get(&node_hash) {
            if let Ok(node) =
                smt::jellyfish_merkle::node_type::Node::<FieldKey, ObjectState>::decode(&bytes)
            {
                match node {
                    smt::jellyfish_merkle::node_type::Node::Internal(internal) => {
                        for child in internal.all_child() {
                            stack.push(child.into());
                        }
                    }
                    smt::jellyfish_merkle::node_type::Node::Leaf(leaf) => {
                        let state = &leaf.value().origin;
                        let child_root = state.metadata.state_root();
                        if child_root != *smt::SPARSE_MERKLE_PLACEHOLDER_HASH {
                            stack.push(child_root);
                        }
                    }
                    smt::jellyfish_merkle::node_type::Node::Null => {}
                }
            }
        }
        if let Some(limit) = limit {
            if seen.len() >= limit {
                break;
            }
        }
    }
    seen
}

fn make_meta(id: ObjectID, state_root: Option<H256>) -> ObjectMeta {
    ObjectMeta::new(
        id,
        AccountAddress::ZERO,
        0,
        state_root,
        0,
        0,
        0,
        TypeTag::Bool,
    )
}

fn make_object(id: ObjectID, state_root: Option<H256>, val: Vec<u8>) -> ObjectState {
    ObjectState::new(make_meta(id, state_root), val)
}

#[test]
fn test_staledb_stale_indices_unreachable_after_apply() {
    let (moveos_store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let state_store = moveos_store.get_state_store();
    let node_store = moveos_store.get_state_node_store();

    // Build child object tree
    let child_id = ObjectID::random();
    let mut child_change = StateChangeSet::new(*smt::SPARSE_MERKLE_PLACEHOLDER_HASH, 0);
    let child_field = FieldKey::derive_from_string("child");
    let child_obj_v1 = make_object(child_id.clone(), None, vec![1u8; 4]);
    let mut child_change_obj = ObjectChange::new(
        child_obj_v1.metadata.clone(),
        Op::New(child_obj_v1.value.clone()),
    );
    child_change_obj.update_state_root(child_obj_v1.metadata.state_root());
    child_change.changes.insert(child_field, child_change_obj);
    let (child_nodes, _child_stale) = state_store.change_set_to_nodes(&mut child_change).unwrap();
    node_store.write_nodes(child_nodes).unwrap();
    let child_root_v1 = child_change.state_root;

    // Root object references child_root_v1
    let root_id = ObjectID::random();
    let root_field = FieldKey::derive_from_string("root");
    let mut root_cs = StateChangeSet::new(*smt::SPARSE_MERKLE_PLACEHOLDER_HASH, 0);
    let mut root_change = ObjectChange::new(
        make_meta(root_id.clone(), Some(child_root_v1)),
        Op::New(vec![9u8; 4]),
    );
    root_change.update_state_root(child_root_v1);
    root_cs.changes.insert(root_field, root_change);
    let (root_nodes, _root_stale) = state_store.change_set_to_nodes(&mut root_cs).unwrap();
    node_store.write_nodes(root_nodes).unwrap();
    let root_state_root_v1 = root_cs.state_root;

    // Update child object (new state_root), ensure stale nodes are unreachable
    let mut child_change2 = StateChangeSet::new(child_root_v1, 0);
    let child_obj_v2 = make_object(child_id.clone(), None, vec![2u8; 4]);
    let mut child_change_obj2 = ObjectChange::new(
        child_obj_v2.metadata.clone(),
        Op::Modify(child_obj_v2.value.clone()),
    );
    child_change_obj2.update_state_root(child_obj_v2.metadata.state_root());
    child_change2.changes.insert(child_field, child_change_obj2);
    let (child_nodes2, child_stale2) = state_store.change_set_to_nodes(&mut child_change2).unwrap();
    node_store.write_nodes(child_nodes2).unwrap();
    let child_root_v2 = child_change2.state_root;

    // Root update points to new child root
    let mut root_cs2 = StateChangeSet::new(root_state_root_v1, 0);
    let mut root_change2 = ObjectChange::new(
        make_meta(root_id.clone(), Some(child_root_v1)),
        Op::Modify(vec![9u8; 4]),
    );
    root_change2.update_state_root(child_root_v2);
    root_cs2.changes.insert(root_field, root_change2);
    let (root_nodes2, root_stale2) = state_store.change_set_to_nodes(&mut root_cs2).unwrap();
    node_store.write_nodes(root_nodes2).unwrap();
    let new_root = root_cs2.state_root;

    let reachable = reachable_nodes(&moveos_store, new_root, None);
    for (_sr, stale) in child_stale2.iter().chain(root_stale2.iter()) {
        assert!(
            !reachable.contains(stale),
            "stale node must not be reachable in new root"
        );
    }
}
