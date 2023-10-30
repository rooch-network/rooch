// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::object::{NamedTableID, ObjectID};
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

pub const MODULE_NAME: &IdentStr = ident_str!("account_storage");

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct AccountStorage {
    pub resources: ObjectID,
    pub modules: ObjectID,
}

impl AccountStorage {
    pub fn new(account: AccountAddress) -> Self {
        let resources = NamedTableID::Resource(account).to_object_id();
        let modules = NamedTableID::Module(account).to_object_id();
        AccountStorage { resources, modules }
    }
}

impl MoveStructType for AccountStorage {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("AccountStorage");
}

impl MoveStructState for AccountStorage {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::Struct(ObjectID::struct_layout()),
            MoveTypeLayout::Struct(ObjectID::struct_layout()),
        ])
    }
}
