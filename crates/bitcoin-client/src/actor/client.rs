// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::messages::{
    BroadcastTransactionMessage, GetBestBlockHashMessage, GetBlockHashMessage,
    GetBlockHeaderInfoMessage, GetBlockMessage, GetChainTipsMessage, GetRawTransactionMessage,
    GetTxOutMessage,
};
use anyhow::Result;
use async_trait::async_trait;
use bitcoin::Transaction;
use bitcoincore_rpc::{bitcoin::Txid, json, Auth, Client, RpcApi};
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use std::fs;
use std::path::PathBuf;
use tokio::time::{sleep, Duration};
use tracing::warn;

pub struct BitcoinClientActor {
    rpc_client: Client,
    max_retries: u32,
    retry_delay: Duration,
    reorg_block_store_dir: Option<PathBuf>,
}

pub struct BitcoinClientConfig {
    pub btc_rpc_url: String,
    pub btc_rpc_user_name: String,
    pub btc_rpc_password: String,
    pub local_block_store_dir: Option<PathBuf>,
}

impl BitcoinClientConfig {
    pub fn build(&self) -> Result<BitcoinClientActor> {
        BitcoinClientActor::new(
            &self.btc_rpc_url,
            &self.btc_rpc_user_name,
            &self.btc_rpc_password,
            self.local_block_store_dir.clone(),
        )
    }
}

impl BitcoinClientActor {
    pub fn new(
        btc_rpc_url: &str,
        btc_rpc_user_name: &str,
        btc_rpc_password: &str,
        local_block_store_dir: Option<PathBuf>,
    ) -> Result<Self> {
        let rpc_client = Client::new(
            btc_rpc_url,
            Auth::UserPass(btc_rpc_user_name.to_owned(), btc_rpc_password.to_owned()),
        )?;
        Ok(Self {
            rpc_client,
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            reorg_block_store_dir: local_block_store_dir,
        })
    }

    async fn retry<F, T>(&self, f: F) -> Result<T>
    where
        F: Fn() -> Result<T, bitcoincore_rpc::Error>,
    {
        let mut last_error = None;
        for _ in 0..self.max_retries {
            match f() {
                Ok(result) => return Ok(result),
                Err(e) if Self::is_network_error(&e) => {
                    warn!("Bitcoin client network error: {:?}, and retry.", e);
                    last_error = Some(e);
                    sleep(self.retry_delay).await;
                }
                Err(e) => return Err(e.into()),
            }
        }
        Err(last_error
            .map(anyhow::Error::from)
            .unwrap_or_else(|| anyhow::anyhow!("Max retries reached")))
    }

    fn is_network_error(error: &bitcoincore_rpc::Error) -> bool {
        matches!(
            error,
            bitcoincore_rpc::Error::JsonRpc(bitcoincore_rpc::jsonrpc::Error::Transport(_))
        )
    }
}

impl Actor for BitcoinClientActor {}

#[async_trait]
impl Handler<GetBlockMessage> for BitcoinClientActor {
    async fn handle(
        &mut self,
        msg: GetBlockMessage,
        _ctx: &mut ActorContext,
    ) -> Result<bitcoin::Block> {
        let GetBlockMessage { hash } = msg;

        let rpc_ret = self.retry(|| self.rpc_client.get_block(&hash)).await;
        if let Ok(block) = rpc_ret {
            return Ok(block);
        }

        let rpc_err = rpc_ret.err().unwrap(); // Safe unwrap because we are in the Err branch
        warn!("Failed to fetch block via RPC ({hash:?}): {rpc_err}");

        if let Some(store_dir) = self.reorg_block_store_dir.clone() {
            let block_file_path = store_dir.join(hash.to_string());
            return match fs::read_to_string(&block_file_path) {
                Ok(block_data) => match bitcoin::consensus::encode::deserialize_hex(&block_data) {
                    Ok(block) => Ok(block),
                    Err(err) => {
                        warn!("Failed to deserialize block from local file ({block_file_path:?}): {err}");
                        Err(anyhow::anyhow!(
                                "Failed to fetch block: RPC failed with {rpc_err}, local file deserialization failed with {err}"
                            ))
                    }
                },
                Err(err) => {
                    warn!("Failed to read block from local file ({block_file_path:?}): {err}");
                    Err(anyhow::anyhow!(
                        "Failed to fetch block: RPC failed with {rpc_err}, local file read failed with {err}"
                    ))
                }
            };
        }

        Err(anyhow::anyhow!(
            "Failed to fetch block: RPC failed with {rpc_err}, and no local store directory is configured"
        ))
    }
}

