// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::effects::{ChangeSet, Op};
use moveos_types::h256::H256;
use moveos_types::object::ObjectID;
use moveos_types::state::{MoveState, StateChangeSet};
use moveos_types::{move_string::MoveString, state::TableChange};
use smt::NodeStore;
use std::str::FromStr;

use crate::MoveOSStore;

#[test]
fn test_statedb() {
    let moveos_store = MoveOSStore::mock_moveos_store().unwrap();

    let change_set = ChangeSet::new();

    let mut table_change_set = StateChangeSet::default();
    let table_handle = ObjectID::ONE;
    let mut table_change = TableChange::default();
    let key = MoveString::from_str("test_key").unwrap();
    let value = MoveString::from_str("test_value").unwrap();

    table_change
        .entries
        .insert(key.to_bytes(), Op::New(value.clone().into()));

    table_change_set.changes.insert(table_handle, table_change);
    moveos_store
        .get_state_store()
        .apply_change_set(change_set, table_change_set)
        .unwrap();

    let state = moveos_store
        .get_state_store()
        .resolve_state(&table_handle, &key.to_bytes())
        .unwrap();
    assert!(state.is_some());
    assert_eq!(state.unwrap(), value.into());
}

#[test]
fn test_reopen() {
    let moveos_store = MoveOSStore::mock_moveos_store().unwrap();
    let node_store = moveos_store.get_state_node_store();

    let key = H256::random();
    let node = b"testnode".to_vec();
    {
        node_store
            .put(key, node.clone())
            .map_err(|e| anyhow::anyhow!("test_state_store test_reopen error: {:?}", e))
            .ok();
        assert_eq!(node_store.get(&key).unwrap(), Some(node.clone()));
    }
    {
        assert_eq!(node_store.get(&key).unwrap(), Some(node));
    }
}
