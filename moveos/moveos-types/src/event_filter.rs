// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::event::{Event, EventID};
use crate::h256::H256;
use crate::move_types::type_tag_match;
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::TypeTag;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::serde_as;

#[serde_as]
// #[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[derive(Eq, PartialEq, Clone, Debug)]
// #[serde(rename = "Event", rename_all = "camelCase")]
pub struct MoveOSEvent {
    pub event_id: EventID,
    /// Sender's address.
    pub sender: AccountAddress,
    /// Transaction hash
    pub tx_hash: Option<H256>,
    /// The number of messages that have been emitted to the path previously
    // pub sequence_number: u64,
    // #[schemars(with = "String")]
    // #[serde_as(as = "TypeTag")]
    /// Move event type.
    pub type_tag: TypeTag,
    /// bcs bytes of the move event
    pub event_data: Vec<u8>,
    /// Parsed json value of the event data
    pub parsed_json: Value,
    /// UTC timestamp in milliseconds since epoch (1/1/1970)
    // #[serde(skip_serializing_if = "Option::is_none")]
    // #[schemars(with = "Option<u64>")]
    // #[serde_as(as = "Option<u64>")]
    pub timestamp_ms: Option<u64>,
    /// block height
    // #[serde(skip_serializing_if = "Option::is_none")]
    // #[schemars(with = "Option<u64>")]
    // #[serde_as(as = "Option<u64>")]
    pub block_height: Option<u64>,
    /// event index in the transaction events.
    pub event_index: u32,
}

impl MoveOSEvent {
    pub fn try_from(
        event: Event,
        tx_hash: Option<H256>,
        timestamp_ms: Option<u64>,
        block_height: Option<u64>,
    ) -> Result<Self> {
        let Event {
            event_id,
            // sequence_number,
            type_tag,
            event_data,
            event_index,
        } = event;

        //TODO how to store and derive sender address ?
        let sender = AccountAddress::ZERO;
        //TODO deserilize field from event_data
        let parsed_json = serde_json::to_value(event_data.clone()).unwrap();

        Ok(MoveOSEvent {
            event_id,
            sender,
            tx_hash,
            type_tag,
            event_data,
            parsed_json,
            timestamp_ms,
            block_height,
            event_index,
        })
    }
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EventFilter {
    /// Query by sender address.
    Sender(AccountAddress),
    /// Return events emitted by the given transaction.
    Transaction(
        ///tx hash of the transaction
        H256,
    ),
    /// Return events with the given move event struct name
    MoveEventType(
        // #[schemars(with = "String")]
        // #[serde_as(as = "TypeTag")]
        TypeTag,
    ),
    MoveEventField {
        path: String,
        value: Value,
    },
    /// Return events emitted in [start_time, end_time) interval
    // #[serde(rename_all = "camelCase")]
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
    /// Return events emitted in [from_block, to_block) interval
    #[serde(rename_all = "camelCase")]
    BlockRange {
        /// left endpoint of block height, inclusive
        // #[schemars(with = "u64")]
        // #[serde_as(as = "u64")]
        from_block: u64, //TODO use BlockNumber
        /// right endpoint of block height, exclusive
        // #[schemars(with = "u64")]
        // #[serde_as(as = "u64")]
        to_block: u64, //TODO use BlockNumber
    },

    All(Vec<EventFilter>),
    Any(Vec<EventFilter>),
    And(Box<EventFilter>, Box<EventFilter>),
    Or(Box<EventFilter>, Box<EventFilter>),
}

impl EventFilter {
    fn try_matches(&self, item: &MoveOSEvent) -> Result<bool> {
        Ok(match self {
            EventFilter::MoveEventType(event_type) => type_tag_match(&item.type_tag, event_type),
            EventFilter::MoveEventField { path, value } => {
                matches!(item.parsed_json.pointer(path), Some(v) if v == value)
            }

            EventFilter::Sender(sender) => &item.sender == sender,
            EventFilter::All(filters) => filters.iter().all(|f| f.matches(item)),
            EventFilter::Any(filters) => filters.iter().any(|f| f.matches(item)),
            EventFilter::And(f1, f2) => {
                EventFilter::All(vec![*(*f1).clone(), *(*f2).clone()]).matches(item)
            }
            EventFilter::Or(f1, f2) => {
                EventFilter::Any(vec![*(*f1).clone(), *(*f2).clone()]).matches(item)
            }
            EventFilter::Transaction(tx_hash) => {
                Option::is_some(&item.tx_hash) && (tx_hash == &item.tx_hash.unwrap())
            }
            EventFilter::TimeRange {
                start_time,
                end_time,
            } => {
                if let Some(timestamp) = &item.timestamp_ms {
                    start_time <= timestamp && end_time > timestamp
                } else {
                    false
                }
            }
            EventFilter::BlockRange {
                from_block,
                to_block,
            } => {
                if let Some(block_height) = &item.block_height {
                    from_block <= block_height && to_block > block_height
                } else {
                    false
                }
            }
        })
    }

    pub fn and(self, other_filter: EventFilter) -> Self {
        Self::All(vec![self, other_filter])
    }
    pub fn or(self, other_filter: EventFilter) -> Self {
        Self::Any(vec![self, other_filter])
    }
}

impl Filter<MoveOSEvent> for EventFilter {
    fn matches(&self, item: &MoveOSEvent) -> bool {
        self.try_matches(item).unwrap_or_default()
    }
}

pub trait Filter<T> {
    fn matches(&self, item: &T) -> bool;
}
