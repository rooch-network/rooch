// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::moveos_std::object_id;
use crate::moveos_std::object_id::ObjectID;
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

pub const MODULE_NAME: &IdentStr = ident_str!("move_module");

/// `MoveModule` is represented `moveos_std::move_module::MoveModule` in Move.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MoveModule {
    pub byte_codes: Vec<u8>,
}

impl MoveModule {
    pub fn new(byte_codes: Vec<u8>) -> Self {
        Self { byte_codes }
    }
}

impl MoveStructType for MoveModule {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("MoveModule");
}

impl MoveStructState for MoveModule {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8))])
    }
}

// pub const MODULE_NAME: &IdentStr = ident_str!("resource");

#[derive(Eq, PartialEq, Debug, Clone, Default, Deserialize, Serialize, Hash)]
pub struct Module {}

impl Module {
    pub fn module_object_id() -> ObjectID {
        object_id::named_object_id(&Self::struct_tag())
    }
}

impl MoveStructType for Module {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("Module");
}

impl MoveStructState for Module {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![])
    }
}
