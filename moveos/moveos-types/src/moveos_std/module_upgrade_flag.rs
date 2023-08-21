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

/// `ModuleUpgradeFlag` is represented `moveos_std::move_module::ModuleUpgradeFlag` in Move.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ModuleUpgradeFlag {
    pub is_upgrade: bool,
}

impl ModuleUpgradeFlag {
    pub fn new(is_upgrade: bool) -> Self {
        Self { is_upgrade }
    }
}

impl MoveStructType for ModuleUpgradeFlag {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("account_storage");
    const STRUCT_NAME: &'static IdentStr = ident_str!("ModuleUpgradeFlag");
}

impl MoveStructState for ModuleUpgradeFlag {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Bool])
    }
}