#[async_trait]
impl Handler<GetBestBlockHashMessage> for BitcoinClientActor {
    async fn handle(
        &mut self,
        msg: GetBestBlockHashMessage,
        _ctx: &mut ActorContext,
    ) -> Result<bitcoin::BlockHash> {
        let GetBestBlockHashMessage {} = msg;
        Ok(self.retry(|| self.rpc_client.get_best_block_hash()).await?)
    }
}

#[async_trait]
impl Handler<GetBlockHashMessage> for BitcoinClientActor {
    async fn handle(
        &mut self,
        msg: GetBlockHashMessage,
        _ctx: &mut ActorContext,
    ) -> Result<bitcoin::BlockHash> {
        let GetBlockHashMessage { height } = msg;
        Ok(self
            .retry(|| self.rpc_client.get_block_hash(height))
            .await?)
    }
}

#[async_trait]
impl Handler<GetBlockHeaderInfoMessage> for BitcoinClientActor {
    async fn handle(
        &mut self,
        msg: GetBlockHeaderInfoMessage,
        _ctx: &mut ActorContext,
    ) -> Result<json::GetBlockHeaderResult> {
        let GetBlockHeaderInfoMessage { hash } = msg;
        Ok(self
            .retry(|| self.rpc_client.get_block_header_info(&hash))
            .await?)
    }
}

#[async_trait]
impl Handler<GetChainTipsMessage> for BitcoinClientActor {
    async fn handle(
        &mut self,
        _msg: GetChainTipsMessage,
        _ctx: &mut ActorContext,
    ) -> Result<json::GetChainTipsResult> {
        Ok(self.retry(|| self.rpc_client.get_chain_tips()).await?)
    }
}

#[async_trait]
impl Handler<BroadcastTransactionMessage> for BitcoinClientActor {
    async fn handle(
        &mut self,
        msg: BroadcastTransactionMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Txid> {
        let BroadcastTransactionMessage {
            hex,
            maxfeerate,
            maxburnamount,
        } = msg;

        // Prepare the parameters for the RPC call
        let mut params = vec![hex.into()];

        // Add maxfeerate and maxburnamount to the params if they are Some
        if let Some(feerate) = maxfeerate {
            params.push(serde_json::to_value(feerate).unwrap());
        } else {
            params.push(serde_json::to_value(0.10).unwrap());
        }

        if let Some(burnamount) = maxburnamount {
            params.push(serde_json::to_value(burnamount).unwrap());
        } else {
            params.push(serde_json::to_value(0.0).unwrap());
        }

        // Make the RPC call
        let tx_id = self
            .retry(|| self.rpc_client.call("sendrawtransaction", &params))
            .await?;
        Ok(tx_id)
    }
}

#[async_trait]
impl Handler<GetTxOutMessage> for BitcoinClientActor {
    async fn handle(
        &mut self,
        msg: GetTxOutMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Option<json::GetTxOutResult>> {
        let GetTxOutMessage {
            txid,
            vout,
            include_mempool,
        } = msg;
        Ok(self
            .retry(|| self.rpc_client.get_tx_out(&txid, vout, include_mempool))
            .await?)
    }
}

#[async_trait]
impl Handler<GetRawTransactionMessage> for BitcoinClientActor {
    async fn handle(
        &mut self,
        msg: GetRawTransactionMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Transaction> {
        let GetRawTransactionMessage { txid } = msg;
        Ok(self
            .retry(|| self.rpc_client.get_raw_transaction(&txid, None))
            .await?)
    }
}
