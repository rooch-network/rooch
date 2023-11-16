// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{
    AccountAddressView, H256View, TransactionExecutionInfoView, TransactionSequenceInfoView,
    TransactionView,
};
use rooch_types::indexer::transaction_filter::TransactionFilter;
use rooch_types::transaction::TransactionWithInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransactionWithInfoView {
    pub transaction: TransactionView,
    pub sequence_info: TransactionSequenceInfoView,
    pub execution_info: TransactionExecutionInfoView,
}

impl From<TransactionWithInfo> for TransactionWithInfoView {
    fn from(tx: TransactionWithInfo) -> Self {
        Self {
            transaction: tx.transaction.into(),
            sequence_info: tx.sequence_info.into(),
            execution_info: tx.execution_info.into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TransactionFilterView {
    /// Query by sender address.
    Sender(AccountAddressView),
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
