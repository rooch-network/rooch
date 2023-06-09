// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use moveos_types::{move_string::MoveString, state::TableChange};

use super::*;
use crate::MoveOSDB;

#[test]
fn test_statedb() {
    let db = MoveOSDB::new_with_memory_store();

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
    db.state_store
        .apply_change_set(change_set, table_change_set)
        .unwrap();

    let state = db
        .get_state_store()
        .resolve_state(&table_handle, &key.to_bytes())
        .unwrap();
    assert!(state.is_some());
    assert_eq!(state.unwrap(), value.into());
}
