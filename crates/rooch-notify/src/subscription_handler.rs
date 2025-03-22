// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use std::sync::Arc;

use crate::streamer::Streamer;
use moveos_types::moveos_std::event::Event;
use moveos_types::moveos_std::tx_context::TxContext;
use prometheus::{
    register_int_counter_vec_with_registry, register_int_gauge_vec_with_registry, IntCounterVec,
    IntGaugeVec, Registry,
};
use rooch_rpc_api::jsonrpc_types::event_view::{EventFilterView, IndexerEventView};
use rooch_rpc_api::jsonrpc_types::transaction_view::{
    TransactionFilterView, TransactionWithInfoView,
};
use rooch_types::indexer::event::IndexerEvent;
use rooch_types::transaction::TransactionWithInfo;
use tokio_stream::Stream;
use tracing::{error, trace};

pub const TRANSACTION_DISPATCH_BUFFER_SIZE: usize = 1000;
pub const EVENT_DISPATCH_BUFFER_SIZE: usize = 5000;

pub struct SubscriptionMetrics {
    pub streaming_success: IntCounterVec,
    pub streaming_failure: IntCounterVec,
    pub streaming_active_subscriber_number: IntGaugeVec,
    pub dropped_submissions: IntCounterVec,
}

impl SubscriptionMetrics {
    pub fn new(registry: &Registry) -> Self {
        Self {
            streaming_success: register_int_counter_vec_with_registry!(
                "streaming_success",
                "Total number of items that are streamed successfully",
                &["type"],
                registry,
            )
            .unwrap(),
            streaming_failure: register_int_counter_vec_with_registry!(
                "streaming_failure",
                "Total number of items that fail to be streamed",
                &["type"],
                registry,
            )
            .unwrap(),
            streaming_active_subscriber_number: register_int_gauge_vec_with_registry!(
                "streaming_active_subscriber_number",
                "Current number of active subscribers",
                &["type"],
                registry,
            )
            .unwrap(),
            dropped_submissions: register_int_counter_vec_with_registry!(
                "streaming_dropped_submissions",
                "Total number of submissions that are dropped",
                &["type"],
                registry,
            )
            .unwrap(),
        }
    }
}

pub struct SubscriptionHandler {
    event_streamer: Streamer<IndexerEventView, IndexerEventView, EventFilterView>,
    transaction_streamer:
        Streamer<TransactionWithInfoView, TransactionWithInfoView, TransactionFilterView>,
}

impl SubscriptionHandler {
    pub fn new(registry: &Registry) -> Self {
        let metrics = Arc::new(SubscriptionMetrics::new(registry));
        Self {
            event_streamer: Streamer::spawn(EVENT_DISPATCH_BUFFER_SIZE, metrics.clone(), "event"),
            transaction_streamer: Streamer::spawn(TRANSACTION_DISPATCH_BUFFER_SIZE, metrics, "tx"),
        }
    }
}

impl SubscriptionHandler {
    pub fn process_tx_with_events(
        &self,
        tx: TransactionWithInfo,
        events: Vec<Event>,
        ctx: TxContext,
    ) -> Result<()> {
        if tracing::enabled!(tracing::Level::TRACE) {
            trace!(
                num_events = events.len(),
                tx_order =? tx.transaction.sequence_info.tx_order,
                "Processing tx/event subscription"
            );
        }

        let tx_view = TransactionWithInfoView::new_from_transaction_with_info(tx.clone(), None);
        if let Err(e) = self.transaction_streamer.try_send(tx_view) {
            error!("Failed to send transaction to dispatch: {:?}", e);
        }

        // serially dispatch event processing to normal events' orders.
        let indexer_events = events
            .into_iter()
            .map(|event| {
                let indexer_event = IndexerEvent::new(event, tx.transaction.clone(), ctx.clone());
                IndexerEventView::from(indexer_event)
            })
            .collect::<Vec<_>>();
        for event in indexer_events.clone() {
            if let Err(e) = self.event_streamer.try_send(event) {
                error!("Failed to send event to dispatch: {:?}", e);
            }
        }
        Ok(())
    }

    pub fn subscribe_events(
        &self,
        filter: EventFilterView,
    ) -> impl Stream<Item = IndexerEventView> {
        // pub fn subscribe_events(&self, filter: EventFilterView) -> ReceiverStream<IndexerEventView> {
        self.event_streamer.subscribe(filter)
    }

    pub fn subscribe_transactions(
        &self,
        filter: TransactionFilterView,
    ) -> impl Stream<Item = TransactionWithInfoView> {
        // ) -> ReceiverStream<TransactionWithInfoView> {
        self.transaction_streamer.subscribe(filter)
    }
}
