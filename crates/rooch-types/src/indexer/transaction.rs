// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::transaction::{LedgerTransaction, LedgerTxData};
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use moveos_types::h256::H256;
use moveos_types::transaction::{MoveAction, TransactionExecutionInfo, VerifiedMoveOSTransaction};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct IndexerTransaction {
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

    pub created_at: u64,
}

impl IndexerTransaction {
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
        let indexer_transaction = IndexerTransaction {
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

            created_at: transaction.sequence_info.tx_timestamp,
        };
        Ok(indexer_transaction)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransactionFilter {
    /// Query by sender address.
    Sender(AccountAddress),
    /// Query by multi chain original address.
    OriginalAddress(String),
    /// Query by the transaction hash list.
    TxHashes(Vec<H256>),
    /// Return transactions in [start_time, end_time) interval
    TimeRange {
        /// left endpoint of time interval, milliseconds since epoch, inclusive
        start_time: u64,
        /// right endpoint of time interval, milliseconds since epoch, exclusive
        end_time: u64,
    },
    /// Return transactions in [from_order, to_order) interval
    TxOrderRange {
        /// left endpoint of transaction order, inclusive
        from_order: u64,
        /// right endpoint of transaction order, exclusive
        to_order: u64,
    },
}
