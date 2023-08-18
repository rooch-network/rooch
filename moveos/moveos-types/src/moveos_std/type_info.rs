// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    state::{MoveStructState, MoveStructType},
};
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::{IdentStr, Identifier},
};
use serde::{Deserialize, Serialize};

const MODULE_NAME: &IdentStr = ident_str!("type_info");

/// The structure of TypeInfo is consistent of contract type_info
#[derive(Clone, Debug, Eq, Ord, PartialOrd, PartialEq, Serialize, Deserialize, Hash)]
pub struct TypeInfo {
    pub account_address: AccountAddress,
    pub module_name: Vec<u8>,
    pub struct_name: Vec<u8>,
}

impl TypeInfo {
    pub fn new(
        account_address: AccountAddress,
        module_name: Identifier,
        struct_name: Identifier,
    ) -> Self {
        Self {
            account_address,
            module_name: module_name.as_bytes().to_vec(),
            struct_name: struct_name.as_bytes().to_vec(),
        }
    }
}

impl MoveStructType for TypeInfo {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("TypeInfo");
}

impl MoveStructState for TypeInfo {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::Address,
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                move_core_types::value::MoveTypeLayout::U8,
            )),
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                move_core_types::value::MoveTypeLayout::U8,
            )),
        ])
    }
}
