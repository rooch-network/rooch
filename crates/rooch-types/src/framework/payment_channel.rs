// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use anyhow::Result;
use move_core_types::language_storage::{StructTag, TypeTag};
use move_core_types::u256::U256;
use move_core_types::value::MoveValue;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::module_binding::{ModuleBinding, MoveFunctionCaller};
use moveos_types::move_std::option::MoveOption;
use moveos_types::move_std::string::MoveString;
use moveos_types::moveos_std::object::{custom_object_id, ObjectID};
use moveos_types::state::{MoveStructState, MoveStructType};
use moveos_types::{state::MoveState, transaction::MoveAction};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("payment_channel");

/// Key structure for identifying a unidirectional payment channel
/// Must match the ChannelKey struct in payment_channel.move
#[derive(Serialize)]
struct ChannelKey {
    sender: AccountAddress,
    receiver: AccountAddress,
    coin_type: String,
}

/// SubRAV data structure for BCS serialization
#[derive(Serialize, Deserialize)]
pub struct SubRAV {
    pub channel_id: ObjectID,
    pub vm_id_fragment: String,
    pub amount: U256,
    pub nonce: u64,
}

/// Structure for deserializing signed RAV from multibase encoded string
#[derive(Serialize, Deserialize)]
pub struct SignedSubRav {
    pub sub_rav: SubRAV,
    /// signature is the compressed signature bytes in hex format
    pub signature: String,
}

/// Proof for closing a sub-channel with final state
#[derive(Serialize, Deserialize)]
pub struct CloseProof {
    pub vm_id_fragment: String,
    pub accumulated_amount: U256,
    pub nonce: u64,
    pub sender_signature: Vec<u8>,
}

/// Container for multiple close proofs
#[derive(Serialize, Deserialize)]
pub struct CloseProofs {
    pub proofs: Vec<CloseProof>,
}

/// Proof for initiating cancellation of a sub-channel (no signature needed from sender)
#[derive(Serialize, Deserialize)]
pub struct CancelProof {
    pub vm_id_fragment: String,
    pub accumulated_amount: U256,
    pub nonce: u64,
}

/// Container for multiple cancel proofs
#[derive(Serialize, Deserialize)]
pub struct CancelProofs {
    pub proofs: Vec<CancelProof>,
}

impl SignedSubRav {
    pub fn encode_to_multibase(&self) -> Result<String> {
        let json_bytes = serde_json::to_vec(&self)?;
        Ok(multibase::encode(multibase::Base::Base58Btc, &json_bytes))
    }

    pub fn decode_from_multibase(encoded: &str) -> Result<Self> {
        let json_bytes = multibase::decode(encoded)?.1;
        Ok(serde_json::from_slice(&json_bytes)?)
    }
}

/// PaymentHub structure for Rust binding
/// Matches the PaymentHub struct in payment_channel.move
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentHub {
    pub multi_coin_store: ObjectID, // Object<MultiCoinStore> stored as ObjectID
    pub active_channels: ObjectID,  // Table<String, u64> handle stored as ObjectID
}

impl MoveStructType for PaymentHub {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("PaymentHub");

    fn struct_tag() -> StructTag {
        StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![],
        }
    }
}

impl MoveStructState for PaymentHub {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            // multi_coin_store: Object<MultiCoinStore>
            ObjectID::type_layout(),
            // active_channels: Table<String, u64>
            ObjectID::type_layout(),
        ])
    }
}

impl PaymentHub {
    pub fn multi_coin_store(&self) -> ObjectID {
        self.multi_coin_store.clone()
    }

    pub fn active_channels(&self) -> ObjectID {
        self.active_channels.clone()
    }
}

/// PaymentChannel structure for Rust binding
/// Matches the PaymentChannel struct in payment_channel.move
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentChannel {
    pub sender: AccountAddress,
    pub receiver: AccountAddress,
    pub coin_type: MoveString,
    // sub_channels: Table<String, SubChannel> - we'll handle this separately via field access
    pub sub_channels: ObjectID,
    pub status: u8,
    pub cancellation_info: MoveOption<CancellationInfo>,
}

impl MoveStructType for PaymentChannel {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("PaymentChannel");

    fn struct_tag() -> StructTag {
        StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![],
        }
    }
}

impl MoveStructState for PaymentChannel {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::Address, // sender
            move_core_types::value::MoveTypeLayout::Address, // receiver
            MoveString::type_layout(),                       // coin_type
            // sub_channels: Table<String, SubChannel>
            ObjectID::type_layout(),
            move_core_types::value::MoveTypeLayout::U8, // status
            MoveOption::<CancellationInfo>::type_layout(), // cancellation_info
        ])
    }
}

impl PaymentChannel {
    pub fn sender(&self) -> AccountAddress {
        self.sender
    }

    pub fn receiver(&self) -> AccountAddress {
        self.receiver
    }

    pub fn coin_type(&self) -> String {
        self.coin_type.to_string()
    }

