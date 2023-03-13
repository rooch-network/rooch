// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::moveos::MoveOS;
use move_core_types::{
    account_address::AccountAddress,
    identifier::IdentStr,
    language_storage::ModuleId,
    value::{serialize_values, MoveValue},
};
use statedb::{StateDB};

#[test]
pub fn test_moveos() {
    let db = StateDB::new();
    let moveos = MoveOS::new(db).unwrap();
    //let hash_module = ModuleId::new(AccountAddress::from_hex_literal("0x1").unwrap(), IdentStr::new("hash").unwrap().to_owned());
    //let data = HashValue::random().to_vec();

    let account_module = ModuleId::new(
        AccountAddress::from_hex_literal("0x1").unwrap(),
        IdentStr::new("account").unwrap().to_owned(),
    );

    let args = serialize_values(&vec![MoveValue::U64(1), MoveValue::U64(2)]);
    let result = moveos
        .execute_view_function(&account_module, IdentStr::new("add").unwrap(), vec![], args)
        .unwrap();
    assert_eq!(
        result.return_values[0].0,
        serialize_values(&vec![MoveValue::U64(3)])[0]
    );
}
