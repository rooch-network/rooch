// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use anyhow::ensure;
use move_core_types::language_storage::{StructTag, TypeTag};
use move_core_types::u256::U256;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::state::{MoveState, MoveStructState, MoveStructType, PlaceholderStruct, State};
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
pub struct CoinStore<CoinType = PlaceholderStruct> {
    balance: Balance,
    frozen: bool,
    phantom: std::marker::PhantomData<CoinType>,
}

impl CoinStore {
    pub fn struct_tag_without_coin_type() -> StructTag {
        move_core_types::language_storage::StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![],
        }
    }
}

impl<CoinType> MoveStructType for CoinStore<CoinType>
where
    CoinType: MoveStructType,
{
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("CoinStore");

    fn type_params() -> Vec<move_core_types::language_storage::TypeTag> {
        vec![CoinType::struct_tag().into()]
    }
}

impl<CoinType> MoveStructState for CoinStore<CoinType>
where
    CoinType: MoveStructType,
{
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            Balance::type_layout(),
            move_core_types::value::MoveTypeLayout::Bool,
        ])
    }
}

impl<CoinType> CoinStore<CoinType>
where
    CoinType: MoveStructType,
{
    pub fn struct_tag_with_coin_type(coin_type: StructTag) -> StructTag {
        move_core_types::language_storage::StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![coin_type.into()],
        }
    }
}

impl<CoinType> CoinStore<CoinType>
where
    CoinType: MoveStructType,
{
    pub fn new(balance: U256, frozen: bool) -> Self {
        Self {
            balance: Balance { value: balance },
            frozen,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn coin_type_str(&self) -> String {
        self.coin_type().to_string()
    }
    pub fn coin_type(&self) -> StructTag {
        CoinType::struct_tag()
    }
    pub fn balance(&self) -> U256 {
        self.balance.value
    }
    pub fn frozen(&self) -> bool {
        self.frozen
    }
}

/// The CoinStore without generic type
pub struct CoinStoreInfo {
    pub coin_type: StructTag,
    pub balance: Balance,
    pub frozen: bool,
}

impl CoinStoreInfo {
    pub fn new(coin_type: StructTag, balance: Balance, frozen: bool) -> Self {
        Self {
            coin_type,
            balance,
            frozen,
        }
    }

    pub fn coin_type(&self) -> StructTag {
        self.coin_type.clone()
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

impl TryFrom<State> for CoinStoreInfo {
    type Error = anyhow::Error;

    fn try_from(state: State) -> Result<Self, Self::Error> {
        let raw_object = state.as_raw_object()?;
        ensure!(
            CoinStore::<PlaceholderStruct>::struct_tag_match_without_type_param(
                &raw_object.value.struct_tag
            ),
            "Expected CoinStore struct tag"
        );
        ensure!(
            raw_object.value.struct_tag.type_params.len() == 1,
            "Expected CoinStore type params length to be 1"
        );
        let coin_type = raw_object.value.struct_tag.type_params[0].clone();
        let coin_type = match coin_type {
            TypeTag::Struct(coin_type) => *coin_type,
            _ => return Err(anyhow::anyhow!("Invalid CoinType TypeTag")),
        };
        let coin_store = CoinStore::<PlaceholderStruct>::from_bytes(raw_object.value.value)?;
        Ok(Self::new(coin_type, coin_store.balance, coin_store.frozen))
    }
}
