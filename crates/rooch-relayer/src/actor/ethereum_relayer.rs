// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::Relayer;
use anyhow::Result;
use async_trait::async_trait;
use ethers::prelude::*;
use moveos_types::transaction::FunctionCall;
use rooch_config::EthereumRelayerConfig;
use rooch_types::framework::ethereum_light_client::{BlockHeader, EthereumLightClientModule};
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

    async fn relay_ethereum(&mut self) -> Result<Option<FunctionCall>> {
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
                let call = EthereumLightClientModule::create_submit_new_block_call(&block_header);
                info!(
                    "EthereumRelayer process block, hash: {}, number: {}, timestamp: {}",
                    block_hash, block_header.number, block_header.timestamp
                );
                self.processed_blocks.insert(block_hash);
                Ok(Some(call))
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
    async fn relay(&mut self) -> Result<Option<FunctionCall>> {
        self.relay_ethereum().await
    }
}
