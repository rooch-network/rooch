// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::messages::{GetReadyL1BlockMessage, GetReadyL1TxsMessage, SyncTick};
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use ethers::prelude::*;
use rooch_config::EthereumRelayerConfig;
use rooch_types::{
    framework::ethereum::BlockHeader,
    multichain_id::RoochMultiChainID,
    transaction::{L1Block, L1BlockWithBody, L1Transaction},
};
use std::collections::HashSet;
use tracing::info;

pub struct EthereumRelayer {
    rpc_client: Provider<Http>,
    processed_blocks: HashSet<H256>,
}

impl EthereumRelayer {
    pub fn new(config: EthereumRelayerConfig) -> Result<Self> {
        let rpc_client = Provider::<Http>::try_from(config.eth_rpc_url)?;
        Ok(Self {
            rpc_client,
            //TODO load processed block from Move state
            processed_blocks: HashSet::new(),
        })
    }

    async fn relay_ethereum(&mut self) -> Result<Option<L1BlockWithBody>> {
        let block = self
            .rpc_client
            .get_block(BlockId::Number(BlockNumber::Latest))
            .await?;
        match block {
            Some(block) => {
                let block_hash = block
                    .hash
                    .ok_or_else(|| anyhow::format_err!("The block is a pending block"))?;
                if self.processed_blocks.contains(&block_hash) {
                    info!("The block {} has already been processed", block_hash);
                    return Ok(None);
                }
                let block_header = BlockHeader::try_from(&block)?;
                info!(
                    "EthereumRelayer process block, hash: {:?}, number: {}, timestamp: {}",
                    block_hash, block_header.number, block_header.timestamp
                );
                let l1_block = L1BlockWithBody {
                    block: L1Block {
                        chain_id: RoochMultiChainID::Ether.multichain_id(),
                        block_height: block_header.number,
                        block_hash: block_hash.as_bytes().to_vec(),
                    },
                    block_body: block_header.encode(),
                };
                self.processed_blocks.insert(block_hash);
                Ok(Some(l1_block))
            }
            None => {
                info!("The RPC returned no block");
                Ok(None)
            }
        }
        //TODO clean up processed block
    }
}

#[async_trait]
impl Actor for EthereumRelayer {
    async fn started(&mut self, _ctx: &mut ActorContext) {}
}

#[async_trait]
impl Handler<SyncTick> for EthereumRelayer {
    async fn handle(&mut self, _message: SyncTick, _ctx: &mut ActorContext) {
        //TODO support buffer block
    }
}

#[async_trait]
impl Handler<GetReadyL1BlockMessage> for EthereumRelayer {
    async fn handle(
        &mut self,
        _message: GetReadyL1BlockMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Option<L1BlockWithBody>> {
        self.relay_ethereum().await
    }
}

#[async_trait]
impl Handler<GetReadyL1TxsMessage> for EthereumRelayer {
    async fn handle(
        &mut self,
        _message: GetReadyL1TxsMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<L1Transaction>> {
        //TODO
        Ok(vec![])
    }
}
