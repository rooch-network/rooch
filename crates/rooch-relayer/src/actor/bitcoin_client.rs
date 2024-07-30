// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::messages::{
    BroadcastTransactionMessage, GetBestBlockHashMessage, GetBlockHashMessage,
    GetBlockHeaderInfoMessage, GetBlockMessage,
};
use anyhow::Result;
use async_trait::async_trait;
use bitcoincore_rpc::{bitcoin::Txid, json, Auth, Client, RpcApi};
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use rooch_config::BitcoinRelayerConfig;
use tracing::info;

use super::messages::GetChainTipsMessage;

pub struct BitcoinClientActor {
    rpc_client: Client,
}

impl BitcoinClientActor {
    pub fn new(config: BitcoinRelayerConfig) -> Result<Self> {
        let rpc_client = Client::new(
            config.btc_rpc_url.as_str(),
            Auth::UserPass(config.btc_rpc_user_name, config.btc_rpc_password),
        )?;
        Ok(Self { rpc_client })
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

        Ok(self.rpc_client.get_block(&hash)?)
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

        Ok(self.rpc_client.get_best_block_hash()?)
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

        Ok(self.rpc_client.get_block_hash(height)?)
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

        Ok(self.rpc_client.get_block_header_info(&hash)?)
    }
}

#[async_trait]
impl Handler<GetChainTipsMessage> for BitcoinClientActor {
    async fn handle(
        &mut self,
        _msg: GetChainTipsMessage,
        _ctx: &mut ActorContext,
    ) -> Result<json::GetChainTipsResult> {
        Ok(self.rpc_client.get_chain_tips()?)
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
        let tx_id: bitcoin::Txid = self.rpc_client.call("sendrawtransaction", &params)?;
        Ok(tx_id)
    }
}
