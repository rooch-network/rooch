// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::language_storage::TypeTag;
use move_core_types::vm_status::KeptVMStatus;
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::transaction::MoveAction;
use rooch_types::address::RoochAddress;
use rooch_types::multichain_id::RoochMultiChainID;
use rooch_types::transaction::TransactionType;

#[derive(Debug, Clone)]
pub struct IndexedTransaction {
    /// The hash of this transaction.
    pub tx_hash: H256,
    /// The tx order of this transaction.
    pub tx_order: u128,

    pub transaction_type: TransactionType,
    pub sequence_number: u64,
    pub multichain_id: RoochMultiChainID,
    pub multichain_raw_address: Vec<u8>,
    /// the rooch address of sender who send the transaction
    pub sender: RoochAddress,
    pub action: MoveAction,
    pub action_type: u8,
    pub action_raw: Vec<u8>,
    pub auth_validator_id: u64,
    pub authenticator_payload: Vec<u8>,
    pub tx_accumulator_root: H256,
    pub transaction_raw: Vec<u8>,

    pub state_root: H256,
    pub event_root: H256,
    /// the amount of gas used.
    pub gas_used: u64,
    /// the vm status.
    pub status: KeptVMStatus,

    /// The tx order signature,
    pub tx_order_auth_validator_id: u64,
    pub tx_order_authenticator_payload: Vec<u8>,

    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone)]
pub struct IndexedEvent {
    /// event handle id
    pub event_handle_id: ObjectID,
    /// the number of messages that have been emitted to the path previously
    pub event_seq: u64,
    /// the type of the event data
    pub type_tag: TypeTag,
    /// the data payload of the event
    pub event_data: Vec<u8>,
    /// event index in the transaction events
    pub event_index: u64,

    /// the hash of this transaction.
    pub tx_hash: H256,
    /// the tx order of this transaction.
    pub tx_order: u128,
    /// the rooch address of sender who emit the event
    pub sender: RoochAddress,

    pub created_at: u64,
    pub updated_at: u64,
}
