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
    move_std::ascii::MoveAsciiString,
    moveos_std::object::{self, ObjectID},
    state::{MoveState, MoveStructState, MoveStructType},
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("onchain_config");

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct GasEntry {
    pub key: MoveAsciiString,
    pub val: u64,
}

impl MoveStructType for GasEntry {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("GasEntry");
}

impl MoveStructState for GasEntry {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveAsciiString::type_layout(), MoveTypeLayout::U64])
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct GasScheduleConfig {
    pub entries: Vec<GasEntry>,
}

impl MoveStructType for GasScheduleConfig {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("GasScheduleConfig");
}

impl MoveStructState for GasScheduleConfig {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Vector(Box::new(
            GasEntry::type_layout(),
        ))])
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct GasSchedule {
    pub schedule_version: u64,
    pub entries: Vec<GasEntry>,
}

impl GasSchedule {
    pub fn gas_schedule_object_id() -> ObjectID {
        object::named_object_id(&Self::struct_tag())
    }
}

impl MoveStructType for GasSchedule {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("GasSchedule");
}

impl MoveStructState for GasSchedule {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::U64,
            MoveTypeLayout::Vector(Box::new(GasEntry::type_layout())),
        ])
    }
}
