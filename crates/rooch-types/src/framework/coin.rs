// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use anyhow::{bail, Ok, Result};
use move_core_types::language_storage::StructTag;
use move_core_types::u256::U256;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use move_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue};
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

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coin {
    value: U256,
}

impl Coin {
    pub fn new(value: U256) -> Self {
        Coin { value }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotatedCoin {
    type_: StructTag,
    value: Coin,
}

impl AnnotatedCoin {
    pub fn new(type_: StructTag, value: Coin) -> Self {
        AnnotatedCoin { type_, value }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundCoinStore {
    coin: AnnotatedCoin,
    frozen: bool,
}

impl CompoundCoinStore {
    pub fn new(coin: AnnotatedCoin, frozen: bool) -> Self {
        CompoundCoinStore { coin, frozen }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotatedCoinStore {
    type_: StructTag,
    value: CompoundCoinStore,
}

impl AnnotatedCoinStore {
    pub fn new(type_: StructTag, value: CompoundCoinStore) -> Self {
        AnnotatedCoinStore { type_, value }
    }

    /// Create a new AnnotatedCoinStore from a AnnotatedMoveStruct
    pub fn new_from_annotated_struct(annotated_struct: AnnotatedMoveStruct) -> Result<Self> {
        let annotated_coin_store_type = annotated_struct.type_;
        //         "move_value": {
        //         "abilities": 8,
        //         "type": "0x3::coin::CoinStore<0xe1176537c0175d336353dad12f7eb60c658ce526eeb3cd08409e6fd8c2dfa1d7::fixed_supply_coin::FSC>",
        //         "value": {
        //             "coin": {
        //                 "abilities": 4,
        //                 "type": "0x3::coin::Coin<0xe1176537c0175d336353dad12f7eb60c658ce526eeb3cd08409e6fd8c2dfa1d7::fixed_supply_coin::FSC>",
        //                 "value": {
        //                     "value": "10000"
        //                 }
        //             },
        //             "frozen": false
        //         }
        //     }
        //     },

        let mut fields = annotated_struct.value.into_iter();
        // let object_id = ObjectID::try_from(fields.next().expect("Object should have id").1)?;
        let annotated_coin = match fields.next().expect("CoinStore should have coin field") {
            (field_name, AnnotatedMoveValue::Struct(filed_value)) => {
                debug_assert!(
                    field_name.as_str() == "coin",
                    "CoinStore coin field name should be coin"
                );

                let coin_type_ = filed_value.type_;
                let mut inner_fields = filed_value.value.into_iter();
                let coin_value = match inner_fields
                    .next()
                    .expect("CoinValue should have value field")
                {
                    (field_name, AnnotatedMoveValue::Bytes(inner_filed_value)) => {
                        debug_assert!(
                            field_name.as_str() == "value",
                            "CoinValue value field name should be value"
                        );
                        // String::from_utf8(inner_filed_value)
                        U256::from_bytes(inner_filed_value.as_slice())
                    }
                    _ => bail!("CoinValue value field should be value"),
                }?;

                let coin = Coin { value: coin_value };
                let annotated_coin = AnnotatedCoin {
                    type_: coin_type_,
                    value: coin,
                };
                annotated_coin
            }
            _ => bail!("CoinStore coin field should be struct"),
        };
        let frozen = match fields.next().expect("CoinStore should have frozen field") {
            (field_name, AnnotatedMoveValue::Bool(filed_value)) => {
                debug_assert!(
                    field_name.as_str() == "frozen",
                    "CoinStore field name should be frozen"
                );
                filed_value
            }
            _ => bail!("CoinStore frozen field should be bool"),
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

    pub fn get_coin_type(&self) -> StructTag {
        self.value.coin.type_.clone()
    }

    pub fn get_coin_value(&self) -> U256 {
        self.value.coin.value.value.clone()
    }
}

/// Rust bindings for RoochFramework coin module
pub struct CoinModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> CoinModule<'a> {
    pub const COIN_STORE_HANDLE_FUNCTION_NAME: &'static IdentStr = ident_str!("coin_store_handle");

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
                let value = values.get(0).expect("Expected return value");
                let result = MoveOption::<ObjectID>::from_bytes(&value.value)
                    .expect("Expected Option<ObjectID>");
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
