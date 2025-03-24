// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{
    AnnotatedMoveStructView, H256View, HumanReadableDisplay, ObjectIDView, RoochAddressView,
    StrView, StructTagView, UnitedAddressView,
};
use moveos_types::moveos_std::{
    event::{AnnotatedEvent, Event, EventID, TransactionEvent},
    object::ObjectID,
};
use rooch_types::address::RoochAddress;
use rooch_types::indexer::event::{
    AnnotatedIndexerEvent, EventFilter, IndexerEvent, IndexerEventID,
};
use rooch_types::indexer::Filter;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct TransactionEventView {
    pub event_type: StructTagView,
    pub event_data: StrView<Vec<u8>>,
    pub event_index: StrView<u64>,
    pub decoded_event_data: Option<AnnotatedMoveStructView>,
}

impl From<TransactionEvent> for TransactionEventView {
    fn from(event: TransactionEvent) -> Self {
        TransactionEventView {
            event_type: event.event_type.into(),
            event_data: StrView(event.event_data),
            event_index: event.event_index.into(),
            decoded_event_data: None,
        }
    }
}

#[derive(
    Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema,
)]
pub struct EventIDView {
    /// each event handle corresponds to a unique event handle id. event handler id equal to guid.
    pub event_handle_id: ObjectID,
    /// For expansion: The number of messages that have been emitted to the path previously
    pub event_seq: StrView<u64>,
}

impl From<EventID> for EventIDView {
    fn from(event_id: EventID) -> Self {
        EventIDView {
            event_handle_id: event_id.event_handle_id,
            event_seq: StrView(event_id.event_seq),
        }
    }
}

