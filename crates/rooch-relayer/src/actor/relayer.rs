// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::bitcoin_relayer::BitcoinRelayer;
use super::ethereum_relayer::EthereumRelayer;
use super::messages::RelayTick;
use crate::actor::bitcoin_client::BitcoinClientActor;
use crate::actor::bitcoin_client_proxy::BitcoinClientProxy;
use crate::actor::relayer_proxy::RelayerProxy;
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor, LocalActorRef};
use move_core_types::vm_status::KeptVMStatus;
use moveos_eventbus::bus::EventData;
use moveos_types::module_binding::MoveFunctionCaller;
use rooch_config::{BitcoinRelayerConfig, EthereumRelayerConfig};
use rooch_event::actor::{EventActor, EventActorSubscribeMessage};
use rooch_event::event::ServiceStatusEvent;
use rooch_executor::proxy::ExecutorProxy;
use rooch_pipeline_processor::proxy::PipelineProcessorProxy;
use rooch_types::bitcoin::pending_block::PendingBlockModule;
use rooch_types::multichain_id::RoochMultiChainID;
use rooch_types::service_status::ServiceStatus;
use rooch_types::transaction::{L1BlockWithBody, L1Transaction};
use std::ops::Deref;
use tracing::{debug, error, info, warn};

pub struct RelayerActor {
    relayers: Vec<RelayerProxy>,
    executor: ExecutorProxy,
    processor: PipelineProcessorProxy,
    ethereum_config: Option<EthereumRelayerConfig>,
    bitcoin_config: Option<BitcoinRelayerConfig>,
    event_actor: Option<LocalActorRef<EventActor>>,
    paused: bool,
}

impl RelayerActor {
    pub async fn new(
        executor: ExecutorProxy,
        processor: PipelineProcessorProxy,
        ethereum_config: Option<EthereumRelayerConfig>,
        bitcoin_config: Option<BitcoinRelayerConfig>,
        event_actor: Option<LocalActorRef<EventActor>>,
    ) -> Result<Self> {
        Ok(Self {
            relayers: vec![],
            executor,
            processor,
            ethereum_config,
            bitcoin_config,
            event_actor,
            paused: false,
        })
    }

    pub async fn subscribe_event(
        &self,
        event_actor_ref: LocalActorRef<EventActor>,
        executor_actor_ref: LocalActorRef<RelayerActor>,
    ) {
        let service_status_event = ServiceStatusEvent::default();
        let actor_subscribe_message = EventActorSubscribeMessage::new(
            service_status_event,
            "relayer".to_string(),
            Box::new(executor_actor_ref),
        );
        let _ = event_actor_ref.send(actor_subscribe_message).await;
    }

    async fn init_relayer(&mut self, ctx: &mut ActorContext) -> Result<()> {
        if let Some(ethereum_config) = &self.ethereum_config {
            let eth_relayer = EthereumRelayer::new(ethereum_config.clone())?;
            let eth_relayer_actor_ref = ctx.spawn("eth_relayer".into(), eth_relayer).await?;
            self.relayers
                .push(RelayerProxy::ethereum(eth_relayer_actor_ref.into()));
            info!("EthereumRelayer started")
        }

        if let Some(bitcoin_config) = &self.bitcoin_config {
            let bitcoin_client = BitcoinClientActor::new(
                &bitcoin_config.btc_rpc_url,
                &bitcoin_config.btc_rpc_user_name,
                &bitcoin_config.btc_rpc_password,
            )?;
            let bitcoin_client_actor_ref =
                ctx.spawn("bitcoin_client".into(), bitcoin_client).await?;
            let bitcoin_client_proxy = BitcoinClientProxy::new(bitcoin_client_actor_ref.into());
            let bitcoin_relayer = BitcoinRelayer::new(
                bitcoin_config.clone(),
                bitcoin_client_proxy,
                self.executor.clone(),
            )?;
            let bitcoin_relayer_actor_ref =
                ctx.spawn("bitcoin_relayer".into(), bitcoin_relayer).await?;
            self.relayers
                .push(RelayerProxy::bitcoin(bitcoin_relayer_actor_ref.into()));
            info!("BitcoinRelayer started")
        }
        Ok(())
    }

    async fn handle_l1_block(&mut self, l1_block: L1BlockWithBody) -> Result<()> {
        let block_hash = hex::encode(&l1_block.block.block_hash);
        let block_height = l1_block.block.block_height;
        let result = self.processor.execute_l1_block(l1_block.clone()).await?;

        match result.execution_info.status {
            KeptVMStatus::Executed => {
                info!(
                    "Relayer execute relay block(hash: {}, height: {}) success",
                    block_hash, block_height
                );
            }
            _ => {
                //TODO should we stop the service if the relayer failed
                error!(
                    "Relayer execute relay block(hash: {}, height: {}) failed, status: {:?}",
                    block_hash, block_height, result.execution_info.status
                );
            }
        }
        Ok(())
    }

