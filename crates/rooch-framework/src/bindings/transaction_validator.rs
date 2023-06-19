// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::ROOCH_FRAMEWORK_ADDRESS;
use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, value::MoveValue,
};
use moveos_types::{
    module_binding::{ModuleBundle, MoveFunctionCaller},
    move_types::FunctionId,
    transaction::FunctionCall,
    tx_context::TxContext,
};
use rooch_types::transaction::AuthenticatorInfo;

/// Rust bindings for RoochFramework transaction_validator module
pub struct TransactionValidator<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> TransactionValidator<'a> {
    const VALIDATE_FUNCTION_NAME: &'static IdentStr = ident_str!("validate");
    const PRE_EXECUTE_FUNCTION_NAME: &IdentStr = ident_str!("pre_execute");
    const POST_EXECUTE_FUNCTION_NAME: &IdentStr = ident_str!("post_execute");

    pub fn validate(&self, ctx: &TxContext, auth: AuthenticatorInfo) -> Result<()> {
        let call = FunctionCall::new(
            Self::function_id(Self::VALIDATE_FUNCTION_NAME),
            vec![],
            vec![MoveValue::vector_u8(
                bcs::to_bytes(&auth).expect("serialize authenticator should success"),
            )
            .simple_serialize()
            .expect("serialize authenticator should success")],
        );
        self.caller.call_function(ctx, call).map(|values| {
            debug_assert!(values.is_empty(), "Expected no return value");
        })
    }

    pub fn pre_execute_function_id() -> FunctionId {
        Self::function_id(Self::PRE_EXECUTE_FUNCTION_NAME)
    }

    pub fn post_execute_function_id() -> FunctionId {
        Self::function_id(Self::POST_EXECUTE_FUNCTION_NAME)
    }
}

impl<'a> ModuleBundle<'a> for TransactionValidator<'a> {
    const MODULE_NAME: &'static IdentStr = ident_str!("transaction_validator");
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
