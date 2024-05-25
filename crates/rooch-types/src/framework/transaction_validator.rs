// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, value::MoveValue,
};

use moveos_types::{
    function_return_value::DecodedFunctionResult,
    module_binding::{ModuleBinding, MoveFunctionCaller},
    move_types::FunctionId,
    moveos_std::tx_context::TxContext,
    transaction::FunctionCall,
};

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use crate::framework::auth_validator::TxValidateResult;
use crate::transaction::AuthenticatorInfo;

/// Rust bindings for RoochFramework transaction_validator module
pub struct TransactionValidator<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> TransactionValidator<'a> {
    pub const VALIDATE_FUNCTION_NAME: &'static IdentStr = ident_str!("validate");
    pub const PRE_EXECUTE_FUNCTION_NAME: &'static IdentStr = ident_str!("pre_execute");
    pub const POST_EXECUTE_FUNCTION_NAME: &'static IdentStr = ident_str!("post_execute");

    pub fn validate(
        &self,
        ctx: &TxContext,
        auth: AuthenticatorInfo,
    ) -> Result<DecodedFunctionResult<TxValidateResult>> {
        let tx_validator_call = FunctionCall::new(
            Self::function_id(Self::VALIDATE_FUNCTION_NAME),
            vec![],
            vec![
                MoveValue::U64(auth.chain_id).simple_serialize().unwrap(),
                MoveValue::U64(auth.authenticator.auth_validator_id)
                    .simple_serialize()
                    .unwrap(),
                MoveValue::vector_u8(auth.authenticator.payload)
                    .simple_serialize()
                    .unwrap(),
            ],
        );
        let function_result =
            self.caller
                .call_function(ctx, tx_validator_call)?
                .decode(|mut values| {
                    let value = values.pop().expect("should have one return value");
                    let result = bcs::from_bytes::<TxValidateResult>(&value.value)?;
                    Ok(result)
                })?;
        Ok(function_result)
    }

    pub fn pre_execute_function_id() -> FunctionId {
        Self::function_id(Self::PRE_EXECUTE_FUNCTION_NAME)
    }

    pub fn pre_execute_function_call() -> FunctionCall {
        FunctionCall::new(Self::pre_execute_function_id(), vec![], vec![])
    }

    pub fn post_execute_function_id() -> FunctionId {
        Self::function_id(Self::POST_EXECUTE_FUNCTION_NAME)
    }

    pub fn post_execute_function_call() -> FunctionCall {
        FunctionCall::new(Self::post_execute_function_id(), vec![], vec![])
    }
}

impl<'a> ModuleBinding<'a> for TransactionValidator<'a> {
    const MODULE_NAME: &'static IdentStr = ident_str!("transaction_validator");
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
