// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::bitcoin_relayer::BitcoinRelayer;
use super::ethereum_relayer::EthereumRelayer;
use super::messages::RelayTick;
use crate::actor::bitcoin_client::BitcoinClientActor;
use crate::actor::bitcoin_client_proxy::BitcoinClientProxy;
use crate::Relayer;
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::system::ActorSystem;
use coerce::actor::{context::ActorContext, message::Handler, Actor, IntoActor};
use move_core_types::account_address::AccountAddress;
use move_core_types::vm_status::KeptVMStatus;
use moveos_types::gas_config::GasConfig;
use moveos_types::moveos_std::tx_context::TxContext;
use rooch_config::{BitcoinRelayerConfig, EthereumRelayerConfig};
use rooch_executor::proxy::ExecutorProxy;
use rooch_pipeline_processor::proxy::PipelineProcessorProxy;
use rooch_types::crypto::RoochKeyPair;
use tracing::{error, info, warn};

pub struct RelayerActor {
    relayer_address: AccountAddress,
    max_gas_amount: u64,
    relayers: Vec<Box<dyn Relayer>>,
    executor: ExecutorProxy,
    processor: PipelineProcessorProxy,
}

impl RelayerActor {
    pub async fn new(
        executor: ExecutorProxy,
        processor: PipelineProcessorProxy,
        relayer_key: RoochKeyPair,
        ethereum_config: Option<EthereumRelayerConfig>,
        bitcoin_config: Option<BitcoinRelayerConfig>,
    ) -> Result<Self> {
        let relayer_address = relayer_key.public().address().into();
        let mut relayers: Vec<Box<dyn Relayer>> = vec![];
        if let Some(ethereum_config) = ethereum_config {
            let eth_relayer = EthereumRelayer::new(ethereum_config)?;
            relayers.push(Box::new(eth_relayer));
        }

        if let Some(bitcoin_config) = bitcoin_config {
            let actor_system = ActorSystem::global_system();
            let bitcoin_client_actor = BitcoinClientActor::new(bitcoin_config.clone())?
                .into_actor(Some("BitcoinClient"), &actor_system)
                .await?;
            let bitcoin_client_proxy = BitcoinClientProxy::new(bitcoin_client_actor.into());
            let bitcoin_relayer =
                BitcoinRelayer::new(bitcoin_config, bitcoin_client_proxy, executor.clone())?;
            relayers.push(Box::new(bitcoin_relayer));
        }

        Ok(Self {
            relayer_address,
            max_gas_amount: GasConfig::DEFAULT_MAX_GAS_AMOUNT * 1000,
            relayers,
            executor,
            processor,
        })
    }

    async fn sync(&mut self) -> Result<()> {
        for relayer in &mut self.relayers {
            let relayer_name = relayer.name();
            loop {
                match relayer.relay().await {
                    Ok(Some(l1_block)) => {
                        let sequence_number = self
                            .executor
                            .get_sequence_number(self.relayer_address)
                            .await?;
                        let tx_hash = l1_block.block.tx_hash();
                        let ctx = TxContext::new(
                            self.relayer_address,
                            sequence_number,
                            self.max_gas_amount,
                            tx_hash,
                            l1_block.block.tx_size(),
                        );
                        let block_hash = hex::encode(&l1_block.block.block_hash);
                        let block_height = l1_block.block.block_height;
                        let result = self.processor.execute_l1_block(ctx, l1_block).await?;

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
                                break;
                            }
                        }
                    }
                    Ok(None) => {
                        //skip
                        break;
                    }
                    Err(err) => {
                        warn!("Relayer {} error: {:?}", relayer_name, err);
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Actor for RelayerActor {
    async fn started(&mut self, _ctx: &mut ActorContext) {}
}

#[async_trait]
impl Handler<RelayTick> for RelayerActor {
    async fn handle(&mut self, _message: RelayTick, _ctx: &mut ActorContext) {
        if let Err(err) = self.sync().await {
            warn!("Relayer tick task error: {:?}", err);
        }
    }
}