    pub fn sub_channels(&self) -> ObjectID {
        self.sub_channels.clone()
    }

    pub fn status(&self) -> u8 {
        self.status
    }

    pub fn is_active(&self) -> bool {
        self.status == 0 // STATUS_ACTIVE
    }

    pub fn is_cancelling(&self) -> bool {
        self.status == 1 // STATUS_CANCELLING
    }

    pub fn is_closed(&self) -> bool {
        self.status == 2 // STATUS_CLOSED
    }

    pub fn cancellation_info(&self) -> Option<&CancellationInfo> {
        self.cancellation_info.as_ref()
    }
}

/// SubChannel structure for Rust binding
/// Matches the SubChannel struct in payment_channel.move
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubChannel {
    pub pk_multibase: MoveString,
    pub method_type: MoveString,
    pub last_claimed_amount: U256,
    pub last_confirmed_nonce: u64,
    pub status: u8,
}

impl MoveStructType for SubChannel {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("SubChannel");

    fn struct_tag() -> StructTag {
        StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![],
        }
    }
}

impl MoveStructState for SubChannel {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            MoveString::type_layout(),                    // pk_multibase
            MoveString::type_layout(),                    // method_type
            move_core_types::value::MoveTypeLayout::U256, // last_claimed_amount
            move_core_types::value::MoveTypeLayout::U64,  // last_confirmed_nonce
            move_core_types::value::MoveTypeLayout::U8,     // status
        ])
    }
}

impl SubChannel {
    pub fn pk_multibase(&self) -> String {
        self.pk_multibase.to_string()
    }

    pub fn method_type(&self) -> String {
        self.method_type.to_string()
    }

    pub fn last_claimed_amount(&self) -> U256 {
        self.last_claimed_amount
    }

    pub fn last_confirmed_nonce(&self) -> u64 {
        self.last_confirmed_nonce
    }

    pub fn status(&self) -> u8 {
        self.status
    }
}

/// CancellationInfo structure for Rust binding
/// Matches the CancellationInfo struct in payment_channel.move
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancellationInfo {
    pub initiated_time: u64,
    pub pending_amount: U256,
}

impl MoveStructType for CancellationInfo {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("CancellationInfo");

    fn struct_tag() -> StructTag {
        StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![],
        }
    }
}

impl MoveStructState for CancellationInfo {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::U64, // initiated_time
            move_core_types::value::MoveTypeLayout::U256, // pending_amount
        ])
    }
}

impl CancellationInfo {
    pub fn initiated_time(&self) -> u64 {
        self.initiated_time
    }

    pub fn pending_amount(&self) -> U256 {
        self.pending_amount
    }
}

/// Rust bindings for rooch_framework::payment_channel module
pub struct PaymentChannelModule<'a> {
    _caller: &'a dyn MoveFunctionCaller,
}

impl<'a> PaymentChannelModule<'a> {
    pub const CREATE_PAYMENT_HUB_FUNCTION_NAME: &'static IdentStr =
        ident_str!("create_payment_hub");
    pub const DEPOSIT_TO_HUB_ENTRY_FUNCTION_NAME: &'static IdentStr =
        ident_str!("deposit_to_hub_entry");
    pub const OPEN_CHANNEL_ENTRY_FUNCTION_NAME: &'static IdentStr =
        ident_str!("open_channel_entry");
    pub const OPEN_SUB_CHANNEL_ENTRY_FUNCTION_NAME: &'static IdentStr =
        ident_str!("open_sub_channel_entry");
    pub const OPEN_CHANNEL_WITH_SUB_CHANNEL_ENTRY_FUNCTION_NAME: &'static IdentStr =
        ident_str!("open_channel_with_sub_channel_entry");
    pub const CLAIM_FROM_CHANNEL_ENTRY_FUNCTION_NAME: &'static IdentStr =
        ident_str!("claim_from_channel_entry");
    pub const CLOSE_CHANNEL_ENTRY_FUNCTION_NAME: &'static IdentStr =
        ident_str!("close_channel_entry");
    pub const INITIATE_CANCELLATION_ENTRY_FUNCTION_NAME: &'static IdentStr =
        ident_str!("initiate_cancellation_entry");
    pub const DISPUTE_CANCELLATION_ENTRY_FUNCTION_NAME: &'static IdentStr =
        ident_str!("dispute_cancellation_entry");
    pub const FINALIZE_CANCELLATION_ENTRY_FUNCTION_NAME: &'static IdentStr =
        ident_str!("finalize_cancellation_entry");
    pub const INITIATE_CANCELLATION_WITH_PROOFS_ENTRY_FUNCTION_NAME: &'static IdentStr =
        ident_str!("initiate_cancellation_with_proofs_entry");

    /// Calculate the ObjectID for a payment hub
    /// This replicates the logic from payment_channel.move::get_payment_hub_id
    pub fn payment_hub_id(owner: AccountAddress) -> ObjectID {
        moveos_types::moveos_std::object::account_named_object_id(owner, &PaymentHub::struct_tag())
    }

