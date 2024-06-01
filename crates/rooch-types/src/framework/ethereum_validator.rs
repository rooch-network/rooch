// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::auth_validator::BuiltinAuthValidator;
use anyhow::Result;
use framework_types::addresses::ROOCH_NURSERY_ADDRESS;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, value::MoveValue,
};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    moveos_std::tx_context::TxContext,
    state::MoveStructType,
    transaction::FunctionCall,
};

pub struct EthereumValidator {}

impl EthereumValidator {
    pub fn auth_validator_id() -> u64 {
        BuiltinAuthValidator::Ethereum.flag().into()
    }
}

impl MoveStructType for EthereumValidator {
    const ADDRESS: AccountAddress = ROOCH_NURSERY_ADDRESS;
    const MODULE_NAME: &'static IdentStr = EthereumValidatorModule::MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("EthereumValidator");
}

/// Rust bindings for RoochFramework ethereum_validator module
pub struct EthereumValidatorModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> EthereumValidatorModule<'a> {
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

impl<'a> ModuleBinding<'a> for EthereumValidatorModule<'a> {
    const MODULE_NAME: &'static IdentStr = ident_str!("ethereum_validator");
    const MODULE_ADDRESS: AccountAddress = ROOCH_NURSERY_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
