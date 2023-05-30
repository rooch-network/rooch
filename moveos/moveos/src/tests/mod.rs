// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::moveos::MoveOS;
use move_core_types::{
    account_address::AccountAddress,
    value::{serialize_values, MoveValue},
};
use moveos_store::MoveOSDB;
use moveos_types::{move_types::FunctionId, transaction::FunctionCall};
use std::str::FromStr;

#[test]
pub fn test_check_account() {
    let db = MoveOSDB::new_with_memory_store();
    let moveos = MoveOS::new(db).unwrap();

    let args = serialize_values(&vec![MoveValue::Address(AccountAddress::ZERO)]);

    let function_id = FunctionId::from_str("0x1::account::exists_at").unwrap();

    let result = moveos
        .execute_view_function(FunctionCall::new(function_id, vec![], args))
        .unwrap();

    assert_eq!(
        result.return_values[0].0,
        serialize_values(&vec![MoveValue::Bool(false)])[0]
    );
}
