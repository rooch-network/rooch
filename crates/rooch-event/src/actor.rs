// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use coerce::actor::context::ActorContext;
use coerce::actor::message::{Handler, Message};
use coerce::actor::Actor;
use log;
use moveos_eventbus::bus::{EventBus, EventNotifier};
use moveos_eventbus::event::GasUpgradeEvent;

#[derive(Default)]
pub struct EventActor {
    event_bus: EventBus,
}

impl EventActor {
    pub fn new(event_bus: EventBus) -> Self {
        Self { event_bus }
    }
}

impl Actor for EventActor {}

#[derive(Default, Clone, Debug)]
pub struct GasUpgradeMessage {}

impl Message for GasUpgradeMessage {
    type Result = anyhow::Result<()>;
}

#[async_trait]
impl Handler<GasUpgradeMessage> for EventActor {
    async fn handle(
        &mut self,
        message: GasUpgradeMessage,
        _ctx: &mut ActorContext,
    ) -> anyhow::Result<()> {
        log::debug!("EventActor receive message {:?}", message);
        self.event_bus
            .notify::<GasUpgradeEvent>(GasUpgradeEvent {})?;
        Ok(())
    }
}

pub struct EventActorSubscribeMessage<T: Send + Sync + 'static> {
    event_type: T,
    subscriber: String,
    actor: Box<dyn EventNotifier + Send + Sync + 'static>,
}

impl<T: Send + Sync + 'static> Message for EventActorSubscribeMessage<T> {
    type Result = anyhow::Result<()>;
}

impl<T: Send + Sync + 'static> EventActorSubscribeMessage<T> {
    pub fn new(
        event_type: T,
        subscriber: String,
        actor: Box<dyn EventNotifier + Send + Sync + 'static>,
    ) -> EventActorSubscribeMessage<T> {
        Self {
            event_type,
            subscriber,
            actor,
        }
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> Handler<EventActorSubscribeMessage<T>> for EventActor {
    async fn handle(
        &mut self,
        message: EventActorSubscribeMessage<T>,
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
