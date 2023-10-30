// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::address::{MultiChainAddress, RoochAddress};
use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use anyhow::{Ok, Result};
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    move_std::option::MoveOption,
    moveos_std::tx_context::TxContext,
    state::MoveState,
    transaction::FunctionCall,
};

/// Rust bindings for RoochFramework address_mapping module
pub struct AddressMapping<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> AddressMapping<'a> {
    const RESOLVE_FUNCTION_NAME: &'static IdentStr = ident_str!("resolve");
    const RESOLVE_OR_GENERATE_FUNCTION_NAME: &'static IdentStr = ident_str!("resolve_or_generate");

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
            let result = self
                .caller
                .call_function(&ctx, call)?
                .into_result()
                .map(|values| {
                    let value = values.get(0).expect("Expected return value");
                    let result = MoveOption::<AccountAddress>::from_bytes(&value.value)
                        .expect("Expected Option<address>");
                    result.into()
                })?;
            Ok(result)
        }
    }

    pub fn resolve_or_generate(
        &self,
        multichain_address: MultiChainAddress,
    ) -> Result<AccountAddress> {
        if multichain_address.is_rooch_address() {
            let rooch_address: RoochAddress = multichain_address.try_into()?;
            Ok(rooch_address.into())
        } else {
            let ctx = TxContext::zero();
            let call = FunctionCall::new(
                Self::function_id(Self::RESOLVE_OR_GENERATE_FUNCTION_NAME),
                vec![],
                vec![multichain_address.to_bytes()],
            );
            let address = self
                .caller
                .call_function(&ctx, call)?
                .into_result()
                .map(|values| {
                    let value = values.get(0).expect("Expected return value");
                    AccountAddress::from_bytes(&value.value).expect("Expected return address")
                })?;
            Ok(address)
        }
    }
}

impl<'a> ModuleBinding<'a> for AddressMapping<'a> {
    const MODULE_NAME: &'static IdentStr = ident_str!("address_mapping");
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
