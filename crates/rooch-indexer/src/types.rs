// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::{StructTag, TypeTag};
use moveos_types::h256::H256;
use moveos_types::moveos_std::event::Event;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::moveos_std::object::RawObject;
use moveos_types::state::MoveStructType;
use moveos_types::transaction::{MoveAction, TransactionExecutionInfo, VerifiedMoveOSTransaction};
use rooch_types::bitcoin::utxo::UTXO;
use rooch_types::transaction::{LedgerTransaction, LedgerTxData};

use crate::errors::IndexerError;
use crate::utils::format_struct_tag;

pub type IndexerResult<T> = Result<T, IndexerError>;

#[derive(Debug, Clone)]
pub struct IndexedTransaction {
    // The hash of this transaction.
    pub tx_hash: H256,
    // The tx order of this transaction.
    pub tx_order: u64,

    pub sequence_number: u64,
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
    pub size: u64,
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
        mut transaction: LedgerTransaction,
        execution_info: TransactionExecutionInfo,
        moveos_tx: VerifiedMoveOSTransaction,
    ) -> Result<Self> {
        let move_action = MoveAction::from(moveos_tx.action);
        //TODO remove the action_raw field, and simply use the action field
        let action_raw = move_action.encode()?;
        let status = serde_json::to_string(&execution_info.status)?;
        let (auth_validator_id, authenticator_payload) = match &transaction.data {
            LedgerTxData::L1Block(_block) => (0, vec![]),
            LedgerTxData::L2Tx(tx) => (
                tx.authenticator().auth_validator_id,
                tx.authenticator().payload.clone(),
            ),
        };
        //TODO index L1Block
        let indexed_transaction = IndexedTransaction {
            tx_hash: transaction.tx_hash(),
            // The tx order of this transaction.
            tx_order: transaction.sequence_info.tx_order,

            sequence_number: moveos_tx.ctx.sequence_number,
            // the account address of sender who send the transaction
            sender: moveos_tx.ctx.sender,
            action: move_action.clone(),
            action_type: move_action.action_type(),
            action_raw,
            auth_validator_id,
            authenticator_payload,
            tx_accumulator_root: transaction.sequence_info.tx_accumulator_root,
            transaction_raw: transaction.encode(),

            state_root: execution_info.state_root,
            size: execution_info.size,
            event_root: execution_info.event_root,
            // the amount of gas used.
            gas_used: execution_info.gas_used,
            // the vm status.
            status,

            // The tx order signature,
            tx_order_auth_validator_id: transaction
                .sequence_info
                .tx_order_signature
                .auth_validator_id,
            tx_order_authenticator_payload: transaction.sequence_info.tx_order_signature.payload,

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
        mut transaction: LedgerTransaction,
        moveos_tx: VerifiedMoveOSTransaction,
    ) -> Self {
        IndexedEvent {
            event_handle_id: event.event_id.event_handle_id,
            event_seq: event.event_id.event_seq,
            event_type: event.event_type,
            event_data: event.event_data,
            event_index: event.event_index,

            tx_hash: transaction.tx_hash(),
            tx_order: transaction.sequence_info.tx_order,
            sender: moveos_tx.ctx.sender,

            //TODO record transaction timestamp
            created_at: 0,
        }
    }
}

/// Index all Object state, include child object
#[derive(Debug, Clone)]
pub struct IndexedObjectState {
    // The global state key
    pub object_id: ObjectID,
    // The owner of the object
    pub owner: AccountAddress,
    // A flag to indicate whether the object is shared or frozen
    pub flag: u8,
    // The table state root of the object
    pub state_root: AccountAddress,
    // The table length
    pub size: u64,
    // The T struct tag of the object value
    pub object_type: String,
    // The tx order of this transaction
    pub tx_order: u64,
    // The state index in the tx
    pub state_index: u64,
    // The object created timestamp on chain
    pub created_at: u64,
    // The object updated timestamp on chain
    pub updated_at: u64,
}

impl IndexedObjectState {
    pub fn new_from_raw_object(
        raw_object: RawObject,
        object_type: String,
        tx_order: u64,
        state_index: u64,
    ) -> Self {
        IndexedObjectState {
            object_id: raw_object.id,
            owner: raw_object.owner,
            flag: raw_object.flag,
            state_root: raw_object.state_root,
            size: raw_object.size,
            object_type,
            tx_order,
            state_index,

            //TODO record transaction timestamp
            created_at: 0,
            updated_at: 0,
        }
    }

    pub fn is_utxo_object_state(&self) -> bool {
        self.object_type == format_struct_tag(UTXO::struct_tag())
    }
}

/// Index all Object dynamic field
#[derive(Debug, Clone)]
pub struct IndexedFieldState {
    // The state object id
    pub object_id: ObjectID,
    // The hex of the field key state
    pub key_hex: String,
    // The key of the field, json format
    // `key` is a key word in SQlite, so use key_str as column name
    pub key_str: String,
    // The type tag of the key
    pub key_type: TypeTag,
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

impl IndexedFieldState {
    pub fn new(
        object_id: ObjectID,
        key_hex: String,
        key_state_json: String,
        key_type: TypeTag,
        value_type: TypeTag,
        tx_order: u64,
        state_index: u64,
    ) -> Self {
        IndexedFieldState {
            object_id,
            key_hex,
            key_str: key_state_json,
            key_type,
            value_type,
            tx_order,
            state_index,

            //TODO record transaction timestamp
            created_at: 0,
            updated_at: 0,
        }
    }

    // pub fn is_utxo_object_state(&self) -> bool {
    //     self.object_type == format_struct_tag(UTXO::struct_tag())
    // }
}
