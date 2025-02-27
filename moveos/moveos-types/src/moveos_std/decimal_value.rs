// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    state::{MoveStructState, MoveStructType},
};
use move_core_types::u256::U256;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use serde::{Deserialize, Serialize};

const MODULE_NAME: &IdentStr = ident_str!("decimal_value");

/// `DecimalValue` is represented `moveos_std::decimal_value::DecimalValue` in Move.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DecimalValue {
    pub value: U256,
    pub decimal: u8,
}

impl DecimalValue {
    pub fn new(value: U256, decimal: u8) -> Self {
        Self { value, decimal }
    }
}

impl MoveStructType for DecimalValue {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("DecimalValue");
}

impl MoveStructState for DecimalValue {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::U256,
            move_core_types::value::MoveTypeLayout::U8,
        ])
    }
}
