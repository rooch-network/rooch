// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::address::{BitcoinAddress, MultiChainAddress, RoochAddress};
use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use anyhow::{Ok, Result};
use move_core_types::value::MoveTypeLayout;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::moveos_std::object;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::state::{MoveStructState, MoveStructType};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    move_std::option::MoveOption,
    moveos_std::tx_context::TxContext,
    state::MoveState,
    transaction::FunctionCall,
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("address_mapping");

pub const NAMED_MAPPING_INDEX: u64 = 0;
pub const NAMED_REVERSE_MAPPING_INDEX: u64 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MultiChainAddressMapping {
    _placeholder: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoochToBitcoinAddressMapping {
    _placeholder: bool,
}

impl MoveStructType for MultiChainAddressMapping {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("MultiChainAddressMapping");
}

impl MoveStructState for MultiChainAddressMapping {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![MoveTypeLayout::Bool])
    }
}

impl MultiChainAddressMapping {
    pub fn object_id() -> ObjectID {
        object::named_object_id(&Self::struct_tag())
    }
}

impl MoveStructType for RoochToBitcoinAddressMapping {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("RoochToBitcoinAddressMapping");
}

impl MoveStructState for RoochToBitcoinAddressMapping {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![MoveTypeLayout::Bool])
    }
}

impl RoochToBitcoinAddressMapping {
    pub fn object_id() -> ObjectID {
        object::named_object_id(&Self::struct_tag())
    }
}

/// Rust bindings for RoochFramework address_mapping module
pub struct AddressMappingModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> AddressMappingModule<'a> {
    const RESOLVE_FUNCTION_NAME: &'static IdentStr = ident_str!("resolve");

    pub fn resolve(&self, multichain_address: MultiChainAddress) -> Result<Option<AccountAddress>> {
        if multichain_address.is_rooch_address() {
            let rooch_address: RoochAddress = multichain_address.try_into()?;
            Ok(Some(rooch_address.into()))
        } else if multichain_address.is_bitcoin_address() {
            let bitcoin_address: BitcoinAddress = multichain_address.try_into()?;
            Ok(Some(bitcoin_address.to_rooch_address().into()))
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
                    let value = values.first().expect("Expected return value");
                    let result = MoveOption::<AccountAddress>::from_bytes(&value.value)
                        .expect("Expected Option<address>");
                    result.into()
                })?;
            Ok(result)
        }
    }
}

impl<'a> ModuleBinding<'a> for AddressMappingModule<'a> {
    const MODULE_NAME: &'static IdentStr = ident_str!("address_mapping");
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
