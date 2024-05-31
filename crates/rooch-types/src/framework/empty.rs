// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use anyhow::Result;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    move_types::FunctionId,
    moveos_std::tx_context::TxContext,
    transaction::FunctionCall,
};

/// Rust bindings for RoochFramework transaction_validator module
pub struct Empty<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> Empty<'a> {
    pub const EMPTY_FUNCTION_NAME: &'static IdentStr = ident_str!("empty");

    pub fn empty(&self, ctx: &TxContext) -> Result<()> {
        let empty_call =
            FunctionCall::new(Self::function_id(Self::EMPTY_FUNCTION_NAME), vec![], vec![]);

        self.caller
            .call_function(ctx, empty_call)?
            .into_result()
            .map(|values| {
                debug_assert!(values.is_empty(), "should not have return values");
            })?;
        Ok(())
    }

    pub fn empty_function_id() -> FunctionId {
        Self::function_id(Self::EMPTY_FUNCTION_NAME)
    }

    pub fn empty_function_call() -> FunctionCall {
        FunctionCall::new(Self::empty_function_id(), vec![], vec![])
    }
}

impl<'a> ModuleBinding<'a> for Empty<'a> {
    const MODULE_NAME: &'static IdentStr = ident_str!("empty");
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
