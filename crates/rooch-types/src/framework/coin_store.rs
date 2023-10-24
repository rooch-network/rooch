// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use move_core_types::language_storage::StructTag;
use move_core_types::u256::U256;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::move_std::string::MoveString;
use moveos_types::state::{MoveStructState, MoveStructType};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("coin_store");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    value: U256,
}

impl MoveStructType for Balance {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("Balance");

    fn struct_tag() -> move_core_types::language_storage::StructTag {
        move_core_types::language_storage::StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![],
        }
    }
}

impl MoveStructState for Balance {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::U256,
        ])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinStore {
    coin_type: MoveString,
    balance: Balance,
    frozen: bool,
}

impl MoveStructType for CoinStore {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("CoinStore");

    fn struct_tag() -> move_core_types::language_storage::StructTag {
        move_core_types::language_storage::StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![],
        }
    }
}

impl MoveStructState for CoinStore {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            MoveString::type_layout(),
            Balance::type_layout(),
            move_core_types::value::MoveTypeLayout::Bool,
        ])
    }
}

impl CoinStore {
    pub fn coin_type(&self) -> String {
        self.coin_type.to_string()
    }
    pub fn coin_type_tag(&self) -> StructTag {
        let coin_type_str = format!("0x{}", self.coin_type);
        coin_type_str
            .parse::<StructTag>()
            .expect("CoinType in CoinStore should be valid StructTag")
    }
    pub fn balance(&self) -> U256 {
        self.balance.value
    }
    pub fn frozen(&self) -> bool {
        self.frozen
    }
}
