// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use anyhow::Result;
use move_core_types::language_storage::StructTag;
use move_core_types::u256::U256;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::move_std::string::MoveString;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::state::{MoveStructState, MoveStructType};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    moveos_std::tx_context::TxContext,
    transaction::FunctionCall,
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("coin");

pub const DEFAULT_DECIMALS: u8 = 9;

/// Rust bindings for RoochFramework coin module
pub struct CoinModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> CoinModule<'a> {
    pub const COIN_INFOS_HANDLE_FUNCTION_NAME: &'static IdentStr = ident_str!("coin_infos_handle");

    pub fn coin_infos_handle(&self) -> Result<ObjectID> {
        let ctx = TxContext::zero();
        let call = FunctionCall::new(
            Self::function_id(Self::COIN_INFOS_HANDLE_FUNCTION_NAME),
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
pub struct CoinInfo {
    coin_type: MoveString,
    name: MoveString,
    symbol: MoveString,
    decimals: u8,
    supply: U256,
}

impl MoveStructType for CoinInfo {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("CoinInfo");

    fn struct_tag() -> move_core_types::language_storage::StructTag {
        move_core_types::language_storage::StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![],
        }
    }
}

impl MoveStructState for CoinInfo {
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

impl CoinInfo {
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
