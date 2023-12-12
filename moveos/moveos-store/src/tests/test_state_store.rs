// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::MoveOSStore;
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_core_types::effects::{AccountChangeSet, ChangeSet, Op};
use moveos_types::h256::H256;
use moveos_types::move_std::string::MoveString;
use moveos_types::move_types::random_type_tag;
use moveos_types::moveos_std::context;
use moveos_types::moveos_std::object::{NamedTableID, ObjectID};
use moveos_types::state::{MoveState, State, StateChangeSet, TableChange, TableTypeInfo};
use rand::{thread_rng, Rng};
use smt::NodeStore;
use std::str::FromStr;

fn random_bytes() -> Vec<u8> {
    H256::random().0.to_vec()
}

fn random_account_change_set() -> AccountChangeSet {
    AccountChangeSet::new()

    // let mut account_change_set = AccountChangeSet::new();

    // let mut rng = thread_rng();
    // // generate modules
    // for _n in 0..rng.gen_range(1..=5) {
    //     account_change_set
    //         .add_module_op(random_identity(), Op::New(random_bytes()))
    //         .expect("account_change_set add module op should succ");
    // }
    // // generate resources
    // for _n in 0..rng.gen_range(1..=10) {
    //     account_change_set
    //         .add_resource_op(random_struct_tag(), Op::New(random_bytes()))
    //         .expect("account_change_set add resource op should succ");
    // }

    // account_change_set
}

fn random_change_set() -> ChangeSet {
    let mut change_set = ChangeSet::new();

    let mut rng = thread_rng();
    for _n in 0..rng.gen_range(1..=5) {
        let addr = AccountAddress::random();
        change_set
            .add_account_changeset(addr, random_account_change_set())
            .expect("change_set add account change set should succ");
    }
    change_set
}

fn random_table_change() -> TableChange {
    let mut table_change = TableChange::new(random_type_tag());

    let mut rng = thread_rng();
    for _n in 0..rng.gen_range(1..=10) {
        table_change.entries.insert(
            random_bytes(),
            Op::New(State::new(random_bytes(), random_type_tag())),
        );
    }
    table_change
}

fn random_state_change_set() -> StateChangeSet {
    let mut state_change_set = StateChangeSet::default();

    // generate new tables
    let mut rng = thread_rng();
    for _n in 0..rng.gen_range(1..=5) {
        let handle = ObjectID::from(AccountAddress::random());
        state_change_set
            .new_tables
            .insert(handle, TableTypeInfo::new(random_type_tag()));
    }

    // generate remove tables
    for _n in 0..rng.gen_range(1..=5) {
        let handle = ObjectID::from(AccountAddress::random());
        state_change_set.removed_tables.insert(handle);
    }

    // generate change tables
    for _n in 0..rng.gen_range(1..=5) {
        let handle = ObjectID::from(AccountAddress::random());
        state_change_set
            .changes
            .insert(handle, random_table_change());
    }

    // generate modules change tables
    for _n in 0..rng.gen_range(1..=5) {
        let handle = NamedTableID::Module(AccountAddress::random()).to_object_id();
        state_change_set
            .changes
            .insert(handle, random_table_change());
    }

    // generate resources change tables
    for _n in 0..rng.gen_range(1..=10) {
        let handle = NamedTableID::Resource(AccountAddress::random()).to_object_id();
        state_change_set
            .changes
            .insert(handle, random_table_change());
    }

    // generate global table
    state_change_set
        .changes
        .insert(context::GLOBAL_OBJECT_STORAGE_HANDLE, random_table_change());

    state_change_set
}

#[test]
fn test_statedb() {
    let moveos_store = MoveOSStore::mock_moveos_store().unwrap();

    let change_set = ChangeSet::new();

    let mut table_change_set = StateChangeSet::default();
    let table_handle = ObjectID::ONE;
    let mut table_change = TableChange::new(random_type_tag());
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

#[test]
fn test_statedb_state_root() -> Result<()> {
    let moveos_store = MoveOSStore::mock_moveos_store().expect("moveos store mock should succ");
    let state_root = moveos_store
        .get_state_store()
        .apply_change_set(random_change_set(), random_state_change_set())?;
    let new_state_root = moveos_store
        .get_state_store()
        .apply_change_set(random_change_set(), random_state_change_set())?;
    assert_ne!(state_root, new_state_root);
    Ok(())
}

#[test]
fn test_state_db_dump_and_apply() -> Result<()> {
    let moveos_store = MoveOSStore::mock_moveos_store().expect("moveos store mock should succ");

    let (change_set, state_change_set) = (random_change_set(), random_state_change_set());
    let _state_root = moveos_store
        .get_state_store()
        .apply_change_set(change_set, state_change_set)?;

    let global_state_set = moveos_store.get_state_store().dump()?;
    let moveos_store2 = MoveOSStore::mock_moveos_store().expect("moveos store mock should succ");
    moveos_store2
        .get_state_store()
        .apply(global_state_set.clone())?;
    let global_state_set2 = moveos_store2.get_state_store().dump()?;
    assert_eq!(global_state_set, global_state_set2);
    Ok(())
}
