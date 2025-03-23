// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use coerce::actor::message::Message;
use moveos_eventbus::bus::EventNotifier;
use moveos_types::moveos_std::event::Event;
use moveos_types::moveos_std::tx_context::TxContext;
use rooch_types::service_status::ServiceStatus;
use rooch_types::transaction::TransactionWithInfo;

#[derive(Default, Clone, Debug)]
pub struct GasUpgradeMessage {}

impl Message for GasUpgradeMessage {
    type Result = anyhow::Result<()>;
}

#[derive(Default, Clone, Debug)]
pub struct UpdateServiceStatusMessage {
    pub status: ServiceStatus,
}

impl Message for UpdateServiceStatusMessage {
    type Result = anyhow::Result<()>;
}

pub struct NotifyActorSubscribeMessage<T: Send + Sync + 'static> {
    pub event_type: T,
    pub subscriber: String,
    pub actor: Box<dyn EventNotifier + Send + Sync + 'static>,
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

#[derive(Clone, Debug)]
pub struct ProcessTxWithEventsMessage {
    pub tx: TransactionWithInfo,
    pub events: Vec<Event>,
    pub ctx: TxContext,
}

impl Message for ProcessTxWithEventsMessage {
    type Result = anyhow::Result<()>;
}
