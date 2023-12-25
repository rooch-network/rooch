// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::types::Transaction;
use crate::addresses::BITCOIN_MOVE_ADDRESS;
use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, value::MoveValue,
};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    move_std::string::MoveString,
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
pub struct BRC20Store {
    /// The latest inscription index has been processed
    pub next_inscription_index: u64,
    /// The coins id
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
        move_core_types::value::MoveStructLayout::new(vec![
            u64::type_layout(),
            ObjectID::type_layout(),
        ])
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
    pub const FROM_TRANSACTION_FUNCTION_NAME: &'static IdentStr =
        ident_str!("from_transaction_bytes");

    pub fn from_transaction(&self, tx: &Transaction) -> Result<Vec<Op>> {
        let call = Self::create_function_call(
            Self::FROM_TRANSACTION_FUNCTION_NAME,
            vec![],
            vec![MoveValue::vector_u8(tx.to_bytes())],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ONE);
        let ops = self
            .caller
            .call_function(&ctx, call)?
            .into_result()
            .map(|mut values| {
                let value = values.pop().expect("should have one return value");
                bcs::from_bytes::<Vec<Op>>(&value.value).expect("should be a valid Vec<Op>")
            })?;
        Ok(ops)
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
