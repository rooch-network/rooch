// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::BITCOIN_MOVE_ADDRESS;
use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, u256::U256, value::MoveValue,
};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    move_std::{option::MoveOption, string::MoveString},
    moveos_std::{
        object::{self, ObjectID},
        simple_map::SimpleMap,
        tx_context::TxContext,
    },
    state::{MoveState, MoveStructState, MoveStructType},
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("brc20");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BRC20CoinInfo {
    pub tick: MoveString,
    pub max: U256,
    pub lim: U256,
    pub dec: u64,
    pub supply: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BRC20Store {
    pub coins: ObjectID,
}

impl BRC20Store {
    pub fn object_id() -> ObjectID {
        object::named_object_id(&Self::struct_tag())
    }
}

impl MoveStructType for BRC20Store {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("BRC20Store");
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
}

impl MoveStructState for BRC20Store {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![ObjectID::type_layout()])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Op {
    pub json_map: SimpleMap<MoveString, MoveString>,
}

impl MoveStructType for Op {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("Op");
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
}

impl MoveStructState for Op {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            SimpleMap::<MoveString, MoveString>::type_layout(),
        ])
    }
}

/// Rust bindings for BitcoinMove brc20 module
pub struct BRC20Module<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> BRC20Module<'a> {
    pub const GET_TICK_INFO_FUNCTION_NAME: &'static IdentStr = ident_str!("get_tick_info");
    pub const GET_BALANCE_FUNCTION_NAME: &'static IdentStr = ident_str!("get_balance");

    pub fn get_tick_info(&self, tick: String) -> Result<Option<BRC20CoinInfo>> {
        let call = Self::create_function_call(
            Self::GET_TICK_INFO_FUNCTION_NAME,
            vec![],
            vec![
                MoveValue::Address(BRC20Store::object_id().into()),
                MoveValue::vector_u8(MoveString::from(tick).to_bytes()),
            ],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ONE);
        let result = self
            .caller
            .call_function(&ctx, call)?
            .into_result()
            .map(|mut values| {
                let value = values.pop().expect("should have one return value");
                bcs::from_bytes::<MoveOption<BRC20CoinInfo>>(&value.value)
                    .expect("should be a valid MoveOption<BRC20CoinInfo>")
            })?;
        Ok(result.into())
    }

    pub fn get_balance(&self, tick: String, addr: AccountAddress) -> Result<U256> {
        let call = Self::create_function_call(
            Self::GET_BALANCE_FUNCTION_NAME,
            vec![],
            vec![
                MoveValue::Address(BRC20Store::object_id().into()),
                MoveValue::vector_u8(MoveString::from(tick).to_bytes()),
                MoveValue::Address(addr),
            ],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ONE);
        let result = self
            .caller
            .call_function(&ctx, call)?
            .into_result()
            .map(|mut values| {
                let value = values.pop().expect("should have one return value");
                bcs::from_bytes::<U256>(&value.value).expect("should be a valid u256")
            })?;
        Ok(result)
    }
}

impl<'a> ModuleBinding<'a> for BRC20Module<'a> {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const MODULE_ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
