// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::bitcoin_client::BitcoinClientActor;
use crate::actor::messages::{
    BroadcastTransactionMessage, GetBestBlockHashMessage, GetBlockHashMessage,
    GetBlockHeaderInfoMessage, GetBlockMessage,
};
use anyhow::Result;
use bitcoincore_rpc::bitcoin::Txid;
use bitcoincore_rpc::json;
use coerce::actor::ActorRef;

use super::messages::GetChainTipsMessage;

#[derive(Clone)]
pub struct BitcoinClientProxy {
    pub actor: ActorRef<BitcoinClientActor>,
}

impl BitcoinClientProxy {
    pub fn new(actor: ActorRef<BitcoinClientActor>) -> Self {
        Self { actor }
    }

    pub async fn get_block(&self, hash: bitcoin::BlockHash) -> Result<bitcoin::Block> {
        self.actor.send(GetBlockMessage { hash }).await?
    }

    pub async fn get_best_block_hash(&self) -> Result<bitcoin::BlockHash> {
        self.actor.send(GetBestBlockHashMessage {}).await?
    }

    pub async fn get_block_hash(&self, height: u64) -> Result<bitcoin::BlockHash> {
        self.actor.send(GetBlockHashMessage { height }).await?
    }

    pub async fn get_block_header_info(
        &self,
        hash: bitcoin::BlockHash,
    ) -> Result<json::GetBlockHeaderResult> {
        self.actor.send(GetBlockHeaderInfoMessage { hash }).await?
    }

    pub async fn get_chain_tips(&self) -> Result<json::GetChainTipsResult> {
        self.actor.send(GetChainTipsMessage {}).await?
    }

    pub async fn broadcast_transaction(
        &self,
        hex: String,
        maxfeerate: Option<f64>,
        maxburnamount: Option<u64>,
    ) -> Result<Txid> {
        self.actor
            .send(BroadcastTransactionMessage {
                hex,
                maxfeerate,
                maxburnamount,
            })
            .await?
    }
}
