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

/// `MoveModuleUpgradeFlag` is represented `moveos_std::move_module::MoveModuleUpgradeFlag` in Move.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MoveModuleUpgradeFlag {
    pub is_upgrade: bool,
}

impl MoveModuleUpgradeFlag {
    pub fn new(is_upgrade: bool) -> Self {
        Self { is_upgrade }
    }
}

impl MoveStructType for MoveModuleUpgradeFlag {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("account_storage");
    const STRUCT_NAME: &'static IdentStr = ident_str!("ModuleUpgradeFlag");
}

impl MoveStructState for MoveModuleUpgradeFlag {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Bool])
    }
}
