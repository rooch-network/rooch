// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::moveos_std::object_id::ObjectID;
use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    state::{MoveStructState, MoveStructType},
};
use move_core_types::language_storage::StructTag;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, value::MoveStructLayout,
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("resource");

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct Resource {}

impl Resource {
    pub fn resource_object_id(account: AccountAddress) -> ObjectID {
        ObjectID::from(account)
    }
}

impl MoveStructType for Resource {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("Resource");

    fn struct_tag() -> StructTag {
        StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![],
        }
    }
}

impl MoveStructState for Resource {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![])
    }
}
