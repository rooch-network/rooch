// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::MoveOSDB;

#[test]
fn test_statedb() {
    let db = MoveOSDB::new_with_memory_store();

    let mut change_set = ChangeSet::new();
    let struct_tag: StructTag = "0x1::account::Account".parse().unwrap();
    for i in 1..10 {
        change_set
            .add_resource_op(
                AccountAddress::from_hex_literal(format!("0x{}", i).as_str()).unwrap(),
                struct_tag.clone(),
                Op::New(vec![i]),
            )
            .unwrap();
    }
    let table_change_set = TableChangeSet::default();
    db.state_store
        .apply_change_set(change_set, table_change_set)
        .unwrap();
    let addr = AccountAddress::from_hex_literal("0x1").unwrap();
    let resource = db
        .get_state_store()
        .get_resource(&addr, &struct_tag)
        .unwrap();
    assert!(resource.is_some());
    assert_eq!(resource.unwrap(), vec![1]);
}
