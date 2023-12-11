// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::ord::InscriptionStore;
use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, value::MoveValue,
};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    moveos_std::{
        object::{self, ObjectID},
        tx_context::TxContext,
    },
    state::{MoveState, MoveStructState, MoveStructType},
    transaction::FunctionCall,
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("brc20");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BRC20Store {
    /// The latest inscription index has been processed
    pub next_inscription_index: u64,
    /// The coins id
    pub coins: ObjectID,
    /// The balance id
    pub balance: ObjectID,
}

impl BRC20Store {
    pub fn object_id() -> ObjectID {
        object::named_object_id(&Self::struct_tag())
    }
}

impl MoveStructType for BRC20Store {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("BRC20Store");
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
}

impl MoveStructState for BRC20Store {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            u64::type_layout(),
            ObjectID::type_layout(),
            ObjectID::type_layout(),
        ])
    }
}

/// Rust bindings for RoochFramework brc20 module
pub struct BRC20Module<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> BRC20Module<'a> {
    pub const REMAINING_INSCRIPTION_COUNT_FUNCTION_NAME: &'static IdentStr =
        ident_str!("remaining_inscription_count");
    pub const PROGRESS_BRC20_OPS_ENTRY_FUNCTION_NAME: &'static IdentStr =
        ident_str!("progress_brc20_ops");

    pub fn remaining_inscription_count(&self) -> Result<u64> {
        let call = Self::create_function_call(
            Self::REMAINING_INSCRIPTION_COUNT_FUNCTION_NAME,
            vec![],
            vec![
                MoveValue::Address(InscriptionStore::object_id().into()),
                MoveValue::Address(BRC20Store::object_id().into()),
            ],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        let remaining_count =
            self.caller
                .call_function(&ctx, call)?
                .into_result()
                .map(|mut values| {
                    let value = values.pop().expect("should have one return value");
                    bcs::from_bytes::<u64>(&value.value).expect("should be a valid bool")
                })?;
        Ok(remaining_count)
    }

    pub fn create_progress_brc20_ops_call(batch_size: u64) -> FunctionCall {
        Self::create_function_call(
            Self::PROGRESS_BRC20_OPS_ENTRY_FUNCTION_NAME,
            vec![],
            vec![
                MoveValue::Address(InscriptionStore::object_id().into()),
                MoveValue::Address(BRC20Store::object_id().into()),
                MoveValue::U64(batch_size),
            ],
        )
    }
}

impl<'a> ModuleBinding<'a> for BRC20Module<'a> {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
