// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{address::MultiChainAddress, addresses::ROOCH_FRAMEWORK_ADDRESS};
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    language_storage::{StructTag, TypeTag},
    u256::U256,
    value::MoveValue,
};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    transaction::MoveAction,
};

pub const MODULE_NAME: &IdentStr = ident_str!("transfer");

/// Rust bindings for RoochFramework transfer module
pub struct TransferModule;

impl TransferModule {
    pub const TRANSFER_COIN_FUNCTION_NAME: &'static IdentStr = ident_str!("transfer_coin");
    pub const TRANSFER_COIN_TO_MULTICHAIN_ADDRESS_FUNCTION_NAME: &'static IdentStr =
        ident_str!("transfer_coin_to_multichain_address");

    pub fn create_transfer_coin_action(
        coin_type: StructTag,
        to: AccountAddress,
        amount: U256,
    ) -> MoveAction {
        Self::create_move_action(
            Self::TRANSFER_COIN_FUNCTION_NAME,
            vec![TypeTag::Struct(Box::new(coin_type))],
            vec![MoveValue::Address(to), MoveValue::U256(amount)],
        )
    }

    pub fn create_transfer_coin_to_multichain_address_action(
        coin_type: StructTag,
        to: MultiChainAddress,
        amount: U256,
    ) -> MoveAction {
        Self::create_move_action(
            Self::TRANSFER_COIN_TO_MULTICHAIN_ADDRESS_FUNCTION_NAME,
            vec![TypeTag::Struct(Box::new(coin_type))],
            vec![
                MoveValue::U64(to.multichain_id as u64),
                MoveValue::vector_u8(to.raw_address),
                MoveValue::U256(amount),
            ],
        )
    }
}

impl<'a> ModuleBinding<'a> for TransferModule {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(_caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self
    }
}