impl From<EventIDView> for EventID {
    fn from(event_id: EventIDView) -> Self {
        EventID {
            event_handle_id: event_id.event_handle_id,
            event_seq: event_id.event_seq.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct EventView {
    pub event_id: EventIDView,
    pub event_type: StructTagView,
    pub event_data: StrView<Vec<u8>>,
    pub event_index: StrView<u64>,
    pub decoded_event_data: Option<AnnotatedMoveStructView>,
}

impl From<Event> for EventView {
    fn from(event: Event) -> Self {
        EventView {
            event_id: event.event_id.into(),
            event_type: event.event_type.into(),
            event_data: StrView(event.event_data),
            event_index: event.event_index.into(),
            decoded_event_data: None,
        }
    }
}

impl From<EventView> for Event {
    fn from(event: EventView) -> Self {
        Event {
            event_id: event.event_id.into(),
            event_type: event.event_type.into(),
            event_data: event.event_data.0,
            event_index: event.event_index.into(),
        }
    }
}

impl From<AnnotatedEvent> for EventView {
    fn from(event: AnnotatedEvent) -> Self {
        EventView {
            event_id: event.event.event_id.into(),
            event_type: event.event.event_type.into(),
            event_data: StrView(event.event.event_data),
            event_index: event.event.event_index.into(),
            decoded_event_data: Some(event.decoded_event_data.into()),
        }
    }
}

impl HumanReadableDisplay for EventView {
    fn to_human_readable_string(&self, _verbose: bool, indent: usize) -> String {
        format!(
            "{indent}event handle id: {}\n{indent}event seq      : {}\n{indent}event type     : {}",
            self.event_id.event_handle_id,
            self.event_id.event_seq.0,
            self.event_type,
            indent = " ".repeat(indent),
        )
    }
}

#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema,
)]
pub struct IndexerEventIDView {
    pub tx_order: StrView<u64>,
    pub event_index: StrView<u64>,
}

impl From<IndexerEventID> for IndexerEventIDView {
    fn from(event_id: IndexerEventID) -> Self {
        IndexerEventIDView {
            tx_order: StrView(event_id.tx_order),
            event_index: StrView(event_id.event_index),
        }
    }
}

impl From<IndexerEventIDView> for IndexerEventID {
    fn from(event_id: IndexerEventIDView) -> Self {
        IndexerEventID {
            tx_order: event_id.tx_order.0,
            event_index: event_id.event_index.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct IndexerEventView {
    pub indexer_event_id: IndexerEventIDView,
    pub event_id: EventIDView,
    pub event_type: StructTagView,
    pub event_data: StrView<Vec<u8>>,
    pub tx_hash: H256View,
    pub sender: RoochAddressView,
    pub created_at: StrView<u64>,
    pub decoded_event_data: Option<AnnotatedMoveStructView>,
}

impl From<IndexerEvent> for IndexerEventView {
    fn from(event: IndexerEvent) -> Self {
        IndexerEventView {
            indexer_event_id: event.indexer_event_id.into(),
            event_id: event.event_id.into(),
            event_type: event.event_type.into(),
            event_data: StrView(event.event_data.unwrap_or_default()),
            tx_hash: event.tx_hash.into(),
            sender: RoochAddress::from(event.sender).into(),
            created_at: event.created_at.into(),

            decoded_event_data: None,
        }
    }
}

impl From<IndexerEventView> for IndexerEvent {
    fn from(event: IndexerEventView) -> Self {
        IndexerEvent {
            indexer_event_id: event.indexer_event_id.into(),
            event_id: event.event_id.into(),
            event_type: event.event_type.into(),
            event_data: Some(event.event_data.0),
            tx_hash: event.tx_hash.into(),
            sender: RoochAddress::from(event.sender).into(),
            created_at: event.created_at.into(),
        }
    }
}

impl From<AnnotatedIndexerEvent> for IndexerEventView {
    fn from(event: AnnotatedIndexerEvent) -> Self {
        IndexerEventView {
            indexer_event_id: event.event.indexer_event_id.into(),
            event_id: event.event.event_id.into(),
            event_type: event.event.event_type.into(),
            event_data: StrView(event.event.event_data.unwrap_or_default()),
            tx_hash: event.event.tx_hash.into(),
            sender: RoochAddress::from(event.event.sender).into(),
            created_at: event.event.created_at.into(),
            decoded_event_data: Some(event.decoded_event_data.into()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EventFilterView {
    /// Query by event type with sender
    EventTypeWithSender {
        event_type: StructTagView,
        sender: UnitedAddressView,
    },
    /// Query by event type.
    EventType(StructTagView),
    /// Query by event handle id with sender
    EventHandleWithSender {
        event_handle_id: ObjectIDView,
        sender: UnitedAddressView,
    },
    /// Query by event handle id.
    EventHandle(ObjectIDView),
    /// Query by sender address.
    Sender(UnitedAddressView),
    /// Return events emitted by the given transaction hash.
    TxHash(H256View),
    /// Return events emitted in [start_time, end_time) interval
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
    All,
}

impl From<EventFilterView> for EventFilter {
    fn from(event_filter: EventFilterView) -> Self {
        match event_filter {
            EventFilterView::EventTypeWithSender { event_type, sender } => {
                Self::EventTypeWithSender {
                    event_type: event_type.into(),
                    sender: sender.0.rooch_address.into(),
                }
            }
            EventFilterView::EventType(event_type) => Self::EventType(event_type.into()),
            EventFilterView::EventHandleWithSender {
                event_handle_id,
                sender,
            } => Self::EventHandleWithSender {
                event_handle_id: event_handle_id.0,
                sender: sender.0.rooch_address.into(),
            },
            EventFilterView::EventHandle(event_handle_id) => Self::EventHandle(event_handle_id.0),
            EventFilterView::Sender(address) => Self::Sender(address.0.rooch_address.into()),
            EventFilterView::TxHash(tx_hash) => Self::TxHash(tx_hash.into()),
            EventFilterView::TimeRange {
                start_time,
                end_time,
            } => Self::TimeRange {
                start_time: start_time.0,
                end_time: end_time.0,
            },
            EventFilterView::TxOrderRange {
                from_order,
                to_order,
            } => Self::TxOrderRange {
                from_order: from_order.0,
                to_order: to_order.0,
            },
            EventFilterView::All => Self::All,
        }
    }
}

impl EventFilterView {
    fn try_matches(&self, item_view: &IndexerEventView) -> anyhow::Result<bool> {
        let filter: EventFilter = self.clone().into();
        let item: IndexerEvent = item_view.clone().into();
        Ok(filter.matches(&item))
    }
}

impl Filter<IndexerEventView> for EventFilterView {
    fn matches(&self, item: &IndexerEventView) -> bool {
        self.try_matches(item).unwrap_or_default()
    }
}
