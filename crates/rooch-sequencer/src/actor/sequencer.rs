// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::io;
use std::ops::Deref;
use std::sync::Arc;
use std::time::SystemTime;

use crate::messages::{
    GetSequencerInfoMessage, GetSequencerOrderMessage, GetTransactionByHashMessage,
    GetTransactionsByHashMessage, GetTxHashsMessage, TransactionSequenceMessage,
};
use crate::metrics::SequencerMetrics;
use accumulator::{Accumulator, MerkleAccumulator};
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor, LocalActorRef};
use function_name::named;
use moveos_eventbus::bus::EventData;
use moveos_types::h256::H256;
use prometheus::Registry;
use rooch_event::actor::{EventActor, EventActorSubscribeMessage};
use rooch_event::event::ServiceStatusEvent;
use rooch_store::transaction_store::TransactionStore;
use rooch_store::RoochStore;
use rooch_types::crypto::RoochKeyPair;
use rooch_types::sequencer::SequencerInfo;
use rooch_types::service_status::ServiceStatus;
use rooch_types::transaction::{LedgerTransaction, LedgerTxData};
use tracing::info;

pub struct SequencerActor {
    last_sequencer_info: SequencerInfo,
    tx_accumulator: MerkleAccumulator,
    sequencer_key: RoochKeyPair,
    rooch_store: RoochStore,
    service_status: ServiceStatus,
    metrics: Arc<SequencerMetrics>,
    event_actor: Option<LocalActorRef<EventActor>>,
}

impl SequencerActor {
    pub fn new(
        sequencer_key: RoochKeyPair,
        rooch_store: RoochStore,
        service_status: ServiceStatus,
        registry: &Registry,
        event_actor: Option<LocalActorRef<EventActor>>,
    ) -> Result<Self> {
        // The sequencer info would be initialized when genesis, so the sequencer info should not be None
        let last_sequencer_info = rooch_store
            .get_meta_store()
            .get_sequencer_info()?
            .ok_or_else(|| anyhow::anyhow!("Load sequencer info failed"))?;
        let (last_order, last_accumulator_info) = (
            last_sequencer_info.last_order,
            last_sequencer_info.last_accumulator_info.clone(),
        );
        info!("Load latest sequencer order {:?}", last_order);
        info!(
            "Load latest sequencer accumulator info {:?}",
            last_accumulator_info
        );
        let tx_accumulator = MerkleAccumulator::new_with_info(
            last_accumulator_info,
            rooch_store.get_transaction_accumulator_store(),
        );

        Ok(Self {
            last_sequencer_info,
            tx_accumulator,
            sequencer_key,
            rooch_store,
            service_status,
            metrics: Arc::new(SequencerMetrics::new(registry)),
            event_actor,
        })
    }

    pub async fn subscribe_event(
        &self,
        event_actor_ref: LocalActorRef<EventActor>,
        executor_actor_ref: LocalActorRef<SequencerActor>,
    ) {
        let service_status_event = ServiceStatusEvent::default();
        let actor_subscribe_message = EventActorSubscribeMessage::new(
            service_status_event,
            "sequencer".to_string(),
            Box::new(executor_actor_ref),
        );
        let _ = event_actor_ref.send(actor_subscribe_message).await;
    }

    pub fn last_order(&self) -> u64 {
        self.last_sequencer_info.last_order
    }

    /// Check the service status and validate the incoming transaction based on the status
    fn check_service_status(&self, tx_data: &LedgerTxData) -> Result<()> {
        match self.service_status {
            ServiceStatus::ReadOnlyMode => {
                return Err(anyhow::anyhow!("The service is in read-only mode"));
            }
            ServiceStatus::DateImportMode => {
                if !tx_data.is_l1_block() && !tx_data.is_l1_tx() {
                    return Err(anyhow::anyhow!(
                        "The service is in date import mode, only allow l1 block and l1 tx"
                    ));
                }
            }
            ServiceStatus::Maintenance => {
                // Only the sequencer can send transactions in maintenance mode
                if let Some(sender) = tx_data.sender() {
                    if sender != self.sequencer_key.public().rooch_address()? {
                        return Err(anyhow::anyhow!("The service is in maintenance mode"));
                    }
                } else {
                    return Err(anyhow::anyhow!("The service is in maintenance mode"));
                }
            }
            _ => {}
        }
        Ok(())
    }

    // last_sequencer_info may be inconsistent with the sequencer info in db caused by pipeline revert
    fn get_next_tx_order(&mut self) -> Result<u64> {
        let sequencer_info_db = self
            .rooch_store
            .get_meta_store()
            .get_sequencer_info()?
            .ok_or_else(|| anyhow::anyhow!("Load sequencer info failed"))?;
        self.last_sequencer_info = sequencer_info_db;

        Ok(self.last_sequencer_info.last_order + 1)
    }

