// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::{
    move_string::MoveAsciiString,
    state::{MoveStructState, MoveStructType},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct SessionScope {
    pub module_address: AccountAddress,
    pub module_name: MoveAsciiString,
    pub function_name: MoveAsciiString,
}

impl SessionScope {
    pub fn new(module_address: AccountAddress, module_name: &str, function_name: &str) -> Self {
        Self {
            module_address,
            module_name: MoveAsciiString::from_str(module_name).expect("invalid module name"),
            function_name: MoveAsciiString::from_str(function_name).expect("invalid function name"),
        }
    }
}

impl MoveStructType for SessionScope {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("session_key");
    const STRUCT_NAME: &'static IdentStr = ident_str!("SessionScope");
}

impl MoveStructState for SessionScope {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::Address,
            MoveAsciiString::type_layout(),
            MoveAsciiString::type_layout(),
        ])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionKey {
    pub authentication_key: Vec<u8>,
    pub scheme: u64,
    pub scopes: Vec<SessionScope>,
    pub expiration_time: u64,
    pub last_active_time: u64,
    pub max_inactive_interval: u64,
}

impl MoveStructType for SessionKey {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("session_key");
    const STRUCT_NAME: &'static IdentStr = ident_str!("SessionKey");
}

impl MoveStructState for SessionKey {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                move_core_types::value::MoveTypeLayout::U8,
            )),
            move_core_types::value::MoveTypeLayout::U64,
            move_core_types::value::MoveTypeLayout::Vector(Box::new(SessionScope::type_layout())),
            move_core_types::value::MoveTypeLayout::U64,
            move_core_types::value::MoveTypeLayout::U64,
            move_core_types::value::MoveTypeLayout::U64,
        ])
    }
}
