// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::framework::auth_validator::BuiltinAuthValidator;
use anyhow::Result;
use framework_types::addresses::BITCOIN_MOVE_ADDRESS;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, value::MoveValue,
};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    moveos_std::tx_context::TxContext,
    state::MoveStructType,
    transaction::FunctionCall,
};

const MODULE_NAME: &IdentStr = ident_str!("bitcoin_multisign_validator");

/// Bitcoin Multisign Auth Validator
pub struct BitcoinMultisignValidator {}

impl BitcoinMultisignValidator {
    pub fn auth_validator_id() -> u64 {
        BuiltinAuthValidator::BitcoinMultisign.flag().into()
    }
}

impl MoveStructType for BitcoinMultisignValidator {
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("BitcoinMultisignValidator");
}

/// Rust bindings for RoochFramework bitcoin_validator module
pub struct BitcoinMultisignValidatorModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> BitcoinMultisignValidatorModule<'a> {
    const VALIDATE_FUNCTION_NAME: &'static IdentStr = ident_str!("validate");

    pub fn validate(&self, ctx: &TxContext, payload: Vec<u8>) -> Result<()> {
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

impl<'a> ModuleBinding<'a> for BitcoinMultisignValidatorModule<'a> {
    const MODULE_ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
