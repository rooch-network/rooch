// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    value::{MoveStructLayout, MoveTypeLayout},
};
use moveos_types::{
    moveos_std::object::{self, ObjectID},
    state::{MoveStructState, MoveStructType},
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("onchain_config");

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct OnchainConfig {
    pub framework_version: u64,
    pub sequencer: AccountAddress,
}

impl MoveStructType for OnchainConfig {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("OnchainConfig");
}

impl MoveStructState for OnchainConfig {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::U64, MoveTypeLayout::Address])
    }
}

impl OnchainConfig {
    pub fn get_onchain_config_object_id() -> ObjectID {
        object::named_object_id(&Self::struct_tag())
    }
}
