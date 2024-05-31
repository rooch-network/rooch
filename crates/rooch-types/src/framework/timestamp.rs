// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use framework_types::addresses::ROOCH_FRAMEWORK_ADDRESS;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, value::MoveValue,
};
use moveos_types::module_binding::{ModuleBinding, MoveFunctionCaller};
use moveos_types::transaction::{FunctionCall, MoveAction};

pub const MODULE_NAME: &IdentStr = ident_str!("timestamp");

/// Rust bindings for RoochFramework timestamp module
#[allow(dead_code)]
pub struct TimestampModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> TimestampModule<'a> {
    pub const FAST_FORWARD_SECONDS_FOR_LOCAL_FUNCTION_NAME: &'static IdentStr =
        ident_str!("fast_forward_seconds_for_local");

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