    #[named]
    pub fn sequence(&mut self, mut tx_data: LedgerTxData) -> Result<LedgerTransaction> {
        let fn_name = function_name!();
        let _timer = self
            .metrics
            .sequencer_sequence_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();

        self.check_service_status(&tx_data)?;

        let now = SystemTime::now();
        let tx_timestamp = now.duration_since(SystemTime::UNIX_EPOCH)?.as_millis() as u64;
        let tx_order = self.get_next_tx_order()?;
        let tx_hash = tx_data.tx_hash();
        let tx_order_signature =
            LedgerTransaction::sign_tx_order(tx_order, tx_hash, &self.sequencer_key);
        let _tx_accumulator_root = self.tx_accumulator.append(vec![tx_hash].as_slice())?;

        let tx_accumulator_unsaved_nodes = self.tx_accumulator.pop_unsaved_nodes();
        let tx_accumulator_info = self.tx_accumulator.get_info();
        let tx = LedgerTransaction::build_ledger_transaction(
            tx_data,
            tx_timestamp,
            tx_order,
            tx_order_signature,
            tx_accumulator_info.clone(),
        );
        let sequencer_info = SequencerInfo::new(tx_order, tx_accumulator_info);
        let save_ret = self.rooch_store.save_sequenced_tx(
            tx_hash,
            tx.clone(),
            sequencer_info.clone(),
            tx_accumulator_unsaved_nodes,
            true,
        );
        if let Err(e) = save_ret {
            // database error/inconsistent issue happened,
            // revert accumulator appends avoiding dirty data in runtime
            // and set status to maintenance
            self.tx_accumulator = self
                .tx_accumulator
                .fork(Some(self.last_sequencer_info.last_accumulator_info.clone()));
            self.service_status = ServiceStatus::Maintenance;
            tracing::error!(
                        "Failed to save sequenced tx, tx_order: {}, error: {:?}, set sequencer to Maintenance mode.",
                        tx_order, e
                    );
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Save sequenced tx failed: {:?}", e),
            )
            .into());
        }
        self.tx_accumulator.clear_after_save();
        info!(
            "sequencer sequenced tx_hash: {:?} tx_order: {:?}",
            tx_hash, tx_order
        );
        self.last_sequencer_info = sequencer_info;

        Ok(tx)
    }
}

#[async_trait]
impl Actor for SequencerActor {
    async fn started(&mut self, ctx: &mut ActorContext) {
        let local_actor_ref: LocalActorRef<Self> = ctx.actor_ref();
        if let Some(event_actor) = self.event_actor.clone() {
            let _ = self.subscribe_event(event_actor, local_actor_ref).await;
        }
    }
}

#[async_trait]
impl Handler<TransactionSequenceMessage> for SequencerActor {
    async fn handle(
        &mut self,
        msg: TransactionSequenceMessage,
        _ctx: &mut ActorContext,
    ) -> Result<LedgerTransaction> {
        self.sequence(msg.tx)
    }
}

#[async_trait]
impl Handler<GetTransactionByHashMessage> for SequencerActor {
    async fn handle(
        &mut self,
        msg: GetTransactionByHashMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Option<LedgerTransaction>> {
        self.rooch_store.get_transaction_by_hash(msg.hash)
    }
}

#[async_trait]
impl Handler<GetTransactionsByHashMessage> for SequencerActor {
    async fn handle(
        &mut self,
        msg: GetTransactionsByHashMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<LedgerTransaction>>> {
        self.rooch_store.get_transactions_by_hash(msg.tx_hashes)
    }
}

#[async_trait]
impl Handler<GetTxHashsMessage> for SequencerActor {
    async fn handle(
        &mut self,
        msg: GetTxHashsMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<Option<H256>>> {
        let GetTxHashsMessage { tx_orders } = msg;
        self.rooch_store.get_tx_hashes(tx_orders)
    }
}

#[async_trait]
impl Handler<GetSequencerOrderMessage> for SequencerActor {
    async fn handle(
        &mut self,
        _msg: GetSequencerOrderMessage,
        _ctx: &mut ActorContext,
    ) -> Result<u64> {
        Ok(self.last_sequencer_info.last_order)
    }
}

#[async_trait]
impl Handler<EventData> for SequencerActor {
    async fn handle(&mut self, msg: EventData, _ctx: &mut ActorContext) -> Result<()> {
        if let Ok(service_status_event) = msg.data.downcast::<ServiceStatusEvent>() {
            let service_status = service_status_event.deref().status;
            tracing::warn!("SequencerActor set self status to {:?}", service_status);
            self.service_status = service_status;
        }

        Ok(())
    }
}

#[async_trait]
impl Handler<GetSequencerInfoMessage> for SequencerActor {
    async fn handle(
        &mut self,
        _msg: GetSequencerInfoMessage,
        _ctx: &mut ActorContext,
    ) -> Result<SequencerInfo> {
        Ok(self.last_sequencer_info.clone())
    }
}
