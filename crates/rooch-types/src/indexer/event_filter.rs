// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::indexer::Filter;
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::StructTag;
use moveos_types::h256::H256;
use moveos_types::move_types::struct_tag_match;
use moveos_types::moveos_std::event::EventID;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema,
)]
pub struct IndexerEventID {
    pub tx_order: u128,
    pub event_index: u64,
}

impl std::fmt::Display for IndexerEventID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "IndexerEventID[tx order: {:?}, event index: {}]",
            self.tx_order, self.event_index,
        )
    }
}

impl IndexerEventID {
    pub fn new(tx_order: u128, event_index: u64) -> Self {
        IndexerEventID {
            tx_order,
            event_index,
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct IndexerEvent {
    /// The unique event_id that the event was indexed
    pub indexer_event_id: IndexerEventID,
    /// The unique event_id that the event was emitted to
    pub event_id: EventID,
    /// The type of the data
    pub event_type: StructTag,
    /// The data payload of the event
    // #[serde(with = "serde_bytes")]
    pub event_data: Vec<u8>,
    /// event index in the transaction events.
    // pub event_index: u64,

    /// the hash of this transaction.
    pub tx_hash: H256,
    /// the tx order of this transaction.
    // pub tx_order: u128,
    /// the account address of sender who emit the event
    pub sender: AccountAddress,

    /// the event created timestamp on chain
    pub created_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase", default)]
pub enum EventFilter {
    /// Query by event type.
    EventType(StructTag),
    /// Query by sender address.
    Sender(AccountAddress),
    /// Return events emitted by the given transaction hash.
    TxHash(H256),
    /// Return events emitted in [start_time, end_time) interval
    #[serde(rename_all = "camelCase")]
    TimeRange {
        /// left endpoint of time interval, milliseconds since epoch, inclusive
        // #[schemars(with = "u64")]
        // #[serde_as(as = "u64")]
        start_time: u64,
        /// right endpoint of time interval, milliseconds since epoch, exclusive
        // #[schemars(with = "u64")]
        // #[serde_as(as = "u64")]
        end_time: u64,
    },
    /// Return events emitted in [from_tx_order, to_tx_order) interval
    #[serde(rename_all = "camelCase")]
    TxOrderRange {
        /// left endpoint of transaction order, inclusive
        // #[schemars(with = "u128")]
        // #[serde_as(as = "u128")]
        from_order: u128,
        /// right endpoint of transaction order, exclusive
        // #[schemars(with = "u128")]
        // #[serde_as(as = "u128")]
        to_order: u128,
    },
}

impl EventFilter {
    fn try_matches(&self, item: &IndexerEvent) -> Result<bool> {
        Ok(match self {
            EventFilter::EventType(event_type) => struct_tag_match(&item.event_type, event_type),
            EventFilter::Sender(sender) => sender == &item.sender,
            EventFilter::TxHash(tx_hash) => tx_hash == &item.tx_hash,
            EventFilter::TimeRange {
                start_time,
                end_time,
            } => *start_time <= item.created_at && *end_time > item.created_at,
            EventFilter::TxOrderRange {
                from_order,
                to_order,
            } => {
                *from_order <= item.indexer_event_id.tx_order
                    && *to_order > item.indexer_event_id.tx_order
            }
        })
    }
}

impl Filter<IndexerEvent> for EventFilter {
    fn matches(&self, item: &IndexerEvent) -> bool {
        self.try_matches(item).unwrap_or_default()
    }
}
