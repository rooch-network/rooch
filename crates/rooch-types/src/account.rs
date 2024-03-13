// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, value::MoveValue,
};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    transaction::MoveAction,
};

/// Rust bindings for RoochFramework account module
#[allow(dead_code)]
pub struct AccountModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> AccountModule<'a> {
    const CREATE_ACCOUNT_ENTRY_FUNCTION_NAME: &'static IdentStr =
        ident_str!("create_account_entry");

    pub fn create_account_action(address: AccountAddress) -> MoveAction {
        Self::create_move_action(
            Self::CREATE_ACCOUNT_ENTRY_FUNCTION_NAME,
            vec![],
            vec![MoveValue::Address(address)],
        )
    }
}

impl<'a> ModuleBinding<'a> for AccountModule<'a> {
    const MODULE_NAME: &'static IdentStr = ident_str!("account");
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
