// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::ethereum_relayer::EthereumRelayer;
use super::messages::RelayTick;
use crate::{Relayer, TxSubmiter};
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use moveos_types::{gas_config::GasConfig, transaction::MoveAction};
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
        relayer_key: RoochKeyPair,
        eth_rpc_url: &str,
        rooch_rpc_url: &str,
    ) -> Result<Self> {
        let rooch_rpc_client = ClientBuilder::default().build(rooch_rpc_url).await?;
        Self::new(relayer_key, eth_rpc_url, rooch_rpc_client).await
    }

    pub async fn new<T: TxSubmiter + 'static>(
        relayer_key: RoochKeyPair,
        eth_rpc_url: &str,
        tx_submiter: T,
    ) -> Result<Self> {
        let chain_id = tx_submiter.get_chain_id().await?;
        let relayer_address = relayer_key.public().address();
        let eth_relayer = EthereumRelayer::new(eth_rpc_url)?;
        let relayers: Vec<Box<dyn Relayer>> = vec![Box::new(eth_relayer)];
        Ok(Self {
            chain_id,
            relayer_address,
            max_gas_amount: GasConfig::DEFAULT_MAX_GAS_AMOUNT,
            relayer_key,
            relayers,
            tx_submiter: Box::new(tx_submiter),
        })
    }

    async fn tick(&mut self) -> Result<()> {
        for relayer in &mut self.relayers {
            let relayer_name = relayer.name();
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
                    let tx = tx_data.sign(&self.relayer_key);
                    let tx_hash = tx.tx_hash();
                    let result = self.tx_submiter.submit_tx(tx).await?;
                    match result.execution_info.status {
                        KeptVMStatusView::Executed => {
                            info!(
                                "Relayer {} execute relay tx({}) success",
                                relayer_name, tx_hash
                            );
                        }
                        _ => {
                            warn!(
                                "Relayer {} execute relay tx({}) failed, status: {:?}",
                                relayer_name, tx_hash, result.execution_info.status
                            );
                        }
                    }
                }
                Ok(None) => {
                    //skip
                }
                Err(err) => {
                    warn!("Relayer {} error: {:?}", relayer_name, err);
                }
            }
        }
        Ok(())
    }
}

impl Actor for RelayerActor {}

#[async_trait]
impl Handler<RelayTick> for RelayerActor {
    async fn handle(&mut self, _message: RelayTick, _ctx: &mut ActorContext) {
        if let Err(err) = self.tick().await {
            warn!("Relayer tick task error: {:?}", err);
        }
    }
}
