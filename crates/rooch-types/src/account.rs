// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, move_resource::MoveStructType,
};
use moveos_types::state::MoveStructState;
use serde::{Deserialize, Serialize};

/// Account is the rust representation of the account in rooch_framework
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub authentication_key: Vec<u8>,
    pub sequence_number: u64,
}

impl Account {
    pub fn new(authentication_key: Vec<u8>, sequence_number: u64) -> Self {
        Self {
            authentication_key,
            sequence_number,
        }
    }
}

impl MoveStructType for Account {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("account");
    const STRUCT_NAME: &'static IdentStr = ident_str!("Account");
}

impl MoveStructState for Account {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                move_core_types::value::MoveTypeLayout::U8,
            )),
            move_core_types::value::MoveTypeLayout::U64,
        ])
    }
}
