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

pub struct Ed25519Validator {}

impl Ed25519Validator {
    pub fn scheme() -> BuiltinScheme {
        BuiltinScheme::Ed25519
    }
}

impl MoveStructType for Ed25519Validator {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = Ed25519ValidatorModule::MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("Ed25519Validator");
}

/// Rust bindings for RoochFramework ed25519_validator module
pub struct Ed25519ValidatorModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> Ed25519ValidatorModule<'a> {
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

impl<'a> ModuleBinding<'a> for Ed25519ValidatorModule<'a> {
    const MODULE_NAME: &'static IdentStr = ident_str!("ed25519_validator");
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
