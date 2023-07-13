// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, language_storage::ModuleId,
};
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
    const MODULE_NAME: &'static IdentStr = ident_str!("auth_validator_registry");
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
