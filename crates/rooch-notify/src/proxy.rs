// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::NotifyActor;
use crate::messages::{SubscribeEventsMessage, SubscribeTransactionsMessage};
use anyhow::Result;
use coerce::actor::ActorRef;
use rooch_rpc_api::jsonrpc_types::event_view::IndexerEventView;
use rooch_rpc_api::jsonrpc_types::transaction_view::TransactionWithInfoView;
use rooch_types::indexer::event::EventFilter;
use rooch_types::indexer::transaction::TransactionFilter;
use tokio_stream::wrappers::ReceiverStream;

#[derive(Clone)]
pub struct NotifyProxy {
    pub actor: ActorRef<NotifyActor>,
}

impl NotifyProxy {
    pub fn new(actor: ActorRef<NotifyActor>) -> Self {
        Self { actor }
    }

    pub async fn subscribe_events(
        &self,
        filter: EventFilter,
    ) -> Result<ReceiverStream<IndexerEventView>> {
        // pub fn subscribe_events(&self, filter: EventFilter) -> ReceiverStream<IndexerEvent> {
        self.actor.send(SubscribeEventsMessage { filter }).await?
    }

    pub async fn subscribe_transactions(
        &self,
        filter: TransactionFilter,
    ) -> Result<ReceiverStream<TransactionWithInfoView>> {
        self.actor
            .send(SubscribeTransactionsMessage { filter })
            .await?
    }
}

impl From<ActorRef<NotifyActor>> for NotifyProxy {
    fn from(actor: ActorRef<NotifyActor>) -> Self {
        Self::new(actor)
    }
}
