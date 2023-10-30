// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    state::{MoveStructState, MoveStructType},
};
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    value::{MoveStructLayout, MoveTypeLayout},
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("raw_table");

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct TableInfo {
    //TODO use u256?
    pub state_root: AccountAddress,
    //TODO keep Table Key TypeTag at here
    pub size: u64,
}

impl TableInfo {
    pub fn new(state_root: AccountAddress) -> Self {
        TableInfo {
            state_root,
            size: 0u64,
        }
    }
}

impl MoveStructType for TableInfo {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("TableInfo");
}

impl MoveStructState for TableInfo {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Address, MoveTypeLayout::U64])
    }
}
