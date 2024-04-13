// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::MoveOSStore;
use anyhow::Result;
use move_core_types::effects::Op;
use moveos_types::h256::H256;
use moveos_types::move_std::string::MoveString;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::moveos_std::object::{ObjectEntity, GENESIS_STATE_ROOT};
use moveos_types::moveos_std::table::TablePlaceholder;
use moveos_types::state::{
    FieldChange, KeyState, MoveState, MoveType, ObjectChange, StateChangeSet,
};
use moveos_types::state_resolver::StatelessResolver;
use moveos_types::test_utils::random_state_change_set;
use smt::NodeReader;
use std::str::FromStr;

#[test]
fn test_statedb() {
    let mut moveos_store = MoveOSStore::mock_moveos_store().unwrap();

    let mut state_change_set = StateChangeSet::default();

    let object_id = ObjectID::random();

    let obj = ObjectEntity::new_table_object(object_id.clone(), *GENESIS_STATE_ROOT, 0);

    let mut object_change = ObjectChange::new(Op::New(obj.into_state()));

    let key = KeyState::new(
        MoveString::from_str("test_key").unwrap().to_bytes(),
        MoveString::type_tag(),
    );
    let value = MoveString::from_str("test_value").unwrap();

    object_change.add_field_change(
        key.clone(),
        FieldChange::new_normal(Op::New(value.clone().into())),
    );

    state_change_set
        .changes
        .insert(object_id.clone(), object_change);

    let (state_root, _size) = moveos_store
        .get_state_store_mut()
        .apply_change_set(state_change_set)
        .unwrap();

    let state = moveos_store
        .get_state_store()
        .get_field_at(state_root, &object_id.to_key())
        .unwrap();
    assert!(state.is_some());
    let obj2 = state.unwrap().as_object::<TablePlaceholder>().unwrap();
    //The object field size is not changed, because the size is updated in the `object.move`` Move module.
    //assert_eq!(obj2.size, 1);
    let value_state = moveos_store.get_field_at(obj2.state_root(), &key).unwrap();
    assert_eq!(value_state.unwrap(), value.into());
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
    let change_set = random_state_change_set();
    let (state_root, _size) = moveos_store
        .get_state_store_mut()
        .apply_change_set(change_set)?;
    let (new_state_root, _new_size) = moveos_store
        .get_state_store_mut()
        .apply_change_set(random_state_change_set())?;
    assert_ne!(state_root, new_state_root);
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
