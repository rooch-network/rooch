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
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use move_core_types::account_address::AccountAddress;
use move_core_types::vm_status::KeptVMStatus;
use moveos_types::moveos_std::gas_schedule::GasScheduleConfig;
use moveos_types::moveos_std::tx_context::TxContext;
use rooch_config::{BitcoinRelayerConfig, EthereumRelayerConfig};
use rooch_executor::proxy::ExecutorProxy;
use rooch_pipeline_processor::proxy::PipelineProcessorProxy;
use rooch_types::address::BitcoinAddress;
use rooch_types::crypto::RoochKeyPair;
use rooch_types::transaction::{L1BlockWithBody, L1Transaction};
use tracing::{error, info, warn};

pub struct RelayerActor {
    sequencer_address: AccountAddress,
    sequencer_bitcoin_address: BitcoinAddress,
    max_gas_amount: u64,
    relayers: Vec<RelayerProxy>,
    executor: ExecutorProxy,
    processor: PipelineProcessorProxy,
    ethereum_config: Option<EthereumRelayerConfig>,
    bitcoin_config: Option<BitcoinRelayerConfig>,
}

impl RelayerActor {
    pub async fn new(
        executor: ExecutorProxy,
        processor: PipelineProcessorProxy,
        sequencer_key: RoochKeyPair,
        ethereum_config: Option<EthereumRelayerConfig>,
        bitcoin_config: Option<BitcoinRelayerConfig>,
    ) -> Result<Self> {
        let sequencer_address = sequencer_key.public().rooch_address()?.into();

        Ok(Self {
            sequencer_address,
            sequencer_bitcoin_address: sequencer_key.public().bitcoin_address()?,
            max_gas_amount: GasScheduleConfig::INITIAL_MAX_GAS_AMOUNT * 1000,
            relayers: vec![],
            executor,
            processor,
            ethereum_config,
            bitcoin_config,
        })
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
            let bitcoin_client = BitcoinClientActor::new(bitcoin_config.clone())?;
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
        let sequence_number = self
            .executor
            .get_sequence_number(self.sequencer_address)
            .await?;
        let tx_hash = l1_block.block.tx_hash();
        let ctx = TxContext::new(
            self.sequencer_address,
            sequence_number,
            self.max_gas_amount,
            tx_hash,
            l1_block.block.tx_size(),
        );
        let block_hash = hex::encode(&l1_block.block.block_hash);
        let block_height = l1_block.block.block_height;
        let result = self
            .processor
            .execute_l1_block(
                ctx,
                l1_block.clone(),
                self.sequencer_bitcoin_address.clone(),
            )
            .await?;

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
        let sequence_number = self
            .executor
            .get_sequence_number(self.sequencer_address)
            .await?;
        let tx_hash = l1_tx.tx_hash();
        let txid = hex::encode(&l1_tx.txid);
        let ctx = TxContext::new(
            self.sequencer_address,
            sequence_number,
            self.max_gas_amount,
            tx_hash,
            l1_tx.tx_size(),
        );
        let result = self
            .processor
            .execute_l1_tx(ctx, l1_tx, self.sequencer_bitcoin_address.clone())
            .await?;

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

    async fn sync(&mut self) {
        let relayers = self.relayers.clone();
        for relayer in relayers {
            let relayer_name = relayer.name();
            if let Err(e) = relayer.sync().await {
                warn!("Relayer {} sync error: {:?}", relayer_name, e);
            }

            loop {
                match relayer.get_ready_l1_txs().await {
                    Ok(txs) => {
                        for tx in txs {
                            if let Err(err) = self.handle_l1_tx(tx).await {
                                warn!("Relayer {} error: {:?}", relayer_name, err);
                            }
                        }
                    }
                    Err(err) => {
                        warn!("Relayer {} error: {:?}", relayer_name, err);
                        break;
                    }
                }
                match relayer.get_ready_l1_block().await {
                    Ok(Some(l1_block)) => {
                        if let Err(err) = self.handle_l1_block(l1_block).await {
                            warn!("Relayer {} error: {:?}", relayer_name, err);
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
    }
}

#[async_trait]
impl Actor for RelayerActor {
    async fn started(&mut self, ctx: &mut ActorContext) {
        if let Err(err) = self.init_relayer(ctx).await {
            error!("Relayer init error: {:?}", err);
        }
    }
}

#[async_trait]
impl Handler<RelayTick> for RelayerActor {
    async fn handle(&mut self, _message: RelayTick, _ctx: &mut ActorContext) {
        self.sync().await
    }
}
