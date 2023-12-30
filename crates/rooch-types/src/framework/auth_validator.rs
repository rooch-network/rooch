// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::ethereum_validator::EthereumValidatorModule;
use super::native_validator::NativeValidatorModule;
use super::transaction_validator::TransactionValidator;
use crate::address::MultiChainAddress;
use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use crate::error::RoochError;
use anyhow::Result;
use clap::ValueEnum;
use move_core_types::value::MoveValue;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, language_storage::ModuleId,
};
use moveos_types::function_return_value::DecodedFunctionResult;
use moveos_types::move_std::option::MoveOption;
use moveos_types::transaction::MoveAction;
use moveos_types::{
    module_binding::MoveFunctionCaller,
    move_std::ascii::MoveAsciiString,
    move_types::FunctionId,
    moveos_std::tx_context::TxContext,
    state::{MoveStructState, MoveStructType},
    transaction::FunctionCall,
};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

/// The Authenticator auth validator which has builtin Rooch and Ethereum
#[derive(
    Copy,
    Clone,
    Debug,
    EnumString,
    PartialEq,
    Eq,
    ValueEnum,
    Display,
    Ord,
    PartialOrd,
    Serialize,
    Deserialize,
)]
#[strum(serialize_all = "lowercase")]
pub enum BuiltinAuthValidator {
    Rooch,
    Ethereum,
}

impl BuiltinAuthValidator {
    const ROOCH_FLAG: u8 = 0x00;
    const ETHEREUM_FLAG: u8 = 0x01;

    pub fn flag(&self) -> u8 {
        match self {
            BuiltinAuthValidator::Rooch => Self::ROOCH_FLAG,
            BuiltinAuthValidator::Ethereum => Self::ETHEREUM_FLAG,
        }
    }

    pub fn from_flag(flag: &str) -> Result<BuiltinAuthValidator, RoochError> {
        let byte_int = flag
            .parse::<u8>()
            .map_err(|_| RoochError::KeyConversionError("Invalid key auth validator".to_owned()))?;
        Self::from_flag_byte(byte_int)
    }

    pub fn from_flag_byte(byte_int: u8) -> Result<BuiltinAuthValidator, RoochError> {
        match byte_int {
            Self::ROOCH_FLAG => Ok(BuiltinAuthValidator::Rooch),
            _ => Err(RoochError::KeyConversionError(
                "Invalid key auth validator".to_owned(),
            )),
        }
    }

    pub fn create_rotate_authentication_key_action(
        &self,
        public_key: Vec<u8>,
    ) -> Result<MoveAction, RoochError> {
        let action = match self {
            BuiltinAuthValidator::Rooch => {
                NativeValidatorModule::rotate_authentication_key_action(public_key)
            }
            BuiltinAuthValidator::Ethereum => {
                EthereumValidatorModule::rotate_authentication_key_action(public_key)
            }
        };
        Ok(action)
    }

    pub fn create_remove_authentication_key_action(&self) -> Result<MoveAction, RoochError> {
        let action = match self {
            BuiltinAuthValidator::Rooch => {
                NativeValidatorModule::remove_authentication_key_action()
            }
            BuiltinAuthValidator::Ethereum => {
                EthereumValidatorModule::remove_authentication_key_action()
            }
        };
        Ok(action)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthValidator {
    pub id: u64,
    pub module_address: AccountAddress,
    pub module_name: MoveAsciiString,
}

impl MoveStructType for AuthValidator {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("auth_validator");
    const STRUCT_NAME: &'static IdentStr = ident_str!("AuthValidator");
}

impl MoveStructState for AuthValidator {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::U64,
            move_core_types::value::MoveTypeLayout::Address,
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                move_core_types::value::MoveTypeLayout::U8,
            )),
        ])
    }
}

impl AuthValidator {
    pub const VALIDATE_FUNCTION_NAME: &'static IdentStr = ident_str!("validate");

    pub fn validator_module_id(&self) -> ModuleId {
        ModuleId::new(
            self.module_address,
            self.module_name
                .clone()
                .try_into()
                .expect("Invalid module name"),
        )
    }

    pub fn validator_function_id(&self) -> FunctionId {
        FunctionId::new(
            self.validator_module_id(),
            Self::VALIDATE_FUNCTION_NAME.to_owned(),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxValidateResult {
    pub auth_validator_id: u64,
    pub auth_validator: MoveOption<AuthValidator>,
    pub session_key: MoveOption<Vec<u8>>,
}

impl MoveStructType for TxValidateResult {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("transaction_validtor");
    const STRUCT_NAME: &'static IdentStr = ident_str!("TxValidateResult");
}

impl MoveStructState for TxValidateResult {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::U64,
            move_core_types::value::MoveTypeLayout::Struct(
                MoveOption::<AuthValidator>::struct_layout(),
            ),
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                move_core_types::value::MoveTypeLayout::U8,
            )),
        ])
    }
}

impl TxValidateResult {
    pub fn auth_validator(&self) -> Option<AuthValidator> {
        self.auth_validator.clone().into()
    }

    pub fn session_key(&self) -> Option<Vec<u8>> {
        self.session_key.clone().into()
    }

    pub fn is_validate_via_session_key(&self) -> bool {
        self.session_key().is_some()
    }
}

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

    pub fn validate(
        &self,
        ctx: &TxContext,
        payload: Vec<u8>,
    ) -> Result<DecodedFunctionResult<Option<MultiChainAddress>>> {
        let auth_validator_call = FunctionCall::new(
            self.auth_validator.validator_function_id(),
            vec![],
            vec![MoveValue::vector_u8(payload).simple_serialize().unwrap()],
        );
        self.caller
            .call_function(ctx, auth_validator_call)?
            .decode(|mut values| {
                // TODO: all validate must return value ?
                let value = values.pop();

                Ok(
                    value.and_then(|v| match bcs::from_bytes::<MultiChainAddress>(&v.value) {
                        Ok(result) => Some(result),
                        Err(_) => None,
                    }),
                )
            })
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
