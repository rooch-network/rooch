// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{BytesView, StrView};
use crate::jsonrpc_types::{
    H256View, TransactionExecutionInfoView, TransactionSequenceInfoView, TransactionView,
    UnitedAddressView,
};
use bitcoin::hashes::Hash;
use rooch_types::indexer::transaction::TransactionFilter;
use rooch_types::transaction::{
    L1Block, L1Transaction, LedgerTransaction, LedgerTxData, TransactionWithInfo,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct L1BlockView {
    pub chain_id: StrView<u64>,
    pub block_height: StrView<u64>,
    pub block_hash: BytesView,
    pub bitcoin_block_hash: Option<String>,
}

impl From<L1Block> for L1BlockView {
    fn from(block: L1Block) -> Self {
        Self {
            chain_id: block.chain_id.id().into(),
            block_height: block.block_height.into(),
            bitcoin_block_hash: if block.chain_id.is_bitcoin() {
                bitcoin::BlockHash::from_slice(&block.block_hash)
                    .map(|hash| hash.to_string())
                    .ok()
            } else {
                None
            },
            block_hash: block.block_hash.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct L1TransactionView {
    pub chain_id: StrView<u64>,
    pub block_hash: BytesView,
    pub bitcoin_block_hash: Option<String>,
    pub txid: BytesView,
    pub bitcoin_txid: Option<String>,
}

impl From<L1Transaction> for L1TransactionView {
    fn from(tx: L1Transaction) -> Self {
        Self {
            chain_id: tx.chain_id.id().into(),
            bitcoin_block_hash: if tx.chain_id.is_bitcoin() {
                bitcoin::BlockHash::from_slice(&tx.block_hash)
                    .map(|hash| hash.to_string())
                    .ok()
            } else {
                None
            },
            block_hash: tx.block_hash.into(),
            bitcoin_txid: if tx.chain_id.is_bitcoin() {
                bitcoin::Txid::from_slice(&tx.txid)
                    .map(|hash| hash.to_string())
                    .ok()
            } else {
                None
            },
            txid: tx.txid.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LedgerTxDataView {
    L1Block(L1BlockView),
    L1Tx(L1TransactionView),
    // The entire enum is at least 312 bytes
    // Consider boxing the large fields to reduce the total size of the enum
    L2Tx(Box<TransactionView>),
}

impl LedgerTxDataView {
    pub fn new_from_ledger_txdata(
        data: LedgerTxData,
        sender_bitcoin_address: Option<String>,
    ) -> Self {
        match data {
            LedgerTxData::L1Block(block) => LedgerTxDataView::L1Block(block.into()),
            LedgerTxData::L1Tx(tx) => LedgerTxDataView::L1Tx(tx.into()),
            LedgerTxData::L2Tx(tx) => LedgerTxDataView::L2Tx(Box::new(
                TransactionView::new_from_rooch_transaction(tx, sender_bitcoin_address),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LedgerTransactionView {
    pub data: LedgerTxDataView,
    pub sequence_info: TransactionSequenceInfoView,
}

impl LedgerTransactionView {
    pub fn new_from_ledger_transaction(
        tx: LedgerTransaction,
        sender_bitcoin_address: Option<String>,
    ) -> Self {
        Self {
            data: LedgerTxDataView::new_from_ledger_txdata(tx.data, sender_bitcoin_address),
            sequence_info: tx.sequence_info.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransactionWithInfoView {
    pub transaction: LedgerTransactionView,
    pub execution_info: Option<TransactionExecutionInfoView>,
}

impl TransactionWithInfoView {
    pub fn new_from_transaction_with_info(
        tx: TransactionWithInfo,
        sender_bitcoin_address: Option<String>,
    ) -> Self {
        Self {
            transaction: LedgerTransactionView::new_from_ledger_transaction(
                tx.transaction,
                sender_bitcoin_address,
            ),
            execution_info: tx.execution_info.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TransactionFilterView {
    /// Query by sender address.
    Sender(UnitedAddressView),
    /// Query by the given transaction hash.
    TxHashes(Vec<H256View>),
    /// Return transactions in [start_time, end_time) interval
    TimeRange {
        /// left endpoint of time interval, milliseconds since block, inclusive
        start_time: StrView<u64>,
        /// right endpoint of time interval, milliseconds since block, exclusive
        end_time: StrView<u64>,
    },
    /// Return events emitted in [from_order, to_order) interval
    // #[serde(rename_all = "camelCase")]
    TxOrderRange {
        /// left endpoint of transaction order, inclusive
        from_order: StrView<u64>,
        /// right endpoint of transaction order, exclusive
        to_order: StrView<u64>,
    },
}

impl From<TransactionFilterView> for TransactionFilter {
    fn from(event_filter: TransactionFilterView) -> Self {
        match event_filter {
            TransactionFilterView::Sender(address) => Self::Sender(address.0.rooch_address.into()),
            TransactionFilterView::TxHashes(tx_hashes) => {
                Self::TxHashes(tx_hashes.into_iter().map(Into::into).collect())
            }
            TransactionFilterView::TimeRange {
                start_time,
                end_time,
            } => Self::TimeRange {
                start_time: start_time.0,
                end_time: end_time.0,
            },
            TransactionFilterView::TxOrderRange {
                from_order,
                to_order,
            } => Self::TxOrderRange {
                from_order: from_order.0,
                to_order: to_order.0,
            },
        }
    }
}
