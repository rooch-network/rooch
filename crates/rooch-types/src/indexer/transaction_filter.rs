// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// use crate::indexer::Filter;
// use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use moveos_types::h256::H256;
use serde::{Deserialize, Serialize};

// #[derive(
//     Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema,
// )]
// pub struct IndexerTransactionID {
//     pub tx_order: u64,
//     pub event_index: u64,
// }
//
// impl std::fmt::Display for IndexerTransactionID {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "IndexerTransactionID[tx order: {:?}, event index: {}]",
//             self.tx_order, self.event_index,
//         )
//     }
// }
//
// impl IndexerTransactionID {
//     pub fn new(tx_order: u64, event_index: u64) -> Self {
//         IndexerTransactionID {
//             tx_order,
//             event_index,
//         }
//     }
// }
//
// #[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
// pub struct IndexerTransaction {
//     /// The unique event_id that the event was indexed
//     pub indexer_event_id: IndexerTransactionID,
//     /// The unique event_id that the event was emitted to
//     pub event_id: TransactionID,
//     /// The type of the data
//     pub event_type: StructTag,
//     /// The data payload of the event
//     pub event_data: Vec<u8>,
//
//     /// the hash of this transaction.
//     pub tx_hash: H256,
//     /// the account address of sender who emit the event
//     pub sender: AccountAddress,
//
//     /// the event created timestamp on chain
//     pub created_at: u64,
// }

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
}

// impl TransactionFilter {
//     fn try_matches(&self, item: &IndexerTransaction) -> Result<bool> {
//         Ok(match self {
//             TransactionFilter::Sender(sender) => sender == &item.sender,
//             TransactionFilter::TxHashes(tx_hashes) => tx_hashes == &item.tx_hash,
//             TransactionFilter::TimeRange {
//                 start_time,
//                 end_time,
//             } => *start_time <= item.created_at && *end_time > item.created_at,
//             TransactionFilter::TxOrderRange {
//                 from_order,
//                 to_order,
//             } => {
//                 *from_order <= item.indexer_event_id.tx_order
//                     && *to_order > item.indexer_event_id.tx_order
//             }
//         })
//     }
// }
//
// impl Filter<IndexerTransaction> for TransactionFilter {
//     fn matches(&self, item: &IndexerTransaction) -> bool {
//         self.try_matches(item).unwrap_or_default()
//     }
// }
