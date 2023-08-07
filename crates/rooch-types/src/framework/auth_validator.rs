// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, language_storage::ModuleId,
};
use moveos_types::move_option::MoveOption;
use moveos_types::{
    move_string::MoveAsciiString,
    move_types::FunctionId,
    state::{MoveStructState, MoveStructType},
};
use serde::{Deserialize, Serialize};

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
