// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::bitcoin_relayer::BitcoinRelayer;
use super::ethereum_relayer::EthereumRelayer;
use super::messages::RelayTick;
use crate::actor::bitcoin_client::BitcoinClientActor;
use crate::actor::bitcoin_client_proxy::BitcoinClientProxy;
use crate::{Relayer, TxSubmiter};
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::system::ActorSystem;
use coerce::actor::{context::ActorContext, message::Handler, Actor, IntoActor};
use moveos_types::{gas_config::GasConfig, transaction::MoveAction};
use rooch_config::{BitcoinRelayerConfig, EthereumRelayerConfig};
use rooch_executor::proxy::ExecutorProxy;
use rooch_rpc_api::jsonrpc_types::KeptVMStatusView;
use rooch_rpc_client::ClientBuilder;
use rooch_types::{
    address::RoochAddress,
    crypto::RoochKeyPair,
    transaction::{rooch::RoochTransactionData, AbstractTransaction},
};
use tracing::{info, warn};

pub struct RelayerActor {
    chain_id: u64,
    relayer_address: RoochAddress,
    max_gas_amount: u64,
    relayer_key: RoochKeyPair,
    tx_submiter: Box<dyn TxSubmiter>,
    relayers: Vec<Box<dyn Relayer>>,
}

impl RelayerActor {
    /// Create a new RelayerActor, use rooch_rpc_client::Client as TxSubmiter
    pub async fn new_for_client(
        executor: ExecutorProxy,
        relayer_key: RoochKeyPair,
        ethereum_config: Option<EthereumRelayerConfig>,
        bitcoin_config: Option<BitcoinRelayerConfig>,
        rooch_rpc_url: &str,
    ) -> Result<Self> {
        let rooch_rpc_client = ClientBuilder::default().build(rooch_rpc_url).await?;
        Self::new(
            executor,
            relayer_key,
            ethereum_config,
            bitcoin_config,
            rooch_rpc_client,
        )
        .await
    }

    pub async fn new<T: TxSubmiter + 'static>(
        executor: ExecutorProxy,
        relayer_key: RoochKeyPair,
        ethereum_config: Option<EthereumRelayerConfig>,
        bitcoin_config: Option<BitcoinRelayerConfig>,
        // bitcoin_client_proxy: Option<BitcoinClientProxy>,
        tx_submiter: T,
    ) -> Result<Self> {
        let chain_id = tx_submiter.get_chain_id().await?;
        let relayer_address = relayer_key.public().address();
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
                BitcoinRelayer::new(bitcoin_config, bitcoin_client_proxy, executor)?;
            relayers.push(Box::new(bitcoin_relayer));
        }

        Ok(Self {
            chain_id,
            relayer_address,
            max_gas_amount: GasConfig::DEFAULT_MAX_GAS_AMOUNT * 200,
            relayer_key,
            relayers,
            tx_submiter: Box::new(tx_submiter),
        })
    }

    async fn sync(&mut self) -> Result<()> {
        for relayer in &mut self.relayers {
            let relayer_name = relayer.name();
            loop {
                match relayer.relay().await {
                    Ok(Some(function_call)) => {
                        let sequence_number = self
                            .tx_submiter
                            .get_sequence_number(self.relayer_address)
                            .await?;
                        let action = MoveAction::Function(function_call);
                        let tx_data = RoochTransactionData::new(
                            self.relayer_address,
                            sequence_number,
                            self.chain_id,
                            self.max_gas_amount,
                            action,
                        );
                        let tx = tx_data.clone().sign(&self.relayer_key);
                        let tx_hash = tx.tx_hash();
                        let result = self.tx_submiter.submit_tx(tx).await?;
                        match result.execution_info.status {
                            KeptVMStatusView::Executed => {
                                info!("Relayer execute relay tx({:?}) success", tx_hash);
                            }
                            _ => {
                                warn!(
                                    "Relayer execute relay tx({:?}) failed, tx_data: {:?},  status: {:?}",
                                    tx_hash, tx_data, result.execution_info.status
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
