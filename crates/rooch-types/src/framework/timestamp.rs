// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, value::MoveValue,
};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    moveos_std::tx_context::TxContext,
    state::{MoveStructState, MoveStructType},
    transaction::{FunctionCall, MoveAction},
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("timestamp");

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Timestamp {
    pub microseconds: u64,
}

impl MoveStructType for Timestamp {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("Timestamp");
}

impl MoveStructState for Timestamp {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::U64,
        ])
    }
}

/// Rust bindings for RoochFramework timestamp module
pub struct TimestampModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> TimestampModule<'a> {
    pub const NOW_MICROSECONDS_FUNCTION_NAME: &'static IdentStr = ident_str!("now_milliseconds");
    pub const NOW_SECONDS_FUNCTION_NAME: &'static IdentStr = ident_str!("now_seconds");
    pub const FAST_FORWARD_SECONDS_FOR_LOCAL_FUNCTION_NAME: &'static IdentStr =
        ident_str!("fast_forward_seconds_for_local");

    pub fn now_milliseconds(&self) -> Result<u64> {
        let call = FunctionCall::new(
            Self::function_id(Self::NOW_MICROSECONDS_FUNCTION_NAME),
            vec![],
            vec![],
        );
        let ctx = TxContext::zero();
        let session_key =
            self.caller
                .call_function(&ctx, call)?
                .into_result()
                .map(|mut values| {
                    let value = values.pop().expect("should have one return value");
                    bcs::from_bytes::<u64>(&value.value).expect("should be a valid u64")
                })?;
        Ok(session_key)
    }

    pub fn now_seconds(&self) -> Result<u64> {
        let call = FunctionCall::new(
            Self::function_id(Self::NOW_SECONDS_FUNCTION_NAME),
            vec![],
            vec![],
        );
        let ctx = TxContext::zero();
        let session_key =
            self.caller
                .call_function(&ctx, call)?
                .into_result()
                .map(|mut values| {
                    let value = values.pop().expect("should have one return value");
                    bcs::from_bytes::<u64>(&value.value).expect("should be a valid u64")
                })?;
        Ok(session_key)
    }

    pub fn create_fast_forward_seconds_for_local_action(seconds: u64) -> MoveAction {
        MoveAction::Function(FunctionCall::new(
            Self::function_id(Self::FAST_FORWARD_SECONDS_FOR_LOCAL_FUNCTION_NAME),
            vec![],
            vec![MoveValue::U64(seconds).simple_serialize().unwrap()],
        ))
    }
}

impl<'a> ModuleBinding<'a> for TimestampModule<'a> {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
