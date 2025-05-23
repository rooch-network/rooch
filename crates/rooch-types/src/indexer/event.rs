// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::indexer::Filter;
use crate::transaction::LedgerTransaction;
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::StructTag;
use move_resource_viewer::AnnotatedMoveStruct;
use moveos_types::h256::H256;
use moveos_types::move_types::struct_tag_match;
use moveos_types::moveos_std::event::{Event, EventID};
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::moveos_std::tx_context::TxContext;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct IndexerEvent {
    /// The unique event_id that the event was indexer
    pub indexer_event_id: IndexerEventID,
    /// The unique event_id that the event was emitted to
    pub event_id: EventID,
    /// The type of the data
    pub event_type: StructTag,
    /// The data payload of the event
    pub event_data: Option<Vec<u8>>,

    /// the hash of this transaction.
    pub tx_hash: H256,
    /// the account address of sender who emit the event
    pub sender: AccountAddress,

    /// the event created timestamp on chain
    pub created_at: u64,
}

impl IndexerEvent {
    pub fn new(event: Event, mut ledger_transaction: LedgerTransaction, ctx: TxContext) -> Self {
        IndexerEvent {
            indexer_event_id: IndexerEventID::new(
                ledger_transaction.sequence_info.tx_order,
                event.event_index,
            ),
            event_id: event.event_id,

            event_type: event.event_type,
            event_data: Some(event.event_data),
            tx_hash: ledger_transaction.tx_hash(),
            sender: ctx.sender,

            created_at: ledger_transaction.sequence_info.tx_timestamp,
        }
    }
}

#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema,
)]
pub struct IndexerEventID {
    pub tx_order: u64,
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
    pub fn new(tx_order: u64, event_index: u64) -> Self {
        IndexerEventID {
            tx_order,
            event_index,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EventFilter {
    /// Query by event type with sender
    EventTypeWithSender {
        event_type: StructTag,
        sender: AccountAddress,
    },
    /// Query by event type.
    EventType(StructTag),
    /// Query by event handle id with sender
    EventHandleWithSender {
        event_handle_id: ObjectID,
        sender: AccountAddress,
    },
    /// Query by event handle id.
    EventHandle(ObjectID),
    /// Query by sender address.
    Sender(AccountAddress),
    /// Return events emitted by the given transaction hash.
    TxHash(H256),
    /// Return events emitted in [start_time, end_time) interval
    TimeRange {
        /// left endpoint of time interval, milliseconds since epoch, inclusive
        start_time: u64,
        /// right endpoint of time interval, milliseconds since epoch, exclusive
        end_time: u64,
    },
    /// Return events emitted in [from_order, to_order) interval
    TxOrderRange {
        /// left endpoint of transaction order, inclusive
        from_order: u64,
        /// right endpoint of transaction order, exclusive
        to_order: u64,
    },
    All,
}

impl EventFilter {
    fn try_matches(&self, item: &IndexerEvent) -> Result<bool> {
        Ok(match self {
            EventFilter::EventTypeWithSender {
                event_type, sender, ..
            } => struct_tag_match(&item.event_type, event_type) && sender == &item.sender,
            EventFilter::EventType(event_type) => struct_tag_match(&item.event_type, event_type),
            EventFilter::EventHandleWithSender {
                event_handle_id,
                sender,
                ..
            } => event_handle_id == &item.event_id.event_handle_id && sender == &item.sender,
            EventFilter::EventHandle(event_handle_id) => {
                event_handle_id == &item.event_id.event_handle_id
            }
            EventFilter::Sender(sender) => sender == &item.sender,
            EventFilter::TxHash(tx_hash) => tx_hash == &item.tx_hash,
            EventFilter::TimeRange {
                start_time,
                end_time,
            } => *start_time <= item.created_at && item.created_at < *end_time,
            EventFilter::TxOrderRange {
                from_order,
                to_order,
            } => {
                *from_order <= item.indexer_event_id.tx_order
                    && item.indexer_event_id.tx_order < *to_order
            }
            EventFilter::All => true,
        })
    }
}

impl Filter<IndexerEvent> for EventFilter {
    fn matches(&self, item: &IndexerEvent) -> bool {
        self.try_matches(item).unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct AnnotatedIndexerEvent {
    pub event: IndexerEvent,
    pub decoded_event_data: AnnotatedMoveStruct,
}

impl AnnotatedIndexerEvent {
    pub fn new(event: IndexerEvent, decoded_event_data: AnnotatedMoveStruct) -> Self {
        AnnotatedIndexerEvent {
            event,
            decoded_event_data,
        }
    }
}
