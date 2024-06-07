// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use move_core_types::language_storage::StructTag;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::module_binding::{ModuleBinding, MoveFunctionCaller};
use moveos_types::moveos_std::object::{self, ObjectID};
use moveos_types::state::PlaceholderStruct;

use super::coin_store::CoinStore;

pub const MODULE_NAME: &IdentStr = ident_str!("account_coin_store");

/// Rust bindings for RoochFramework account_coin_store module
pub struct AccountCoinStoreModule<'a> {
    _caller: &'a dyn MoveFunctionCaller,
}

impl<'a> AccountCoinStoreModule<'a> {
    pub const COIN_STORE_ID: &'static IdentStr = ident_str!("coin_store_id");

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
        Self { _caller: caller }
    }
}
