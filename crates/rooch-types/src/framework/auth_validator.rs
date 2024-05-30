// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

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
use moveos_types::{
    module_binding::MoveFunctionCaller,
    move_std::string::MoveString,
    move_types::FunctionId,
    moveos_std::tx_context::TxContext,
    state::{MoveStructState, MoveStructType},
    transaction::FunctionCall,
};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

pub const MODULE_NAME: &IdentStr = ident_str!("auth_validator");

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
    Bitcoin,
}

impl BuiltinAuthValidator {
    const ROOCH_FLAG: u8 = 0x00;
    const ETHEREUM_FLAG: u8 = 0x01;
    const BITCOIN_FLAG: u8 = 0x02;

    pub fn flag(&self) -> u8 {
        match self {
            BuiltinAuthValidator::Rooch => Self::ROOCH_FLAG,
            BuiltinAuthValidator::Ethereum => Self::ETHEREUM_FLAG,
            BuiltinAuthValidator::Bitcoin => Self::BITCOIN_FLAG,
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
            Self::ETHEREUM_FLAG => Ok(BuiltinAuthValidator::Ethereum),
            Self::BITCOIN_FLAG => Ok(BuiltinAuthValidator::Bitcoin),
            _ => Err(RoochError::KeyConversionError(
                "Invalid key auth validator".to_owned(),
            )),
        }
    }

    pub fn auth_validator(&self) -> AuthValidator {
        match self {
            BuiltinAuthValidator::Rooch => AuthValidator {
                id: self.flag().into(),
                module_address: ROOCH_FRAMEWORK_ADDRESS,
                module_name: MoveString::from_str("native_validator").expect("Should be valid"),
            },
            BuiltinAuthValidator::Ethereum => AuthValidator {
                id: self.flag().into(),
                module_address: ROOCH_FRAMEWORK_ADDRESS,
                module_name: MoveString::from_str("ethereum_validator").expect("Should be valid"),
            },
            BuiltinAuthValidator::Bitcoin => AuthValidator {
                id: self.flag().into(),
                module_address: ROOCH_FRAMEWORK_ADDRESS,
                module_name: MoveString::from_str("bitcoin_validator").expect("Should be valid"),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthValidator {
    pub id: u64,
    pub module_address: AccountAddress,
    pub module_name: MoveString,
}

impl MoveStructType for AuthValidator {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
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
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
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
