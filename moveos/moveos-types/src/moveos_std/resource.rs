// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    state::{MoveStructState, MoveStructType},
};
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, value::MoveStructLayout,
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("resource");

#[derive(Eq, PartialEq, Debug, Default, Clone, Deserialize, Serialize, Hash)]
pub struct Resource {}

impl MoveStructType for Resource {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("Resource");
}

impl MoveStructState for Resource {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![])
    }
}
