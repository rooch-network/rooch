// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;
use std::time::SystemTime;

use crate::finality::finalizer::{Config, Finalizer, FinalizerL1Mock};
use crate::messages::FinalityMessage;
use crate::metrics::FinalityMetrics;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor, LocalActorRef};
use function_name::named;
use moveos_eventbus::bus::EventData;
use prometheus::Registry;
use rooch_db::RoochDB;
use rooch_event::actor::{EventActor, EventActorSubscribeMessage};
use rooch_event::event::ServiceStatusEvent;
use rooch_types::finality_block::Block;
use rooch_types::service_status::ServiceStatus;
use tracing::info;

pub struct FinalityActor {
    // rooch_store: RoochStore,
    finalizer: Finalizer,
    rooch_db: RoochDB,
    service_status: ServiceStatus,
    metrics: Arc<FinalityMetrics>,
    event_actor: Option<LocalActorRef<EventActor>>,
}

impl FinalityActor {
    pub async fn new(
        rooch_db: RoochDB,
        service_status: ServiceStatus,
        registry: &Registry,
        event_actor: Option<LocalActorRef<EventActor>>,
    ) -> Result<Self, anyhow::Error> {
        let babylon_finality_gadget_rpc_str = "";
        let config = Config {
            babylon_finality_gadget_rpc: babylon_finality_gadget_rpc_str.to_string(),
        };
        //TODO implements finalize L1 service
        let finalizer_L1_Mock = Arc::new(FinalizerL1Mock::default());
        let finalizer = Finalizer::new(&config, finalizer_L1_Mock, rooch_db.clone())
            .await
            .map_err(|e| anyhow!(format!("New finality actor error: {:?}", e)))?;

        Ok(Self {
            finalizer,
            rooch_db,
            service_status,
            metrics: Arc::new(FinalityMetrics::new(registry)),
            event_actor,
        })
    }

    pub async fn subscribe_event(
        &self,
        event_actor_ref: LocalActorRef<EventActor>,
        executor_actor_ref: LocalActorRef<FinalityActor>,
    ) {
        let service_status_event = ServiceStatusEvent::default();
        let actor_subscribe_message = EventActorSubscribeMessage::new(
            service_status_event,
            "finality".to_string(),
            Box::new(executor_actor_ref),
        );
        let _ = event_actor_ref.send(actor_subscribe_message).await;
    }

    #[named]
    pub async fn finality(&mut self, block: Block) -> Result<()> {
        let fn_name = function_name!();
        let _timer = self
            .metrics
            .finality_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();

        // match self.service_status {
        //     ServiceStatus::ReadOnlyMode => {
        //         return Err(anyhow::anyhow!("The service is in read-only mode"));
        //     }
        //     ServiceStatus::DateImportMode => {
        //         if !tx_data.is_l1_block() && !tx_data.is_l1_tx() {
        //             return Err(anyhow::anyhow!(
        //                 "The service is in date import mode, only allow l1 block and l1 tx"
        //             ));
        //         }
        //     }
        //     ServiceStatus::Maintenance => {
        //         // Only the sequencer can send transactions in maintenance mode
        //         if let Some(sender) = tx_data.sender() {
        //             if sender != self.sequencer_key.public().rooch_address()? {
        //                 return Err(anyhow::anyhow!("The service is in maintenance mode"));
        //             }
        //         } else {
        //             return Err(anyhow::anyhow!("The service is in maintenance mode"));
        //         }
        //     }
        //     _ => {}
        // }

        let now = SystemTime::now();
        let _tx_timestamp = now.duration_since(SystemTime::UNIX_EPOCH)?.as_millis() as u64;

        self.finalizer.try_finalize().await?;
        info!(
            "rooch finality finalize block_hash: {} block_number: {:?}",
            block.block_hash, block.block_height
        );

        Ok(())
    }
}

#[async_trait]
impl Actor for FinalityActor {
    async fn started(&mut self, ctx: &mut ActorContext) {
        let local_actor_ref: LocalActorRef<Self> = ctx.actor_ref();
        if let Some(event_actor) = self.event_actor.clone() {
            let _ = self.subscribe_event(event_actor, local_actor_ref).await;
        }
    }
}

#[async_trait]
impl Handler<FinalityMessage> for FinalityActor {
    async fn handle(&mut self, msg: FinalityMessage, _ctx: &mut ActorContext) -> Result<()> {
        self.finality(msg.block).await
    }
}

#[async_trait]
impl Handler<EventData> for FinalityActor {
    async fn handle(&mut self, msg: EventData, _ctx: &mut ActorContext) -> Result<()> {
        if let Ok(service_status_event) = msg.data.downcast::<ServiceStatusEvent>() {
            let service_status = service_status_event.status;
            tracing::warn!("FinalityActor set self status to {:?}", service_status);
            self.service_status = service_status;
        }

        Ok(())
    }
}
