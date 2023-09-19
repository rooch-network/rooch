// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use anyhow::{bail, Result};
use move_core_types::language_storage::StructTag;
use move_core_types::u256::U256;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use move_resource_viewer::AnnotatedMoveValue;
use moveos_types::object::ObjectID;
use moveos_types::state::MoveState;
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    move_option::MoveOption,
    transaction::FunctionCall,
    tx_context::TxContext,
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

pub const DEFAULT_DECIMALS: u8 = 9;

/// Rust bindings for RoochFramework coin module
pub struct CoinModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> CoinModule<'a> {
    pub const COIN_STORE_HANDLE_FUNCTION_NAME: &'static IdentStr = ident_str!("coin_store_handle");
    pub const COIN_INFO_HANDLE_FUNCTION_NAME: &'static IdentStr = ident_str!("coin_info_handle");

    pub fn coin_store_handle(&self, addr: AccountAddress) -> Result<Option<ObjectID>> {
        let ctx = TxContext::zero();
        let call = FunctionCall::new(
            Self::function_id(Self::COIN_STORE_HANDLE_FUNCTION_NAME),
            vec![],
            vec![addr.to_vec()],
        );
        let result = self
            .caller
            .call_function(&ctx, call)?
            .into_result()
            .map_err(|e| anyhow::anyhow!("Call coin store handle error:{}", e))?;
        let object_id = match result.get(0) {
            Some(value) => {
                let object_id_result = MoveOption::<ObjectID>::from_bytes(&value.value);
                Option::<ObjectID>::from(object_id_result?)
            }
            None => None,
        };
        Ok(object_id)
    }

    pub fn coin_info_handle(&self) -> Result<ObjectID> {
        let ctx = TxContext::zero();
        let call = FunctionCall::new(
            Self::function_id(Self::COIN_INFO_HANDLE_FUNCTION_NAME),
            vec![],
            vec![],
        );

        let result = self
            .caller
            .call_function(&ctx, call)?
            .into_result()
            .map_err(|e| anyhow::anyhow!("Call coin info handle error:{}", e))?;
        match result.get(0) {
            Some(value) => Ok(bcs::from_bytes::<ObjectID>(&value.value)?),
            None => Err(anyhow::anyhow!("Coin info handle should have value")),
        }
    }
}

impl<'a> ModuleBinding<'a> for CoinModule<'a> {
    const MODULE_NAME: &'static IdentStr = ident_str!("coin");
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coin {
    pub value: U256,
}

