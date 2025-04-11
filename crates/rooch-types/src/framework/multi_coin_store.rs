// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use crate::framework::coin_store::CoinStoreInfo;
use anyhow::ensure;
use move_core_types::language_storage::StructTag;
use move_core_types::u256::U256;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::move_std::string::MoveString;
use moveos_types::state::{MoveState, MoveStructState, MoveStructType, ObjectState};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub const MODULE_NAME: &IdentStr = ident_str!("multi_coin_store");

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
pub struct MultiCoinStore {}

impl MoveStructType for MultiCoinStore {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("MultiCoinStore");
}

impl MoveStructState for MultiCoinStore {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinStoreField {
    coin_type: MoveString,
    balance: Balance,
    frozen: bool,
}

impl MoveStructType for CoinStoreField {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("CoinStoreField");
}

impl MoveStructState for CoinStoreField {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            MoveString::type_layout(),
            Balance::type_layout(),
            move_core_types::value::MoveTypeLayout::Bool,
        ])
    }
}

impl CoinStoreField {
    pub fn new(coin_type: MoveString, balance: U256, frozen: bool) -> Self {
        Self {
            coin_type,
            balance: Balance { value: balance },
            frozen,
        }
    }

    pub fn coin_type_str(&self) -> String {
        self.coin_type.to_string()
    }
    pub fn balance(&self) -> U256 {
        self.balance.value
    }
    pub fn frozen(&self) -> bool {
        self.frozen
    }
}

impl TryFrom<ObjectState> for CoinStoreField {
    type Error = anyhow::Error;

    fn try_from(state: ObjectState) -> Result<Self, Self::Error> {
        let raw_object = state.into_raw_object()?;
        ensure!(
            CoinStoreField::struct_tag_match_without_type_param(&raw_object.value.struct_tag),
            "Expected CoinStoreField struct tag"
        );
        let coin_store_field = CoinStoreField::from_bytes(raw_object.value.value)?;
        Ok(coin_store_field)
    }
}

impl TryFrom<CoinStoreField> for CoinStoreInfo {
    type Error = anyhow::Error;

    fn try_from(field: CoinStoreField) -> Result<Self, Self::Error> {
        let coin_type = StructTag::from_str(field.coin_type.as_str())
            .map_err(|_| anyhow::anyhow!("Invalid coin type string"))?;
        Ok(Self::new(coin_type, field.balance.value, field.frozen))
    }
}
