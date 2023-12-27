// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::{StructTag, TypeTag};

use moveos_types::h256::H256;
use moveos_types::moveos_std::event::Event;
use moveos_types::moveos_std::object::{ObjectEntity, ObjectID, RawObject};
use moveos_types::moveos_std::raw_table::TableInfo;
use moveos_types::state::TableChangeSet;
use moveos_types::transaction::{MoveAction, TransactionExecutionInfo, VerifiedMoveOSTransaction};
use rooch_rpc_api::jsonrpc_types::TableChangeSetView;
use rooch_types::multichain_id::MultiChainID;
use rooch_types::transaction::{
    AbstractTransaction, TransactionSequenceInfo, TransactionType, TypedTransaction,
};

use crate::errors::IndexerError;

pub type IndexerResult<T> = Result<T, IndexerError>;

#[derive(Debug, Clone)]
pub struct IndexedTransaction {
    // The hash of this transaction.
    pub tx_hash: H256,
    // The tx order of this transaction.
    pub tx_order: u64,

    pub transaction_type: TransactionType,
    pub sequence_number: u64,
    pub multichain_id: MultiChainID,
    pub multichain_address: String,
    // the orginal address str
    pub multichain_original_address: String,
    // the account address of sender who send the transaction
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
    // the amount of gas used.
    pub gas_used: u64,
    // the vm status.
    pub status: String,
    // The tx order signature,
    pub tx_order_auth_validator_id: u64,
    pub tx_order_authenticator_payload: Vec<u8>,

    pub created_at: u64,
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
        let status = serde_json::to_string(&execution_info.status)?;

        let indexed_transaction = IndexedTransaction {
            tx_hash: transaction.tx_hash(),
            // The tx order of this transaction.
            tx_order: sequence_info.tx_order,

            transaction_type: transaction.transaction_type(),
            sequence_number: moveos_tx.ctx.sequence_number,
            multichain_id: transaction.multi_chain_id(),
            multichain_address: transaction.sender().to_string(),
            multichain_original_address: transaction.original_address_str(),
            // the account address of sender who send the transaction
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
            // the amount of gas used.
            gas_used: execution_info.gas_used,
            // the vm status.
            status,

            // The tx order signature,
            tx_order_auth_validator_id: sequence_info.tx_order_signature.auth_validator_id,
            tx_order_authenticator_payload: sequence_info.tx_order_signature.payload,

            //TODO record transaction timestamp
            created_at: 0,
        };
        Ok(indexed_transaction)
    }
}

#[derive(Debug, Clone)]
pub struct IndexedEvent {
    // event handle id
    pub event_handle_id: ObjectID,
    // the number of messages that have been emitted to the path previously
    pub event_seq: u64,
    // the type of the event data
    pub event_type: StructTag,
    // the data payload of the event
    pub event_data: Vec<u8>,
    // event index in the transaction events
    pub event_index: u64,

    // the hash of this transaction.
    pub tx_hash: H256,
    // the tx order of this transaction.
    pub tx_order: u64,
    // the account address of sender who emit the event
    pub sender: AccountAddress,

    pub created_at: u64,
}

impl IndexedEvent {
    pub fn new(
        event: Event,
        transaction: TypedTransaction,
        sequence_info: TransactionSequenceInfo,
        moveos_tx: VerifiedMoveOSTransaction,
    ) -> Self {
        IndexedEvent {
            event_handle_id: event.event_id.event_handle_id,
            event_seq: event.event_id.event_seq,
            event_type: event.event_type,
            event_data: event.event_data,
            event_index: event.event_index,

            tx_hash: transaction.tx_hash(),
            tx_order: sequence_info.tx_order,
            sender: moveos_tx.ctx.sender,

            //TODO record transaction timestamp
            created_at: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndexedGlobalState {
    // The global state key
    pub object_id: ObjectID,
    // The owner of the object
    pub owner: AccountAddress,
    // A flag to indicate whether the object is shared or frozen
    pub flag: u8,
    // The value of the object, json format
    pub value: String,
    // The T struct tag of the object value
    pub object_type: String,
    // The key type tag of the table
    pub key_type: String,
    // The table length
    pub size: u64,
    // The tx order of this transaction
    pub tx_order: u64,
    // The state index in the tx
    pub state_index: u64,
    // The object created timestamp on chain
    pub created_at: u64,
    // The object updated timestamp on chain
    pub updated_at: u64,
}

impl IndexedGlobalState {
    pub fn new_from_raw_object(
        raw_object: RawObject,
        raw_object_value_json: String,
        object_type: String,
        tx_order: u64,
        state_index: u64,
    ) -> Self {
        IndexedGlobalState {
            object_id: raw_object.id,
            owner: raw_object.owner,
            flag: raw_object.flag,

            value: raw_object_value_json,
            object_type,
            // Maintenance when it is a table handle
            key_type: "".to_string(),
            // Maintenance when it is a table handle
            size: 0,
            tx_order,
            state_index,

            //TODO record transaction timestamp
            created_at: 0,
            updated_at: 0,
        }
    }

    pub fn new_from_table_object(
        table_object: ObjectEntity<TableInfo>,
        table_object_value_json: String,
        object_type: String,
        key_type: String,
        tx_order: u64,
        state_index: u64,
    ) -> Self {
        IndexedGlobalState {
            object_id: table_object.id,
            owner: table_object.owner,
            flag: table_object.flag,

            value: table_object_value_json,
            object_type,
            // Maintenance when it is a table handle
            key_type,
            // Maintenance when it is a table handle
            size: table_object.value.size,
            tx_order,
            state_index,

            //TODO record transaction timestamp
            created_at: 0,
            updated_at: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndexedTableState {
    // The state table handle
    pub table_handle: ObjectID,
    // The hex of the table key
    pub key_hex: String,
    // The value of the table, json format
    pub value: String,
    // The type tag of the value
    pub value_type: TypeTag,
    // The tx order of this transaction
    pub tx_order: u64,
    // The state index in the tx
    pub state_index: u64,
    // The table item created timestamp on chain
    pub created_at: u64,
    // The table item updated timestamp on chain
    pub updated_at: u64,
}

impl IndexedTableState {
    pub fn new(
        table_handle: ObjectID,
        key_hex: String,
        state_value_json: String,
        value_type: TypeTag,
        tx_order: u64,
        state_index: u64,
    ) -> Self {
        IndexedTableState {
            table_handle,
            key_hex,
            value: state_value_json,
            value_type,
            tx_order,
            state_index,

            //TODO record transaction timestamp
            created_at: 0,
            updated_at: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndexedTableChangeSet {
    // The tx order of this transaction which produce the table change set
    pub tx_order: u64,
    // The table handle index in the tx
    pub state_index: u64,
    // The table handle
    pub table_handle: ObjectID,
    // The table change set, json format
    pub table_change_set: String,
    // The tx executed timestamp on chain
    pub created_at: u64,
}

impl IndexedTableChangeSet {
    pub fn new(
        tx_order: u64,
        state_index: u64,
        table_handle: ObjectID,
        table_change_set: TableChangeSet,
    ) -> Result<Self> {
        let table_change_set_json =
            serde_json::to_string(&TableChangeSetView::from(table_change_set))?;

        Ok(IndexedTableChangeSet {
            tx_order,
            state_index,
            table_handle,
            table_change_set: table_change_set_json,

            //TODO record transaction timestamp
            created_at: 0,
        })
    }
}
