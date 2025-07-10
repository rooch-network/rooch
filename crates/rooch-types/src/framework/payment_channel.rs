// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use move_core_types::language_storage::{StructTag, TypeTag};
use move_core_types::u256::U256;
use move_core_types::value::MoveValue;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::module_binding::{ModuleBinding, MoveFunctionCaller};
use moveos_types::{
    state::MoveState,
    transaction::MoveAction,
};

pub const MODULE_NAME: &IdentStr = ident_str!("payment_channel");

/// Rust bindings for rooch_framework::payment_channel module
pub struct PaymentChannelModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> PaymentChannelModule<'a> {
    pub const CREATE_PAYMENT_HUB_FUNCTION_NAME: &'static IdentStr = ident_str!("create_payment_hub");
    pub const DEPOSIT_TO_HUB_ENTRY_FUNCTION_NAME: &'static IdentStr = ident_str!("deposit_to_hub_entry");
    pub const OPEN_CHANNEL_ENTRY_FUNCTION_NAME: &'static IdentStr = ident_str!("open_channel_entry");
    pub const OPEN_SUB_CHANNEL_ENTRY_FUNCTION_NAME: &'static IdentStr = ident_str!("open_sub_channel_entry");
    pub const OPEN_CHANNEL_WITH_SUB_CHANNEL_ENTRY_FUNCTION_NAME: &'static IdentStr = ident_str!("open_channel_with_sub_channel_entry");
    pub const OPEN_CHANNEL_WITH_MULTIPLE_SUB_CHANNELS_ENTRY_FUNCTION_NAME: &'static IdentStr = ident_str!("open_channel_with_multiple_sub_channels_entry");
    pub const CLAIM_FROM_CHANNEL_ENTRY_FUNCTION_NAME: &'static IdentStr = ident_str!("claim_from_channel_entry");
    pub const CLOSE_CHANNEL_ENTRY_FUNCTION_NAME: &'static IdentStr = ident_str!("close_channel_entry");
    pub const INITIATE_CANCELLATION_ENTRY_FUNCTION_NAME: &'static IdentStr = ident_str!("initiate_cancellation_entry");
    pub const DISPUTE_CANCELLATION_ENTRY_FUNCTION_NAME: &'static IdentStr = ident_str!("dispute_cancellation_entry");
    pub const FINALIZE_CANCELLATION_ENTRY_FUNCTION_NAME: &'static IdentStr = ident_str!("finalize_cancellation_entry");

    pub fn create_payment_hub_action() -> MoveAction {
        Self::create_move_action(
            Self::CREATE_PAYMENT_HUB_FUNCTION_NAME,
            vec![],
            vec![],
        )
    }

    pub fn deposit_to_hub_entry_action(
        coin_type: StructTag,
        receiver: AccountAddress,
        amount: U256,
    ) -> MoveAction {
        Self::create_move_action(
            Self::DEPOSIT_TO_HUB_ENTRY_FUNCTION_NAME,
            vec![TypeTag::Struct(Box::new(coin_type))],
            vec![
                MoveValue::Address(receiver),
                MoveValue::U256(amount),
            ],
        )
    }

    pub fn open_channel_entry_action(
        coin_type: StructTag,
        channel_receiver: AccountAddress,
    ) -> MoveAction {
        Self::create_move_action(
            Self::OPEN_CHANNEL_ENTRY_FUNCTION_NAME,
            vec![TypeTag::Struct(Box::new(coin_type))],
            vec![
                MoveValue::Address(channel_receiver),
            ],
        )
    }

    pub fn open_channel_with_multiple_sub_channels_entry_action(
        coin_type: StructTag,
        channel_receiver: AccountAddress,
        vm_id_fragments: Vec<String>,
    ) -> MoveAction {
        let vm_fragments_move: Vec<MoveValue> = vm_id_fragments
            .into_iter()
            .map(|s| MoveValue::vector_u8(s.into_bytes()))
            .collect();

        Self::create_move_action(
            Self::OPEN_CHANNEL_WITH_MULTIPLE_SUB_CHANNELS_ENTRY_FUNCTION_NAME,
            vec![TypeTag::Struct(Box::new(coin_type))],
            vec![
                MoveValue::Address(channel_receiver),
                MoveValue::Vector(vm_fragments_move),
            ],
        )
    }

    pub fn claim_from_channel_entry_action(
        coin_type: StructTag,
        channel_id: moveos_types::moveos_std::object::ObjectID,
        sender_vm_id_fragment: String,
        sub_accumulated_amount: U256,
        sub_nonce: u64,
        sender_signature: Vec<u8>,
    ) -> MoveAction {
        Self::create_move_action(
            Self::CLAIM_FROM_CHANNEL_ENTRY_FUNCTION_NAME,
            vec![TypeTag::Struct(Box::new(coin_type))],
            vec![
                channel_id.to_move_value(),
                MoveValue::vector_u8(sender_vm_id_fragment.into_bytes()),
                MoveValue::U256(sub_accumulated_amount),
                MoveValue::U64(sub_nonce),
                MoveValue::Vector(sender_signature.into_iter().map(MoveValue::U8).collect()),
            ],
        )
    }

    pub fn close_channel_entry_action(
        coin_type: StructTag,
        channel_id: moveos_types::moveos_std::object::ObjectID,
        serialized_proofs: Vec<u8>,
    ) -> MoveAction {
        Self::create_move_action(
            Self::CLOSE_CHANNEL_ENTRY_FUNCTION_NAME,
            vec![TypeTag::Struct(Box::new(coin_type))],
            vec![
                channel_id.to_move_value(),
                MoveValue::Vector(serialized_proofs.into_iter().map(MoveValue::U8).collect()),
            ],
        )
    }

    pub fn initiate_cancellation_entry_action(
        coin_type: StructTag,
        channel_id: moveos_types::moveos_std::object::ObjectID,
    ) -> MoveAction {
        Self::create_move_action(
            Self::INITIATE_CANCELLATION_ENTRY_FUNCTION_NAME,
            vec![TypeTag::Struct(Box::new(coin_type))],
            vec![
                channel_id.to_move_value(),
            ],
        )
    }

    pub fn dispute_cancellation_entry_action(
        coin_type: StructTag,
        channel_id: moveos_types::moveos_std::object::ObjectID,
        sender_vm_id_fragment: String,
        dispute_accumulated_amount: U256,
        dispute_nonce: u64,
        sender_signature: Vec<u8>,
    ) -> MoveAction {
        Self::create_move_action(
            Self::DISPUTE_CANCELLATION_ENTRY_FUNCTION_NAME,
            vec![TypeTag::Struct(Box::new(coin_type))],
            vec![
                channel_id.to_move_value(),
                MoveValue::vector_u8(sender_vm_id_fragment.into_bytes()),
                MoveValue::U256(dispute_accumulated_amount),
                MoveValue::U64(dispute_nonce),
                MoveValue::Vector(sender_signature.into_iter().map(MoveValue::U8).collect()),
            ],
        )
    }

    pub fn finalize_cancellation_entry_action(
        coin_type: StructTag,
        channel_id: moveos_types::moveos_std::object::ObjectID,
    ) -> MoveAction {
        Self::create_move_action(
            Self::FINALIZE_CANCELLATION_ENTRY_FUNCTION_NAME,
            vec![TypeTag::Struct(Box::new(coin_type))],
            vec![
                channel_id.to_move_value(),
            ],
        )
    }
}

impl<'a> ModuleBinding<'a> for PaymentChannelModule<'a> {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
} 