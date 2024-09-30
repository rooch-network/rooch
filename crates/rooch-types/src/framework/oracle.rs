// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, u256::U256, value::MoveValue,
};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    move_std::string::MoveString,
    moveos_std::object::ObjectID,
    state::{MoveState, MoveStructState, MoveStructType},
    transaction::MoveAction,
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("oracle");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewOracleEvent {
    pub name: MoveString,
    pub oracle_id: ObjectID,
    pub admin_id: ObjectID,
}

impl MoveStructType for NewOracleEvent {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("NewOracleEvent");
}

impl MoveStructState for NewOracleEvent {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            MoveString::type_layout(),
            ObjectID::type_layout(),
            ObjectID::type_layout(),
        ])
    }
}

/// Rust bindings for RoochFramework oracle module
pub struct OracleModule;

impl OracleModule {
    pub const CREATE_ENTRY_FUNCTION_NAME: &'static IdentStr = ident_str!("create_entry");
    pub const SUBMIT_DECIMAL_DATA_FUNCTION_NAME: &'static IdentStr =
        ident_str!("submit_decimal_data");

    pub fn create_oracle_action(name: String, url: String, description: String) -> MoveAction {
        Self::create_move_action(
            Self::CREATE_ENTRY_FUNCTION_NAME,
            vec![],
            vec![
                MoveString::from(name).to_move_value(),
                MoveString::from(url).to_move_value(),
                MoveString::from(description).to_move_value(),
            ],
        )
    }

    pub fn submit_decimal_data_action(
        oracle_id: ObjectID,
        ticker: String,
        value: U256,
        decimal: u8,
        identifier: String,
        admin_obj: ObjectID,
    ) -> MoveAction {
        Self::create_move_action(
            Self::SUBMIT_DECIMAL_DATA_FUNCTION_NAME,
            vec![],
            vec![
                oracle_id.to_move_value(),
                MoveString::from(ticker).to_move_value(),
                MoveValue::U256(value),
                MoveValue::U8(decimal),
                MoveString::from(identifier).to_move_value(),
                admin_obj.to_move_value(),
            ],
        )
    }
}

impl<'a> ModuleBinding<'a> for OracleModule {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(_caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self
    }
}
