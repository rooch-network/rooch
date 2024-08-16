// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::auth_validator::BuiltinAuthValidator;
use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::state::MoveStructType;

const MODULE_NAME: &IdentStr = ident_str!("bitcoin_validator");

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
