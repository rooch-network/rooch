// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::MOVEOS_STD_ADDRESS;
use crate::{
    move_std::ascii::MoveAsciiString,
    moveos_std::object::{self, ObjectID},
    state::{MoveState, MoveStructState, MoveStructType},
};
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    value::{MoveStructLayout, MoveTypeLayout},
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("gas_schedule");

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct GasEntry {
    pub key: MoveAsciiString,
    pub val: u64,
}

impl MoveStructType for GasEntry {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
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
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
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
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
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

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Deserialize)]
pub struct GasScheduleUpdated {
    pub last_updated: u64,
}

impl MoveStructType for GasScheduleUpdated {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("GasScheduleUpdated");
}

impl MoveStructState for GasScheduleUpdated {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::U64,
        ])
    }
}
