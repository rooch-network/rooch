// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::value::MoveValue;
use moveos_types::{
    module_binding::MoveFunctionCaller, move_types::FunctionId, transaction::FunctionCall,
    tx_context::TxContext,
};
use rooch_types::framework::auth_validator::AuthValidator;

use super::transaction_validator::TransactionValidator;

/// Rust bindings for developer custom auth validator module
/// Because the module is not in RoochFramework, we need to dynamically determine the module id base on the AuthValidator struct
pub struct AuthValidatorCaller<'a> {
    caller: &'a dyn MoveFunctionCaller,
    auth_validator: AuthValidator,
}

impl<'a> AuthValidatorCaller<'a> {
    pub fn new(caller: &'a dyn MoveFunctionCaller, auth_validator: AuthValidator) -> Self {
        Self {
            caller,
            auth_validator,
        }
    }

    pub fn validate(&self, ctx: &TxContext, payload: Vec<u8>) -> Result<()> {
        let auth_validator_call = FunctionCall::new(
            self.auth_validator.validator_function_id(),
            vec![],
            vec![MoveValue::vector_u8(payload).simple_serialize().unwrap()],
        );
        self.caller
            .call_function(ctx, auth_validator_call)
            .map(|values| {
                debug_assert!(values.is_empty(), "should not have return values");
            })?;
        Ok(())
    }

    pub fn pre_execute_function_id(&self) -> FunctionId {
        FunctionId::new(
            self.auth_validator.validator_module_id(),
            TransactionValidator::PRE_EXECUTE_FUNCTION_NAME.to_owned(),
        )
    }

    pub fn pre_execute_function_call(&self) -> FunctionCall {
        FunctionCall::new(self.pre_execute_function_id(), vec![], vec![])
    }

    pub fn post_execute_function_id(&self) -> FunctionId {
        FunctionId::new(
            self.auth_validator.validator_module_id(),
            TransactionValidator::POST_EXECUTE_FUNCTION_NAME.to_owned(),
        )
    }

    pub fn post_execute_function_call(&self) -> FunctionCall {
        FunctionCall::new(self.post_execute_function_id(), vec![], vec![])
    }
}
