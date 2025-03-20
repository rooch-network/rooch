// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::event::{GasUpgradeEvent, ServiceStatusEvent};
use crate::subscription_handler::SubscriptionHandler;
use async_trait::async_trait;
use coerce::actor::context::ActorContext;
use coerce::actor::message::{Handler, Message};
use coerce::actor::Actor;
use moveos_eventbus::bus::{EventBus, EventNotifier};
use moveos_types::moveos_std::event::Event;
use moveos_types::moveos_std::tx_context::TxContext;
use rooch_types::service_status::ServiceStatus;
use rooch_types::transaction::TransactionWithInfo;
use std::sync::Arc;

pub struct NotifyActor {
    event_bus: EventBus,
    pub(crate) subscription_handler: Arc<SubscriptionHandler>,
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

#[derive(Default, Clone, Debug)]
pub struct GasUpgradeMessage {}

impl Message for GasUpgradeMessage {
    type Result = anyhow::Result<()>;
}

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

#[derive(Default, Clone, Debug)]
pub struct UpdateServiceStatusMessage {
    pub status: ServiceStatus,
}

impl Message for UpdateServiceStatusMessage {
    type Result = anyhow::Result<()>;
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

pub struct NotifyActorSubscribeMessage<T: Send + Sync + 'static> {
    event_type: T,
    subscriber: String,
    actor: Box<dyn EventNotifier + Send + Sync + 'static>,
}

impl<T: Send + Sync + 'static> Message for NotifyActorSubscribeMessage<T> {
    type Result = anyhow::Result<()>;
}

impl<T: Send + Sync + 'static> NotifyActorSubscribeMessage<T> {
    pub fn new(
        event_type: T,
        subscriber: String,
        actor: Box<dyn EventNotifier + Send + Sync + 'static>,
    ) -> NotifyActorSubscribeMessage<T> {
        Self {
            event_type,
            subscriber,
            actor,
        }
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

#[derive(Clone, Debug)]
pub struct ProcessTxWithEventsMessage {
    pub tx: TransactionWithInfo,
    pub events: Vec<Event>,
    pub ctx: TxContext,
}

impl Message for ProcessTxWithEventsMessage {
    type Result = anyhow::Result<()>;
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
