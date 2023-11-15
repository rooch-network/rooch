// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use anyhow::Result;
use move_core_types::language_storage::StructTag;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::moveos_std::object::{self, ObjectID};
use moveos_types::state::{MoveState, PlaceholderStruct};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    move_std::option::MoveOption,
    moveos_std::tx_context::TxContext,
    transaction::FunctionCall,
};

use super::coin_store::CoinStore;

pub const MODULE_NAME: &IdentStr = ident_str!("account_coin_store");

/// Rust bindings for RoochFramework account_coin_store module
pub struct AccountCoinStoreModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> AccountCoinStoreModule<'a> {
    pub const COIN_STORES_HANDLE_FUNCTION_NAME: &'static IdentStr =
        ident_str!("coin_stores_handle");
    pub const COIN_STORE_ID: &'static IdentStr = ident_str!("coin_store_id");

    pub fn coin_stores_handle(&self, addr: AccountAddress) -> Result<Option<ObjectID>> {
        let ctx = TxContext::zero();
        let call = FunctionCall::new(
            Self::function_id(Self::COIN_STORES_HANDLE_FUNCTION_NAME),
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
                let object_id_result = MoveOption::<ObjectID>::from_bytes(&value.value)?;
                Option::<ObjectID>::from(object_id_result)
            }
            None => None,
        };
        Ok(object_id)
    }

    pub fn account_coin_store_id(addr: AccountAddress, coin_type: StructTag) -> ObjectID {
        let coin_store_struct_tag =
            CoinStore::<PlaceholderStruct>::struct_tag_with_coin_type(coin_type);
        object::account_named_object_id(addr, &coin_store_struct_tag)
    }
}

impl<'a> ModuleBinding<'a> for AccountCoinStoreModule<'a> {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
