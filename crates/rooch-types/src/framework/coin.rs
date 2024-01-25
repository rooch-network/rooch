// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use move_core_types::language_storage::StructTag;
use move_core_types::u256::U256;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::module_binding::{ModuleBinding, MoveFunctionCaller};
use moveos_types::move_std::string::MoveString;
use moveos_types::moveos_std::object_id::{self, ObjectID};
use moveos_types::state::{MoveState, MoveStructState, MoveStructType, PlaceholderStruct};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("coin");

pub const DEFAULT_DECIMALS: u8 = 9;

/// Rust bindings for RoochFramework coin module
pub struct CoinModule<'a> {
    //avoid #[warn(dead_code)] warning
    //TODO change this to private after we use the caller
    pub caller: &'a dyn MoveFunctionCaller,
}

impl<'a> CoinModule<'a> {
    pub fn coin_info_id(coin_type: StructTag) -> ObjectID {
        let coin_info_struct_tag =
            CoinInfo::<PlaceholderStruct>::struct_tag_with_coin_type(coin_type);
        object_id::named_object_id(&coin_info_struct_tag)
    }
}

impl<'a> ModuleBinding<'a> for CoinModule<'a> {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coin<T> {
    pub value: U256,
    pub phantom: std::marker::PhantomData<T>,
}

impl<T> Coin<T> {
    pub fn new(value: U256) -> Self {
        Coin {
            value,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T> MoveStructType for Coin<T>
where
    T: MoveStructType,
{
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("Coin");

    fn struct_tag() -> move_core_types::language_storage::StructTag {
        move_core_types::language_storage::StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![T::struct_tag().into()],
        }
    }
}

impl<T> MoveStructState for Coin<T>
where
    T: MoveStructType,
{
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::U256,
        ])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinInfo<CoinType> {
    coin_type: MoveString,
    name: MoveString,
    symbol: MoveString,
    decimals: u8,
    supply: U256,
    phantom: std::marker::PhantomData<CoinType>,
}

impl<CoinType> MoveStructType for CoinInfo<CoinType>
where
    CoinType: MoveStructType,
{
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("CoinInfo");

    fn struct_tag() -> move_core_types::language_storage::StructTag {
        move_core_types::language_storage::StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![CoinType::struct_tag().into()],
        }
    }
}

impl<CoinType> MoveStructState for CoinInfo<CoinType>
where
    CoinType: MoveStructType,
{
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            MoveString::type_layout(),
            MoveString::type_layout(),
            MoveString::type_layout(),
            move_core_types::value::MoveTypeLayout::U8,
            move_core_types::value::MoveTypeLayout::U256,
        ])
    }
}

impl<CoinType> CoinInfo<CoinType>
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
impl<CoinType> CoinInfo<CoinType> {
    pub fn coin_type(&self) -> String {
        self.coin_type.to_string()
    }
    pub fn coin_type_tag(&self) -> StructTag {
        let coin_type_str = format!("0x{}", self.coin_type);
        coin_type_str
            .parse::<StructTag>()
            .expect("CoinType in CoinInfo should be valid StructTag")
    }
    pub fn name(&self) -> String {
        self.name.to_string()
    }
    pub fn symbol(&self) -> String {
        self.symbol.to_string()
    }
    pub fn decimals(&self) -> u8 {
        self.decimals
    }
    pub fn supply(&self) -> U256 {
        self.supply
    }
}
