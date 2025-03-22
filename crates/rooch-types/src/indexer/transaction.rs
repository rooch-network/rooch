// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::indexer::Filter;
use crate::transaction::{LedgerTransaction, LedgerTxData, TransactionWithInfo};
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use moveos_types::h256::H256;
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::transaction::{MoveAction, TransactionExecutionInfo};
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
    pub action_type: u8,
    pub auth_validator_id: u64,
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
        move_action: MoveAction,
        tx_context: TxContext,
    ) -> Result<Self> {
        let status = serde_json::to_string(&execution_info.status)?;
        let (auth_validator_id, _authenticator_payload) = match &transaction.data {
            LedgerTxData::L1Block(_block) => (0, vec![]),
            LedgerTxData::L1Tx(_tx) => (0, vec![]),
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

            sequence_number: tx_context.sequence_number,
            // the account address of sender who send the transaction
            sender: tx_context.sender,
            action_type: move_action.action_type(),
            auth_validator_id,
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
    All,
}

impl TransactionFilter {
    fn try_matches(&self, item: &TransactionWithInfo) -> Result<bool> {
        Ok(match self {
            TransactionFilter::Sender(sender) => {
                if let Some(tx_sender) = item.transaction.sender() {
                    sender == &AccountAddress::from(tx_sender)
                } else {
                    false
                }
            }
            TransactionFilter::TxHashes(tx_hash) => {
                if let Some(execution_info) = item.execution_info.clone() {
                    tx_hash.contains(&execution_info.tx_hash)
                } else {
                    false
                }
            }
            TransactionFilter::TimeRange {
                start_time,
                end_time,
            } => {
                *start_time <= item.transaction.sequence_info.tx_timestamp
                    && item.transaction.sequence_info.tx_timestamp < *end_time
            }
            TransactionFilter::TxOrderRange {
                from_order,
                to_order,
            } => {
                *from_order <= item.transaction.sequence_info.tx_order
                    && item.transaction.sequence_info.tx_order < *to_order
            }
            TransactionFilter::All => true,
        })
    }
}

impl Filter<TransactionWithInfo> for TransactionFilter {
    fn matches(&self, item: &TransactionWithInfo) -> bool {
        self.try_matches(item).unwrap_or_default()
    }
}