    /// Calculate the deterministic ObjectID for a payment channel
    /// This replicates the logic from payment_channel.move::calc_channel_object_id
    pub fn calc_channel_object_id(
        coin_type: &StructTag,
        sender: AccountAddress,
        receiver: AccountAddress,
    ) -> ObjectID {
        // Create the ChannelKey (matches Move struct)
        let key = ChannelKey {
            sender,
            receiver,
            coin_type: coin_type.to_canonical_string(),
        };

        // Create the PaymentChannel struct tag (no longer generic)
        let channel_struct_tag = PaymentChannel::struct_tag();

        // Use MoveOS custom_object_id function (same as Move VM implementation)
        custom_object_id(&key, &channel_struct_tag)
    }

    pub fn create_payment_hub_action() -> MoveAction {
        Self::create_move_action(Self::CREATE_PAYMENT_HUB_FUNCTION_NAME, vec![], vec![])
    }

    pub fn deposit_to_hub_entry_action(
        coin_type: StructTag,
        receiver: AccountAddress,
        amount: U256,
    ) -> MoveAction {
        Self::create_move_action(
            Self::DEPOSIT_TO_HUB_ENTRY_FUNCTION_NAME,
            vec![TypeTag::Struct(Box::new(coin_type))],
            vec![MoveValue::Address(receiver), MoveValue::U256(amount)],
        )
    }

    pub fn open_channel_entry_action(
        coin_type: StructTag,
        channel_receiver: AccountAddress,
    ) -> MoveAction {
        Self::create_move_action(
            Self::OPEN_CHANNEL_ENTRY_FUNCTION_NAME,
            vec![TypeTag::Struct(Box::new(coin_type))],
            vec![MoveValue::Address(channel_receiver)],
        )
    }

    pub fn open_channel_with_sub_channel_entry_action(
        coin_type: StructTag,
        channel_receiver: AccountAddress,
        vm_id_fragment: String,
    ) -> MoveAction {
        Self::create_move_action(
            Self::OPEN_CHANNEL_WITH_SUB_CHANNEL_ENTRY_FUNCTION_NAME,
            vec![TypeTag::Struct(Box::new(coin_type))],
            vec![
                MoveValue::Address(channel_receiver),
                MoveValue::vector_u8(vm_id_fragment.into_bytes()),
            ],
        )
    }

    pub fn claim_from_channel_entry_action(
        channel_id: moveos_types::moveos_std::object::ObjectID,
        sender_vm_id_fragment: String,
        sub_accumulated_amount: U256,
        sub_nonce: u64,
        sender_signature: Vec<u8>,
    ) -> MoveAction {
        Self::create_move_action(
            Self::CLAIM_FROM_CHANNEL_ENTRY_FUNCTION_NAME,
            vec![],
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
        channel_id: moveos_types::moveos_std::object::ObjectID,
        serialized_proofs: Vec<u8>,
    ) -> MoveAction {
        Self::create_move_action(
            Self::CLOSE_CHANNEL_ENTRY_FUNCTION_NAME,
            vec![],
            vec![
                channel_id.to_move_value(),
                MoveValue::Vector(serialized_proofs.into_iter().map(MoveValue::U8).collect()),
            ],
        )
    }

    pub fn initiate_cancellation_entry_action(
        channel_id: moveos_types::moveos_std::object::ObjectID,
    ) -> MoveAction {
        Self::create_move_action(
            Self::INITIATE_CANCELLATION_ENTRY_FUNCTION_NAME,
            vec![],
            vec![channel_id.to_move_value()],
        )
    }

    pub fn dispute_cancellation_entry_action(
        channel_id: moveos_types::moveos_std::object::ObjectID,
        sender_vm_id_fragment: String,
        dispute_accumulated_amount: U256,
        dispute_nonce: u64,
        sender_signature: Vec<u8>,
    ) -> MoveAction {
        Self::create_move_action(
            Self::DISPUTE_CANCELLATION_ENTRY_FUNCTION_NAME,
            vec![],
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
        channel_id: moveos_types::moveos_std::object::ObjectID,
    ) -> MoveAction {
        Self::create_move_action(
            Self::FINALIZE_CANCELLATION_ENTRY_FUNCTION_NAME,
            vec![],
            vec![channel_id.to_move_value()],
        )
    }

    pub fn initiate_cancellation_with_proofs_entry_action(
        channel_id: moveos_types::moveos_std::object::ObjectID,
        serialized_proofs: Vec<u8>,
    ) -> MoveAction {
        Self::create_move_action(
            Self::INITIATE_CANCELLATION_WITH_PROOFS_ENTRY_FUNCTION_NAME,
            vec![],
            vec![
                channel_id.to_move_value(),
                MoveValue::Vector(serialized_proofs.into_iter().map(MoveValue::U8).collect()),
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
        Self { _caller: caller }
    }
}
