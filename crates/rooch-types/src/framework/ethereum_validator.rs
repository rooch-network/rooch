// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{addresses::ROOCH_FRAMEWORK_ADDRESS, multichain_id::RoochMultiChainID};
use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, value::MoveValue,
};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    state::MoveStructType,
    transaction::{FunctionCall, MoveAction},
    tx_context::TxContext,
};

pub struct EthereumValidator {}

impl EthereumValidator {
    pub fn multichain_id() -> RoochMultiChainID {
        RoochMultiChainID::Ether
    }
}

impl MoveStructType for EthereumValidator {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = EthereumValidatorModule::MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("EthereumValidator");
}

/// Rust bindings for RoochFramework ethereum_validator module
pub struct EthereumValidatorModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> EthereumValidatorModule<'a> {
    const VALIDATE_FUNCTION_NAME: &'static IdentStr = ident_str!("validate");
    const ROTATE_AUTHENTICATION_KEY_ENTRY_FUNCTION_NAME: &'static IdentStr =
        ident_str!("rotate_authentication_key_entry");
    const REMOVE_AUTHENTICATION_KEY_ENTRY_FUNCTION_NAME: &'static IdentStr =
        ident_str!("remove_authentication_key_entry");

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

    pub fn rotate_authentication_key_action(public_key: Vec<u8>) -> MoveAction {
        Self::create_move_action(
            Self::ROTATE_AUTHENTICATION_KEY_ENTRY_FUNCTION_NAME,
            vec![],
            vec![MoveValue::vector_u8(public_key)],
        )
    }

    pub fn remove_authentication_key_action() -> MoveAction {
        Self::create_move_action(
            Self::REMOVE_AUTHENTICATION_KEY_ENTRY_FUNCTION_NAME,
            vec![],
            vec![],
        )
    }
}

impl<'a> ModuleBinding<'a> for EthereumValidatorModule<'a> {
    const MODULE_NAME: &'static IdentStr = ident_str!("ethereum_validator");
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
