// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{
    AccountAddressView, AnnotatedMoveStructView, H256View, StrView, StructTagView,
};
use moveos_types::moveos_std::event::{AnnotatedEvent, Event, EventID, TransactionEvent};
use rooch_types::indexer::event_filter::{EventFilter, IndexerEvent, IndexerEventID};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct TransactionEventView {
    pub event_type: StructTagView,
    pub event_data: StrView<Vec<u8>>,
    pub event_index: u64,
    pub decoded_event_data: Option<AnnotatedMoveStructView>,
}

impl From<TransactionEvent> for TransactionEventView {
    fn from(event: TransactionEvent) -> Self {
        TransactionEventView {
            event_type: event.event_type.into(),
            event_data: StrView(event.event_data),
            event_index: event.event_index,
            decoded_event_data: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct EventView {
    pub event_id: EventID,
    pub event_type: StructTagView,
    pub event_data: StrView<Vec<u8>>,
    pub event_index: u64,
    pub decoded_event_data: Option<AnnotatedMoveStructView>,
}

impl From<Event> for EventView {
    fn from(event: Event) -> Self {
        EventView {
            event_id: event.event_id,
            event_type: event.event_type.into(),
            event_data: StrView(event.event_data),
            event_index: event.event_index,
            decoded_event_data: None,
        }
    }
}

impl From<EventView> for Event {
    fn from(event: EventView) -> Self {
        Event {
            event_id: event.event_id,
            event_type: event.event_type.into(),
            event_data: event.event_data.0,
            event_index: event.event_index,
        }
    }
}

impl From<AnnotatedEvent> for EventView {
    fn from(event: AnnotatedEvent) -> Self {
        EventView {
            event_id: event.event.event_id,
            event_type: event.event.event_type.into(),
            event_data: StrView(event.event.event_data),
            event_index: event.event.event_index,
            decoded_event_data: Some(event.decoded_event_data.into()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct IndexerEventView {
    pub indexer_event_id: IndexerEventID,
    pub event_id: EventID,
    pub event_type: StructTagView,
    pub event_data: StrView<Vec<u8>>,
    pub tx_hash: H256View,
    pub sender: AccountAddressView,
    pub created_at: u64,

    pub decoded_event_data: Option<AnnotatedMoveStructView>,
}

impl From<IndexerEvent> for IndexerEventView {
    fn from(event: IndexerEvent) -> Self {
        IndexerEventView {
            indexer_event_id: event.indexer_event_id,
            event_id: event.event_id,
            event_type: event.event_type.into(),
            event_data: StrView(event.event_data),
            tx_hash: event.tx_hash.into(),
            sender: event.sender.into(),
            created_at: event.created_at,

            decoded_event_data: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum EventFilterView {
    /// Query by event type.
    EventType(StructTagView),
    /// Query by sender address.
    Sender(AccountAddressView),
    /// Return events emitted by the given transaction hash.
    TxHash(H256View),
    /// Return events emitted in [start_time, end_time) interval
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

impl From<EventFilterView> for EventFilter {
    fn from(event_filter: EventFilterView) -> Self {
        match event_filter {
            EventFilterView::EventType(event_type) => Self::EventType(event_type.into()),
            EventFilterView::Sender(address) => Self::Sender(address.into()),
            EventFilterView::TxHash(tx_hash) => Self::TxHash(tx_hash.into()),
            EventFilterView::TimeRange {
                start_time,
                end_time,
            } => Self::TimeRange {
                start_time,
                end_time,
            },
            EventFilterView::TxOrderRange {
                from_order,
                to_order,
            } => Self::TxOrderRange {
                from_order,
                to_order,
            },
        }
    }
}
