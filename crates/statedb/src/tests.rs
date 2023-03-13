// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[test]
fn test_statedb() {
    let state_db = StateDB::new();
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
    state_db
        .apply_change_set(change_set, table_change_set)
        .unwrap();
    let addr = AccountAddress::from_hex_literal("0x1").unwrap();
    let resource = state_db.get_resource(&addr, &struct_tag).unwrap();
    assert!(resource.is_some());
    assert_eq!(resource.unwrap(), vec![1]);
}
