// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use anyhow::{bail, Ok, Result};
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
            .map(|values| {
                let value = values
                    .get(0)
                    .expect("Coin store handle expected return value");
                let result = MoveOption::<ObjectID>::from_bytes(&value.value)
                    .expect("Coin store handle expected Option<ObjectID>");
                result.into()
            })?;
        Ok(result)
    }

    pub fn coin_info_handle(&self) -> Result<Option<ObjectID>> {
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
            .map(|values| {
                let value = values
                    .get(0)
                    .expect("Coin info handle expected return value");
                let result = MoveOption::<ObjectID>::from_bytes(&value.value)
                    .expect("Coin info handle expected Option<ObjectID>");
                result.into()
            })?;
        Ok(result)
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
    pub struct_type: StructTag,
    pub value: Coin,
}

impl AnnotatedCoin {
    pub fn new(struct_type: StructTag, value: Coin) -> Self {
        AnnotatedCoin { struct_type, value }
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
    pub struct_type: StructTag,
    pub value: CompoundCoinStore,
}

impl AnnotatedCoinStore {
    pub fn new(struct_type: StructTag, value: CompoundCoinStore) -> Self {
        AnnotatedCoinStore { struct_type, value }
    }

    /// Create a new AnnotatedCoinStore from a AnnotatedMoveValue
    pub fn new_from_annotated_move_value(annotated_move_value: AnnotatedMoveValue) -> Result<Self> {
        match annotated_move_value {
            AnnotatedMoveValue::Struct(annotated_struct) => {
                let annotated_coin_store_type = annotated_struct.type_;
                let mut fields = annotated_struct.value.into_iter();
                let annotated_coin = match fields.next().expect("CoinStore should have coin field")
                {
                    (field_name, AnnotatedMoveValue::Struct(field_value)) => {
                        debug_assert!(
                            field_name.as_str() == "coin",
                            "CoinStore coin field name should be coin"
                        );
                        let coin_struct_type = field_value.type_;

                        let mut inner_fields = field_value.value.into_iter();
                        let coin_value = match inner_fields
                            .next()
                            .expect("CoinValue should have value field")
                        {
                            (field_name, AnnotatedMoveValue::U256(inner_field_value)) => {
                                debug_assert!(
                                    field_name.as_str() == "value",
                                    "CoinValue value field name should be value"
                                );
                                inner_field_value
                            }
                            _ => bail!("CoinValue value field should be value"),
                        };
                        let coin = Coin { value: coin_value };
                        AnnotatedCoin {
                            struct_type: coin_struct_type,
                            value: coin,
                        }
                    }
                    _ => bail!("CoinStore coin field should be struct"),
                };
                let frozen = match fields.next().expect("CoinStore should have frozen field") {
                    (field_name, AnnotatedMoveValue::Bool(field_value)) => {
                        debug_assert!(
                            field_name.as_str() == "frozen",
                            "CoinStore field name should be frozen"
                        );
                        field_value
                    }
                    _ => bail!("CoinStore frozen field should be bool"),
                };
                let compose_coin_store = CompoundCoinStore {
                    coin: annotated_coin,
                    frozen,
                };

                let annotated_coin_store = AnnotatedCoinStore {
                    struct_type: annotated_coin_store_type,
                    value: compose_coin_store,
                };

                Ok(annotated_coin_store)
            }
            _ => bail!("CoinValue value field should be value"),
        }
    }

    pub fn get_coin_struct_type(&self) -> StructTag {
        self.value.coin.struct_type.clone()
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
    pub struct_type: StructTag,
    pub value: CoinInfo,
}

impl AnnotatedCoinInfo {
    pub fn new(struct_type: StructTag, value: CoinInfo) -> Self {
        AnnotatedCoinInfo { struct_type, value }
    }

    /// Create a new AnnotatedCoinInfo from a AnnotatedMoveValue
    pub fn new_from_annotated_move_value(annotated_move_value: AnnotatedMoveValue) -> Result<Self> {
        match annotated_move_value {
            AnnotatedMoveValue::Struct(annotated_struct) => {
                let struct_type = annotated_struct.type_;
                let mut fields = annotated_struct.value.into_iter();

                let name = match fields.next().expect("CoinInfo should have name field") {
                    (_field_name, AnnotatedMoveValue::Struct(field_value)) => {
                        let mut inner_fields = field_value.value.into_iter();
                        match inner_fields
                            .next()
                            .expect("CoinInfo name struct should have field")
                        {
                            (field_name, AnnotatedMoveValue::Bytes(field_value)) => {
                                debug_assert!(
                                    field_name.as_str() == "bytes",
                                    "CoinInfo inner field name should be bytes"
                                );
                                String::from_utf8(field_value)?
                            }
                            _ => bail!("CoinInfo name field should be Bytes"),
                        }
                    }
                    _ => bail!("CoinInfo name field should be String"),
                };
                let symbol = match fields.next().expect("CoinInfo should have symbol field") {
                    (_field_name, AnnotatedMoveValue::Struct(field_value)) => {
                        let mut inner_fields = field_value.value.into_iter();
                        match inner_fields
                            .next()
                            .expect("CoinInfo symbol struct should have field")
                        {
                            (field_name, AnnotatedMoveValue::Bytes(field_value)) => {
                                debug_assert!(
                                    field_name.as_str() == "bytes",
                                    "CoinInfo field symbol should be symbol"
                                );
                                String::from_utf8(field_value)?
                            }
                            _ => bail!("CoinInfo symbol field should be Bytes"),
                        }
                    }
                    _ => bail!("CoinInfo symbol field should be String"),
                };
                let decimals = match fields.next().expect("CoinInfo should have decimals field") {
                    (field_name, AnnotatedMoveValue::U8(field_value)) => {
                        debug_assert!(
                            field_name.as_str() == "decimals",
                            "CoinInfo field decimals should be decimals"
                        );
                        field_value
                    }
                    _ => bail!("CoinInfo decimals field should be u8"),
                };
                let supply = match fields.next().expect("CoinInfo should have supply field") {
                    (field_name, AnnotatedMoveValue::U256(field_value)) => {
                        debug_assert!(
                            field_name.as_str() == "supply",
                            "CoinInfo field supply should be supply"
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
                    struct_type,
                    value: coin_info,
                };

                Ok(annotated_coin_info)
            }
            _ => bail!("CoinInfo value field should be struct"),
        }
    }

    pub fn get_struct_type(&self) -> StructTag {
        self.struct_type.clone()
    }

    pub fn get_decimals(&self) -> u8 {
        self.value.decimals
    }
}
