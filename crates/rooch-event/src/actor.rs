// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use coerce::actor::context::ActorContext;
use coerce::actor::message::{Handler, Message};
use coerce::actor::Actor;
use log;
use moveos_eventbus::bus::{EventBus, EventNotifier};
use moveos_eventbus::event::GasUpgradeEvent;

#[derive(Default, Clone)]
pub struct EventActor {
    event_bus: EventBus,
}

impl EventActor {
    pub fn new(event_bus: EventBus) -> Self {
        Self { event_bus }
    }

    pub fn subscribe<T: Send + 'static>(
        &self,
        subscriber: &str,
        actor: Box<dyn EventNotifier + Send + Sync + 'static>,
    ) -> anyhow::Result<()> {
        self.event_bus.actor_subscribe::<T>(subscriber, actor)
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
