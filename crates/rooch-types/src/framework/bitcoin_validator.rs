// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::auth_validator::BuiltinAuthValidator;
use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, value::MoveValue,
};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    moveos_std::tx_context::TxContext,
    state::MoveStructType,
    transaction::{FunctionCall, MoveAction},
};

const MODULE_NAME: &'static IdentStr = ident_str!("bitcoin_validator");

/// Bitcoin Auth Validator
pub struct BitcoinValidator {}

impl BitcoinValidator {
    pub fn auth_validator_id() -> u64 {
        BuiltinAuthValidator::Bitcoin.flag().into()
    }
}

impl MoveStructType for BitcoinValidator {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("BitcoinValidator");
}

/// Rust bindings for RoochFramework bitcoin_validator module
pub struct BitcoinValidatorModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> BitcoinValidatorModule<'a> {
    const VALIDATE_FUNCTION_NAME: &'static IdentStr = ident_str!("validate");

    fn validate(&self, ctx: &TxContext, payload: Vec<u8>) -> Result<()> {
        let auth_validator_call = FunctionCall::new(
            Self::function_id(Self::VALIDATE_FUNCTION_NAME),
            vec![],
            vec![MoveValue::vector_u8(payload).simple_serialize().unwrap()],
        );
        self.caller
            .call_function(ctx, auth_validator_call)?
            .into_result()
            .map(|values| {
                debug_assert!(values.is_empty(), "should not have return values");
            })?;
        Ok(())
    }

}

impl<'a> ModuleBinding<'a> for NativeValidatorModule<'a> {
    
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
