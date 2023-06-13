// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::ROOCH_FRAMEWORK_ADDRESS;
use anyhow::Result;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::{
    module_binding::{ModuleBundle, MoveFunctionCaller},
    move_option::MoveOption,
    state::MoveStructState,
    transaction::FunctionCall,
    tx_context::TxContext,
};
use rooch_types::address::{MultiChainAddress, RoochAddress};

/// Rust bindings for RoochFramework address_mapping module
pub struct AddressMapping<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> AddressMapping<'a> {
    const RESOLVE_FUNCTION_NAME: &'static IdentStr = ident_str!("resolve");

    pub fn resolve(&self, multichain_address: MultiChainAddress) -> Result<Option<AccountAddress>> {
        if multichain_address.is_rooch_address() {
            let rooch_address: RoochAddress = multichain_address.try_into()?;
            Ok(Some(rooch_address.into()))
        } else {
            let ctx = TxContext::zero();
            let call = FunctionCall::new(
                Self::function_id(Self::RESOLVE_FUNCTION_NAME),
                vec![],
                vec![multichain_address.to_bytes()],
            );
            self.caller.call_function(&ctx, call).map(|values| {
                let value = values.get(0).expect("Expected return value");
                let result = MoveOption::<AccountAddress>::from_bytes(&value.value)
                    .expect("Expected Option<address>");
                result.into()
            })
        }
    }
}

impl<'a> ModuleBundle<'a> for AddressMapping<'a> {
    const MODULE_NAME: &'static IdentStr = ident_str!("address_mapping");
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
