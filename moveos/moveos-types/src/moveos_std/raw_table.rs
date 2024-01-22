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

pub const MODULE_NAME: &IdentStr = ident_str!("raw_table");

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct TableInfo {
    //TODO use u256?
    pub state_root: AccountAddress,
    pub size: u64,
    // // The type name of table key TypeTag
    // key_type: MoveString,
}

impl TableInfo {
    pub fn new(state_root: AccountAddress) -> Result<Self> {
        // let key_type_str = MoveString::from_str(key_type.to_string().as_str())?;
        Ok(TableInfo {
            state_root,
            size: 0u64,
            // key_type: key_type_str,
        })
    }

    // pub fn key_type(&self) -> String {
    //     self.key_type.to_string()
    // }

    // pub fn key_type_tag(&self) -> Result<TypeTag> {
    //     let key_type_str = self.key_type.to_string();
    //     TypeTag::from_str(key_type_str.as_str()).map_err(|_e| {
    //         anyhow::anyhow!(
    //             "key type in TableInfo should be valid TypeTag: {}",
    //             key_type_str
    //         )
    //     })
    // }
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
