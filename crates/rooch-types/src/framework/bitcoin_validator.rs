// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{addresses::ROOCH_FRAMEWORK_ADDRESS, crypto::BuiltinScheme};
use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, value::MoveValue,
};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    state::MoveStructType,
    transaction::FunctionCall,
    tx_context::TxContext,
};

pub struct BitcoinValidator {}

impl BitcoinValidator {
    pub fn scheme() -> BuiltinScheme {
        BuiltinScheme::Ecdsa
    }
}

impl MoveStructType for BitcoinValidator {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = BitcoinValidatorModule::MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("BitcoinValidator");
}

/// Rust bindings for RoochFramework bitcoin_validator module
pub struct BitcoinValidatorModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> BitcoinValidatorModule<'a> {
    const VALIDATE_FUNCTION_NAME: &'static IdentStr = ident_str!("validate");

    pub fn validate(&self, ctx: &TxContext, payload: Vec<u8>) -> Result<()> {
        let auth_validator_call = FunctionCall::new(
            Self::function_id(Self::VALIDATE_FUNCTION_NAME),
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
}

impl<'a> ModuleBinding<'a> for BitcoinValidatorModule<'a> {
    const MODULE_NAME: &'static IdentStr = ident_str!("bitcoin_validator");
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
