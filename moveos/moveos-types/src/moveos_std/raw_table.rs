// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    state::{MoveStructState, MoveStructType},
};
use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    value::{MoveStructLayout, MoveTypeLayout},
};
use serde::{Deserialize, Serialize};
use smt::SPARSE_MERKLE_PLACEHOLDER_HASH;

pub const MODULE_NAME: &IdentStr = ident_str!("raw_table");

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct TableInfo {
    pub state_root: AccountAddress,
    pub size: u64,
}

impl Default for TableInfo {
    fn default() -> Self {
        TableInfo {
            state_root: AccountAddress::new((*SPARSE_MERKLE_PLACEHOLDER_HASH).into()),
            size: 0u64,
        }
    }
}

impl TableInfo {
    pub fn new(state_root: AccountAddress) -> Result<Self> {
        Ok(TableInfo {
            state_root,
            size: 0u64,
        })
    }
}

impl MoveStructType for TableInfo {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("TableInfo");

    fn struct_tag() -> move_core_types::language_storage::StructTag {
        move_core_types::language_storage::StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![],
        }
    }
}

impl MoveStructState for TableInfo {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Address, MoveTypeLayout::U64])
    }
}