impl Coin {
    pub fn new(value: U256) -> Self {
        Coin { value }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotatedCoin {
    #[serde(rename = "type")]
    pub type_: StructTag,
    pub value: Coin,
}

impl AnnotatedCoin {
    pub fn new(type_: StructTag, value: Coin) -> Self {
        AnnotatedCoin { type_, value }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundCoinStore {
    pub coin: AnnotatedCoin,
    pub frozen: bool,
}

impl CompoundCoinStore {
    pub fn new(coin: AnnotatedCoin, frozen: bool) -> Self {
        CompoundCoinStore { coin, frozen }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotatedCoinStore {
    #[serde(rename = "type")]
    pub type_: StructTag,
    pub value: CompoundCoinStore,
}

impl AnnotatedCoinStore {
    pub fn new(type_: StructTag, value: CompoundCoinStore) -> Self {
        AnnotatedCoinStore { type_, value }
    }

    /// Create a new AnnotatedCoinStore from a AnnotatedMoveValue
    pub fn new_from_annotated_move_value(annotated_move_value: AnnotatedMoveValue) -> Result<Self> {
        match annotated_move_value {
            AnnotatedMoveValue::Struct(annotated_struct) => {
                let annotated_coin_store_type = annotated_struct.type_;
                let mut fields = annotated_struct.value.into_iter();
                let annotated_coin = match fields
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("CoinStore should have coin field"))?
                {
                    (field_name, AnnotatedMoveValue::Struct(field_value)) => {
                        debug_assert!(
                            field_name.as_str() == "coin",
                            "CoinStore field name should be coin"
                        );
                        let coin_type = field_value.type_;

                        let mut inner_fields = field_value.value.into_iter();
                        let coin_value = match inner_fields
                            .next()
                            .ok_or_else(|| anyhow::anyhow!("CoinValue coin should have value"))?
                        {
                            (field_name, AnnotatedMoveValue::U256(inner_field_value)) => {
                                debug_assert!(
                                    field_name.as_str() == "value",
                                    "CoinValue coin field name should be value"
                                );
                                inner_field_value
                            }
                            _ => bail!("CoinValue coin value field type should be U256"),
                        };
                        let coin = Coin { value: coin_value };
                        AnnotatedCoin {
                            type_: coin_type,
                            value: coin,
                        }
                    }
                    _ => bail!("CoinStore coin field type should be Struct"),
                };
                let frozen = match fields
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("CoinStore should have frozen field"))?
                {
                    (field_name, AnnotatedMoveValue::Bool(field_value)) => {
                        debug_assert!(
                            field_name.as_str() == "frozen",
                            "CoinStore field name should be frozen"
                        );
                        field_value
                    }
                    _ => bail!("CoinStore frozen field type should be Bool"),
                };
                let compose_coin_store = CompoundCoinStore {
                    coin: annotated_coin,
                    frozen,
                };

                let annotated_coin_store = AnnotatedCoinStore {
                    type_: annotated_coin_store_type,
                    value: compose_coin_store,
                };

                Ok(annotated_coin_store)
            }
            _ => bail!("CoinStore move value type should be Struct"),
        }
    }

    pub fn get_coin_type(&self) -> StructTag {
        self.value.coin.type_.clone()
    }

    pub fn get_coin_value(&self) -> U256 {
        self.value.coin.value.value
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub supply: U256,
}
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct CoinInfo<T> {
//     pub name: String,
//     pub symbol: String,
//     pub decimals: u8,
//     pub supply: U256,
//     pub coin_type: std::marker::PhantomData<T>,
// }
//
// impl<T> MoveStructType for CoinInfo<T>
//     where T: MoveStructType, {
//     const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
//     const MODULE_NAME: &'static IdentStr = ident_str!("coin");
//     const STRUCT_NAME: &'static IdentStr = ident_str!("CoinInfo");
//
//     fn struct_tag() -> move_core_types::language_storage::StructTag {
//         move_core_types::language_storage::StructTag {
//             address: Self::ADDRESS,
//             module: Self::MODULE_NAME.to_owned(),
//             name: Self::STRUCT_NAME.to_owned(),
//             type_params: vec![T::type_tag()],
//         }
//     }
// }
//
// impl<T> MoveStructState for CoinInfo<T> where T: MoveStructType, {
//     fn struct_layout() -> move_core_types::value::MoveStructLayout {
//         move_core_types::value::MoveStructLayout::new(vec![
//             move_core_types::value::MoveTypeLayout::Vector(Box::new(
//                 move_core_types::value::MoveTypeLayout::U8,
//             )),
//             move_core_types::value::MoveTypeLayout::Vector(Box::new(
//                 move_core_types::value::MoveTypeLayout::U8,
//             )),
//             move_core_types::value::MoveTypeLayout::U8,
//             move_core_types::value::MoveTypeLayout::U256,
//         ])
//     }
// }

impl CoinInfo {
    pub fn new(name: String, symbol: String, decimals: u8, supply: U256) -> Self {
        CoinInfo {
            name,
            symbol,
            decimals,
            supply,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotatedCoinInfo {
    #[serde(rename = "type")]
    pub type_: StructTag,
    pub value: CoinInfo,
}

impl AnnotatedCoinInfo {
    pub fn new(type_: StructTag, value: CoinInfo) -> Self {
        AnnotatedCoinInfo { type_, value }
    }

    /// Create a new AnnotatedCoinInfo from a AnnotatedMoveValue
    pub fn new_from_annotated_move_value(annotated_move_value: AnnotatedMoveValue) -> Result<Self> {
        match annotated_move_value {
            AnnotatedMoveValue::Struct(annotated_struct) => {
                let type_ = annotated_struct.type_;
                let mut fields = annotated_struct.value.into_iter();

                let name = match fields
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("CoinInfo should have name field"))?
                {
                    (_field_name, AnnotatedMoveValue::Struct(field_value)) => {
                        let mut inner_fields = field_value.value.into_iter();
                        match inner_fields.next().ok_or_else(|| {
                            anyhow::anyhow!("CoinInfo name field should have value")
                        })? {
                            (field_name, AnnotatedMoveValue::Bytes(field_value)) => {
                                debug_assert!(
                                    field_name.as_str() == "bytes",
                                    "CoinInfo name inner field name should be Bytes"
                                );
                                String::from_utf8(field_value)?
                            }
                            _ => bail!("CoinInfo name inner field type should be Bytes"),
                        }
                    }
                    _ => bail!("CoinInfo name field type should be Struct"),
                };
                let symbol = match fields
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("CoinInfo should have symbol field"))?
                {
                    (_field_name, AnnotatedMoveValue::Struct(field_value)) => {
                        let mut inner_fields = field_value.value.into_iter();
                        match inner_fields.next().ok_or_else(|| {
                            anyhow::anyhow!("CoinInfo symbol struct should have value")
                        })? {
                            (field_name, AnnotatedMoveValue::Bytes(field_value)) => {
                                debug_assert!(
                                    field_name.as_str() == "bytes",
                                    "CoinInfo symbol inner field name should be bytes"
                                );
                                String::from_utf8(field_value)?
                            }
                            _ => bail!("CoinInfo symbol inner field type should be Bytes"),
                        }
                    }
                    _ => bail!("CoinInfo symbol field type should be Struct"),
                };
                let decimals = match fields
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("CoinInfo should have decimals field"))?
                {
                    (field_name, AnnotatedMoveValue::U8(field_value)) => {
                        debug_assert!(
                            field_name.as_str() == "decimals",
                            "CoinInfo field name should be decimals"
                        );
                        field_value
                    }
                    _ => bail!("CoinInfo decimals field type should be U8"),
                };
                let supply = match fields
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("CoinInfo should have supply field"))?
                {
                    (field_name, AnnotatedMoveValue::U256(field_value)) => {
                        debug_assert!(
                            field_name.as_str() == "supply",
                            "CoinInfo field name should be supply"
                        );
                        field_value
                    }
                    _ => bail!("CoinInfo supply field should be U256"),
                };

                let coin_info = CoinInfo {
                    name,
                    symbol,
                    decimals,
                    supply,
                };

                let annotated_coin_info = AnnotatedCoinInfo {
                    type_,
                    value: coin_info,
                };

                Ok(annotated_coin_info)
            }
            _ => bail!("CoinInfo move value type should be Struct"),
        }
    }

    pub fn get_type(&self) -> StructTag {
        self.type_.clone()
    }

    pub fn get_decimals(&self) -> u8 {
        self.value.decimals
    }
}
