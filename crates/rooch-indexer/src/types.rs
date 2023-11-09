// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::errors::IndexerError;
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::StructTag;
use move_core_types::vm_status::KeptVMStatus;
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::transaction::{MoveAction, TransactionExecutionInfo, VerifiedMoveOSTransaction};
use rooch_types::address::RoochAddress;
use rooch_types::multichain_id::MultiChainID;
use rooch_types::transaction::{
    AbstractTransaction, TransactionSequenceInfo, TransactionType, TypedTransaction,
};

pub type IndexerResult<T> = Result<T, IndexerError>;

#[derive(Debug, Clone)]
pub struct IndexedTransaction {
    /// The hash of this transaction.
    pub tx_hash: H256,
    /// The tx order of this transaction.
    pub tx_order: u128,

    pub transaction_type: TransactionType,
    pub sequence_number: u64,
    pub multichain_id: MultiChainID,
    //TODO transform to hex
    pub multichain_raw_address: String,
    /// the account address of sender who send the transaction
    pub sender: AccountAddress,
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

impl IndexedTransaction {
    pub fn new(
        transaction: TypedTransaction,
        sequence_info: TransactionSequenceInfo,
        execution_info: TransactionExecutionInfo,
        moveos_tx: VerifiedMoveOSTransaction,
    ) -> Result<Self> {
        let move_action = MoveAction::from(moveos_tx.action);
        let action_raw = move_action.encode()?;
        let transaction_authenticator_info = transaction.authenticator_info()?;
        let indexed_transaction = IndexedTransaction {
            tx_hash: transaction.tx_hash(),
            /// The tx order of this transaction.
            tx_order: sequence_info.tx_order,

            transaction_type: transaction.transaction_type(),
            sequence_number: moveos_tx.ctx.sequence_number,
            multichain_id: transaction.multi_chain_id(),
            //TODO transform to hex
            multichain_raw_address: transaction.sender().to_string(),
            /// the rooch address of sender who send the transaction
            sender: moveos_tx.ctx.sender,
            action: move_action.clone(),
            action_type: move_action.action_type(),
            action_raw,
            auth_validator_id: transaction_authenticator_info
                .authenticator
                .auth_validator_id,
            authenticator_payload: transaction_authenticator_info.authenticator.payload,
            tx_accumulator_root: sequence_info.tx_accumulator_root,
            transaction_raw: transaction.encode(),

            state_root: execution_info.state_root,
            event_root: execution_info.event_root,
            /// the amount of gas used.
            gas_used: execution_info.gas_used,
            /// the vm status.
            status: execution_info.status,

            /// The tx order signature,
            tx_order_auth_validator_id: sequence_info.tx_order_signature.auth_validator_id,
            tx_order_authenticator_payload: sequence_info.tx_order_signature.payload,

            //TODO record transaction timestamp
            created_at: 0,
            updated_at: 0,
        };
        Ok(indexed_transaction)
    }
}

#[derive(Debug, Clone)]
pub struct IndexedEvent {
    /// event handle id
    pub event_handle_id: ObjectID,
    /// the number of messages that have been emitted to the path previously
    pub event_seq: u64,
    /// the type of the event data
    pub event_type: StructTag,
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
