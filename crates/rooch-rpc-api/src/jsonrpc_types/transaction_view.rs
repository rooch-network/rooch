// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::BytesView;
use crate::jsonrpc_types::{
    H256View, RoochOrBitcoinAddressView, TransactionExecutionInfoView, TransactionSequenceInfoView,
    TransactionView,
};
use rooch_types::indexer::transaction::TransactionFilter;
use rooch_types::transaction::{L1Block, LedgerTransaction, LedgerTxData, TransactionWithInfo};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct L1BlockView {
    pub chain_id: u64,
    pub block_height: u64,
    pub block_hash: BytesView,
}

impl From<L1Block> for L1BlockView {
    fn from(block: L1Block) -> Self {
        Self {
            chain_id: block.chain_id.id(),
            block_height: block.block_height,
            block_hash: block.block_hash.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LedgerTxDataView {
    L1Block(L1BlockView),
    L2Tx(TransactionView),
}

impl From<LedgerTxData> for LedgerTxDataView {
    fn from(data: LedgerTxData) -> Self {
        match data {
            LedgerTxData::L1Block(block) => LedgerTxDataView::L1Block(block.into()),
            LedgerTxData::L2Tx(tx) => LedgerTxDataView::L2Tx(tx.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LedgerTransactionView {
    pub data: LedgerTxDataView,
    pub sequence_info: TransactionSequenceInfoView,
}

impl From<LedgerTransaction> for LedgerTransactionView {
    fn from(tx: LedgerTransaction) -> Self {
        Self {
            data: tx.data.into(),
            sequence_info: tx.sequence_info.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransactionWithInfoView {
    pub transaction: LedgerTransactionView,
    pub execution_info: TransactionExecutionInfoView,
}

impl From<TransactionWithInfo> for TransactionWithInfoView {
    fn from(tx: TransactionWithInfo) -> Self {
        Self {
            transaction: tx.transaction.into(),
            execution_info: tx.execution_info.into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TransactionFilterView {
    /// Query by sender address.
    Sender(RoochOrBitcoinAddressView),
    /// Query by multi chain original address.
    OriginalAddress(String),
    /// Query by the given transaction hash.
    TxHashes(Vec<H256View>),
    /// Return transactions in [start_time, end_time) interval
    TimeRange {
        /// left endpoint of time interval, milliseconds since block, inclusive
        start_time: u64,
        /// right endpoint of time interval, milliseconds since block, exclusive
        end_time: u64,
    },
    /// Return events emitted in [from_order, to_order) interval
    // #[serde(rename_all = "camelCase")]
    TxOrderRange {
        /// left endpoint of transaction order, inclusive
        from_order: u64,
        /// right endpoint of transaction order, exclusive
        to_order: u64,
    },
}

impl From<TransactionFilterView> for TransactionFilter {
    fn from(event_filter: TransactionFilterView) -> Self {
        match event_filter {
            TransactionFilterView::Sender(address) => Self::Sender(address.into()),
            TransactionFilterView::OriginalAddress(address) => Self::OriginalAddress(address),
            TransactionFilterView::TxHashes(tx_hashes) => {
                Self::TxHashes(tx_hashes.into_iter().map(Into::into).collect())
            }
            TransactionFilterView::TimeRange {
                start_time,
                end_time,
            } => Self::TimeRange {
                start_time,
                end_time,
            },
            TransactionFilterView::TxOrderRange {
                from_order,
                to_order,
            } => Self::TxOrderRange {
                from_order,
                to_order,
            },
        }
    }
}
