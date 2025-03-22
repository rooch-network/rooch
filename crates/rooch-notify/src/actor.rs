// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::event::{GasUpgradeEvent, ServiceStatusEvent};
use crate::messages::{
    GasUpgradeMessage, NotifyActorSubscribeMessage, ProcessTxWithEventsMessage,
    UpdateServiceStatusMessage,
};
use crate::subscription_handler::SubscriptionHandler;
use async_trait::async_trait;
use coerce::actor::context::ActorContext;
use coerce::actor::message::Handler;
use coerce::actor::Actor;
use moveos_eventbus::bus::EventBus;
use std::sync::Arc;

pub struct NotifyActor {
    event_bus: EventBus,
    pub subscription_handler: Arc<SubscriptionHandler>,
}

impl NotifyActor {
    pub fn new(event_bus: EventBus, subscription_handler: Arc<SubscriptionHandler>) -> Self {
        Self {
            event_bus,
            subscription_handler,
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
