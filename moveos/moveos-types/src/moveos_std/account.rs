// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::MOVEOS_STD_ADDRESS;
use crate::moveos_std::object::ObjectID;
use crate::state::{MoveStructState, MoveStructType};
use move_core_types::language_storage::StructTag;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("account");

/// Account is the rust representation of the account in moveos_std
#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Default)]
pub struct Account {
    pub sequence_number: u64,
}

impl Account {
    pub fn new(sequence_number: u64) -> Self {
        Self { sequence_number }
    }

    pub fn account_object_id(account: AccountAddress) -> ObjectID {
        ObjectID::from(account)
    }
}

impl MoveStructType for Account {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("Account");

    fn struct_tag() -> StructTag {
        StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![],
        }
    }
}

impl MoveStructState for Account {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::U64,
        ])
    }
}
