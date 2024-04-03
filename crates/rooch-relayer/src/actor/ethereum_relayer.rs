// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::Relayer;
use anyhow::Result;
use async_trait::async_trait;
use ethers::prelude::*;
use rooch_config::EthereumRelayerConfig;
use rooch_types::{
    framework::ethereum_light_client::BlockHeader,
    multichain_id::RoochMultiChainID,
    transaction::{L1Block, L1BlockWithBody},
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
                    "EthereumRelayer process block, hash: {}, number: {}, timestamp: {}",
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
impl Relayer for EthereumRelayer {
    async fn relay(&mut self) -> Result<Option<L1BlockWithBody>> {
        self.relay_ethereum().await
    }
}
