// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::event::{GasUpgradeEvent, ServiceStatusEvent};
use crate::messages::{
    GasUpgradeMessage, NotifyActorSubscribeMessage, ProcessTxWithEventsMessage,
    SubscribeEventsMessage, SubscribeTransactionsMessage, UpdateServiceStatusMessage,
};
use crate::subscription_handler::SubscriptionHandler;
use async_trait::async_trait;
use coerce::actor::context::ActorContext;
use coerce::actor::message::Handler;
use coerce::actor::Actor;
use moveos_eventbus::bus::EventBus;
use rooch_rpc_api::jsonrpc_types::event_view::IndexerEventView;
use rooch_rpc_api::jsonrpc_types::transaction_view::TransactionWithInfoView;
use std::sync::Arc;
use tokio_stream::wrappers::ReceiverStream;

pub struct NotifyActor {
    event_bus: EventBus,
    pub subscription_handler: Arc<SubscriptionHandler>,
}

impl NotifyActor {
    pub fn new(event_bus: EventBus, subscription_handler: SubscriptionHandler) -> Self {
        Self {
            event_bus,
            subscription_handler: Arc::new(subscription_handler),
        }
    }
}

impl Actor for NotifyActor {}

#[async_trait]
impl Handler<GasUpgradeMessage> for NotifyActor {
    async fn handle(
        &mut self,
        message: GasUpgradeMessage,
        _ctx: &mut ActorContext,
    ) -> anyhow::Result<()> {
        tracing::debug!("NotifyActor receive message {:?}", message);
        self.event_bus
            .notify::<GasUpgradeEvent>(GasUpgradeEvent {})?;
        Ok(())
    }
}

#[async_trait]
impl Handler<UpdateServiceStatusMessage> for NotifyActor {
    async fn handle(
        &mut self,
        message: UpdateServiceStatusMessage,
        _ctx: &mut ActorContext,
    ) -> anyhow::Result<()> {
        tracing::debug!("NotifyActor receive message {:?}", message);
        self.event_bus
            .notify::<ServiceStatusEvent>(ServiceStatusEvent {
                status: message.status,
            })?;
        Ok(())
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> Handler<NotifyActorSubscribeMessage<T>> for NotifyActor {
    async fn handle(
        &mut self,
        message: NotifyActorSubscribeMessage<T>,
        _ctx: &mut ActorContext,
    ) -> anyhow::Result<()> {
        let _ = message.event_type;
        let actor = message.actor;
        let subscriber = message.subscriber;

        self.event_bus
            .actor_subscribe::<T>(subscriber.as_str(), actor)?;

        Ok(())
    }
}

#[async_trait]
impl Handler<ProcessTxWithEventsMessage> for NotifyActor {
    async fn handle(
        &mut self,
        message: ProcessTxWithEventsMessage,
        _ctx: &mut ActorContext,
    ) -> anyhow::Result<()> {
        tracing::debug!("NotifyActor receive message {:?}", message);
        self.subscription_handler.process_tx_with_events(
            message.tx,
            message.events,
            message.ctx,
        )?;
        Ok(())
    }
}

#[async_trait]
impl Handler<SubscribeEventsMessage> for NotifyActor {
    async fn handle(
        &mut self,
        message: SubscribeEventsMessage,
        _ctx: &mut ActorContext,
    ) -> anyhow::Result<ReceiverStream<IndexerEventView>> {
        tracing::debug!("NotifyActor receive message {:?}", message);
        let stream = self.subscription_handler.subscribe_events(message.filter);
        Ok(stream)
    }
}

#[async_trait]
impl Handler<SubscribeTransactionsMessage> for NotifyActor {
    async fn handle(
        &mut self,
        message: SubscribeTransactionsMessage,
        _ctx: &mut ActorContext,
    ) -> anyhow::Result<ReceiverStream<TransactionWithInfoView>> {
        tracing::debug!("NotifyActor receive message {:?}", message);
        let stream = self
            .subscription_handler
            .subscribe_transactions(message.filter);
        Ok(stream)
    }
}