    async fn handle_l1_tx(&mut self, l1_tx: L1Transaction) -> Result<()> {
        let txid = hex::encode(&l1_tx.txid);
        let result = self.processor.execute_l1_tx(l1_tx).await?;

        match result.execution_info.status {
            KeptVMStatus::Executed => {
                info!("Relayer execute relay tx(txid: {}) success", txid);
            }
            _ => {
                error!(
                    "Relayer execute relay tx(txid: {}) failed, status: {:?}",
                    txid, result.execution_info.status
                );
            }
        }
        Ok(())
    }

    //We migrate this function from Relayer to here
    //Becase the relayer actor will blocked when sync block
    //TODO refactor the relayer, put the sync task in a separate actor
    fn get_ready_l1_txs(&self, relayer: &RelayerProxy) -> Result<Vec<L1Transaction>> {
        if relayer.is_bitcoin() {
            self.get_ready_l1_txs_bitcoin()
        } else {
            Ok(vec![])
        }
    }

    fn get_ready_l1_txs_bitcoin(&self) -> Result<Vec<L1Transaction>> {
        let pending_block_module = self.executor.as_module_binding::<PendingBlockModule>();
        let pending_txs = pending_block_module.get_ready_pending_txs()?;
        match pending_txs {
            Some(pending_txs) => {
                let block_hash = pending_txs.block_hash;
                let mut txs = pending_txs.txs;
                if txs.len() > 1 {
                    // move coinbase tx to the end
                    let coinbase_tx = txs.remove(0);
                    txs.push(coinbase_tx);
                }
                let l1_txs = txs
                    .into_iter()
                    .map(|txid| {
                        L1Transaction::new(
                            RoochMultiChainID::Bitcoin.multichain_id(),
                            block_hash.to_vec(),
                            txid.to_vec(),
                        )
                    })
                    .collect();
                Ok(l1_txs)
            }
            None => Ok(vec![]),
        }
    }

    async fn sync(&mut self) {
        let relayers = self.relayers.clone();
        for relayer in relayers {
            let relayer_name = relayer.name();

            loop {
                if self.paused {
                    debug!("Relayer {} is paused, skip sync", relayer_name);
                    break;
                }

                let mut break_flag = false;

                match relayer.get_ready_l1_block().await {
                    Ok(Some(l1_block)) => {
                        if let Err(err) = self.handle_l1_block(l1_block).await {
                            warn!("Relayer {} error: {:?}", relayer_name, err);
                        }
                    }
                    Ok(None) => {
                        //skip
                        break_flag = true;
                    }
                    Err(err) => {
                        warn!("Relayer {} error: {:?}", relayer_name, err);
                        break_flag = true;
                    }
                }

                // Notify the relayer to sync the latest block
                // The sync task will block the relayer actor, but call sync() will not block this actor
                // It a notify call.
                if let Err(e) = relayer.sync().await {
                    warn!("Relayer {} sync error: {:?}", relayer_name, e);
                }

                // Execute all ready l1 txs
                match self.get_ready_l1_txs(&relayer) {
                    Ok(txs) => {
                        for tx in txs {
                            if let Err(err) = self.handle_l1_tx(tx).await {
                                warn!("Relayer {} error: {:?}", relayer_name, err);
                            }
                        }
                    }
                    Err(err) => {
                        warn!("Relayer {} error: {:?}", relayer_name, err);
                        break_flag = true;
                    }
                }

                if break_flag {
                    break;
                }
            }
        }
    }
}

#[async_trait]
impl Actor for RelayerActor {
    async fn started(&mut self, ctx: &mut ActorContext) {
        if let Err(err) = self.init_relayer(ctx).await {
            error!("Relayer init error: {:?}", err);
        }

        let local_actor_ref: LocalActorRef<Self> = ctx.actor_ref();
        if let Some(event_actor) = self.event_actor.clone() {
            let _ = self.subscribe_event(event_actor, local_actor_ref).await;
        }
    }
}

#[async_trait]
impl Handler<RelayTick> for RelayerActor {
    async fn handle(&mut self, _message: RelayTick, _ctx: &mut ActorContext) {
        self.sync().await
    }
}

#[async_trait]
impl Handler<EventData> for RelayerActor {
    async fn handle(&mut self, message: EventData, _ctx: &mut ActorContext) -> Result<()> {
        if let Ok(service_status_event) = message.data.downcast::<ServiceStatusEvent>() {
            if service_status_event.deref().status == ServiceStatus::Maintenance {
                tracing::warn!("RelayerActor: MoveVM panic occurs, set the status to paused...");
                self.paused = true;
            }
        }
        Ok(())
    }
}
