// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use anyhow::{Ok, Result};
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::object::ObjectID;
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    move_option::MoveOption,
    state::MoveStructState,
    transaction::FunctionCall,
    tx_context::TxContext,
};

/// Rust bindings for RoochFramework coin module
pub struct CoinModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> CoinModule<'a> {
    const COIN_STORE_HANDLE_FUNCTION_NAME: &'static IdentStr = ident_str!("coin_store_handle");

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
