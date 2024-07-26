// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::MOVEOS_STD_ADDRESS;
use crate::module_binding::{ModuleBinding, MoveFunctionCaller};
use crate::moveos_std::tx_context::TxContext;
use crate::transaction::FunctionCall;
use crate::{
    move_std::string::MoveString,
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
    pub key: MoveString,
    pub val: u64,
}

impl MoveStructType for GasEntry {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("GasEntry");
}

impl MoveStructState for GasEntry {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveString::type_layout(), MoveTypeLayout::U64])
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct GasScheduleConfig {
    pub max_gas_amount: u64,
    pub entries: Vec<GasEntry>,
}

impl GasScheduleConfig {
    pub const INITIAL_MAX_GAS_AMOUNT: u64 = 1_000_000_000u64;
    /// The maximum gas amount that can be used for a read-only function call
    pub const READONLY_MAX_GAS_AMOUNT: u64 = 5_000_000_000u64;
}

impl MoveStructType for GasScheduleConfig {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("GasScheduleConfig");
}

impl MoveStructState for GasScheduleConfig {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::U64,
            MoveTypeLayout::Vector(Box::new(GasEntry::type_layout())),
        ])
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct GasSchedule {
    pub schedule_version: u64,
    pub max_gas_amount: u64,
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

pub struct GasScheduleModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> GasScheduleModule<'a> {
    pub const GAS_SCHEDULE_FUNCTION_NAME: &'static IdentStr = ident_str!("gas_schedule");

    pub fn gas_schedule(&self) -> anyhow::Result<GasSchedule> {
        let call = FunctionCall::new(
            Self::function_id(Self::GAS_SCHEDULE_FUNCTION_NAME),
            vec![],
            vec![],
        );
        let ctx = TxContext::zero();

        let gas_schedule =
            self.caller
                .call_function(&ctx, call)?
                .into_result()
                .map(|mut values| {
                    let value = values.pop().expect("should have one return value");
                    bcs::from_bytes::<GasSchedule>(&value.value)
                        .expect("should be a valid GasSchedule")
                })?;

        Ok(gas_schedule)
    }
}

impl<'a> ModuleBinding<'a> for GasScheduleModule<'a> {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const MODULE_ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
