// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::moveos::MoveOS;
use move_core_types::{
    account_address::AccountAddress,
    // account_address::AccountAddress,
    identifier::IdentStr,
    language_storage::ModuleId,
    value::{serialize_values, MoveValue},
};
use moveos_store::state_store::StateDB;
use moveos_types::addresses::MOVEOS_STD_ADDRESS;

#[test]
pub fn test_moveos() {
    let db = StateDB::new_with_memory_store();
    let moveos = MoveOS::new(db).unwrap();

    let math_module = ModuleId::new(
        *MOVEOS_STD_ADDRESS,
        IdentStr::new("math").unwrap().to_owned(),
    );

    let args = serialize_values(&vec![MoveValue::U64(1), MoveValue::U64(2)]);
    let result = moveos
        .execute_view_function(&math_module, IdentStr::new("add").unwrap(), vec![], args)
        .unwrap();
    assert_eq!(
        result.return_values[0].0,
        serialize_values(&vec![MoveValue::U64(3)])[0]
    );
}

#[test]
pub fn test_check_account() {
    let db = StateDB::new_with_memory_store();
    let moveos = MoveOS::new(db).unwrap();

    let math_module = ModuleId::new(
        *MOVEOS_STD_ADDRESS,
        IdentStr::new("account").unwrap().to_owned(),
    );
    let args = serialize_values(&vec![MoveValue::Address(AccountAddress::ZERO)]);
    let result = moveos
        .execute_view_function(
            &math_module,
            IdentStr::new("exists_at").unwrap(),
            vec![],
            args,
        )
        .unwrap();

    assert_eq!(
        result.return_values[0].0,
        serialize_values(&vec![MoveValue::Bool(false)])[0]
    );
}
