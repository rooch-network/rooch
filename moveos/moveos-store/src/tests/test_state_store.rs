// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::MoveOSStore;
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_core_types::effects::Op;
use moveos_types::h256::H256;
use moveos_types::move_std::string::MoveString;
use moveos_types::move_types::random_type_tag;
use moveos_types::moveos_std::account::Account;
use moveos_types::moveos_std::context;
use moveos_types::moveos_std::object::{ObjectEntity, GENESIS_STATE_ROOT};
use moveos_types::moveos_std::object_id::ObjectID;
use moveos_types::state::{KeyState, MoveState, MoveType, State, StateChangeSet, TableChange};
use rand::{thread_rng, Rng};
use smt::{NodeStore, UpdateSet};
use std::str::FromStr;

fn random_bytes() -> Vec<u8> {
    H256::random().0.to_vec()
}

fn random_table_change() -> TableChange {
    let mut table_change = TableChange::default();

    let mut rng = thread_rng();
    for _n in 0..rng.gen_range(1..=10) {
        table_change.entries.insert(
            KeyState::new(random_bytes(), random_type_tag()),
            Op::New(State::new(random_bytes(), random_type_tag())),
        );
    }
    table_change
}

fn random_state_change_set() -> StateChangeSet {
    let mut state_change_set = StateChangeSet::default();
    let mut global_change = TableChange::default();
    let mut rng = thread_rng();
    //TODO do we need the new_tables ?
    // generate new tables
    // for _n in 0..rng.gen_range(1..=5) {
    //     let handle = ObjectID::from(AccountAddress::random());
    //     state_change_set.new_tables.insert(handle);
    // }

    // // generate remove tables
    // for _n in 0..rng.gen_range(1..=5) {
    //     let handle = ObjectID::from(AccountAddress::random());
    //     state_change_set.removed_tables.insert(handle);
    // }

    // generate change tables
    for _n in 0..rng.gen_range(1..=5) {
        let handle = ObjectID::from(AccountAddress::random());
        state_change_set
            .changes
            .insert(handle, random_table_change());
        global_change.entries.insert(
            handle.to_key(),
            Op::New(ObjectEntity::new_table_object(handle, *GENESIS_STATE_ROOT, 0).into_state()),
        );
    }

    // generate resources change tables
    for _n in 0..rng.gen_range(1..=10) {
        let account = AccountAddress::random();
        let account_object_id = Account::account_object_id(account);
        state_change_set
            .changes
            .insert(account_object_id, random_table_change());
        global_change.entries.insert(
            account_object_id.to_key(),
            Op::New(ObjectEntity::new_account_object(account).into_state()),
        );
    }

    state_change_set
        .changes
        .insert(context::GLOBAL_OBJECT_STORAGE_HANDLE, global_change);

    state_change_set
}

#[test]
fn test_statedb() {
    let mut moveos_store = MoveOSStore::mock_moveos_store().unwrap();

    let mut table_change_set = StateChangeSet::default();
    let mut global_change = TableChange::default();
    let table_handle = ObjectID::ONE;
    global_change.entries.insert(
        table_handle.to_key(),
        Op::New(ObjectEntity::new_table_object(table_handle, *GENESIS_STATE_ROOT, 0).into_state()),
    );
    table_change_set
        .changes
        .insert(context::GLOBAL_OBJECT_STORAGE_HANDLE, global_change);
    let mut table_change = TableChange::default();
    let key = KeyState::new(
        MoveString::from_str("test_key").unwrap().to_bytes(),
        MoveString::type_tag(),
    );
    let value = MoveString::from_str("test_value").unwrap();

    table_change
        .entries
        .insert(key.clone(), Op::New(value.clone().into()));

    table_change_set.changes.insert(table_handle, table_change);
    moveos_store
        .get_state_store_mut()
        .apply_change_set(table_change_set)
        .unwrap();

    let state = moveos_store
        .get_state_store()
        .resolve_state(&table_handle, &key.clone().into())
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

#[test]
fn test_statedb_state_root() -> Result<()> {
    let mut moveos_store = MoveOSStore::mock_moveos_store().expect("moveos store mock should succ");
    let (state_root, size) = moveos_store
        .get_state_store_mut()
        .apply_change_set(random_state_change_set())?;
    let (new_state_root, new_size) = moveos_store
        .get_state_store_mut()
        .apply_change_set(random_state_change_set())?;
    assert_ne!(state_root, new_state_root);
    assert_ne!(size, new_size);
    Ok(())
}

// #[test]
// fn test_state_db_dump_and_apply() -> Result<()> {
//     let moveos_store = MoveOSStore::mock_moveos_store().expect("moveos store mock should succ");

//     let state_change_set = random_state_change_set();
//     let _state_root = moveos_store
//         .get_state_store()
//         .apply_change_set(state_change_set)?;

//     let global_state_set = moveos_store.get_state_store().dump()?;
//     let moveos_store2 = MoveOSStore::mock_moveos_store().expect("moveos store mock should succ");
//     moveos_store2
//         .get_state_store()
//         .apply(global_state_set.clone())?;
//     let global_state_set2 = moveos_store2.get_state_store().dump()?;
//     assert_eq!(global_state_set, global_state_set2);
//     Ok(())
// }

#[test]
fn test_state_key_and_update_set() {
    let mut update_set = UpdateSet::new();
    (0..100)
        .map(|_| {
            let object_id = ObjectID::random();
            let key = object_id.to_key();
            let state = State::new(random_bytes(), random_type_tag());
            (key, state)
        })
        .for_each(|(k, v)| update_set.put(k, v));
    assert!(update_set.len() == 100);
}
