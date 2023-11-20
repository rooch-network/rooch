// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::Relayer;
use anyhow::Result;
use async_trait::async_trait;
use bitcoin::block::{BlockHash, Header};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use moveos_types::transaction::FunctionCall;
use rooch_config::BitcoinRelayerConfig;
use rooch_types::framework::bitcoin_light_client::{BitcoinLightClientModule, BlockHeader};
use std::collections::HashSet;
use tracing::info;

pub struct BitcoinRelayer {
    rpc_client: Client,
    processed_blocks: HashSet<BlockHash>,
}

impl BitcoinRelayer {
    pub fn new(config: BitcoinRelayerConfig) -> Result<Self> {
        let rpc = Client::new(
            config.btc_rpc_url.as_str(),
            Auth::UserPass(config.btc_rpc_user_name, config.btc_rpc_password),
        )?;
        Ok(Self {
            rpc_client: rpc,
            processed_blocks: HashSet::new(),
        })
    }
}

#[async_trait]
impl Relayer for BitcoinRelayer {
    async fn relay(&mut self) -> Result<Option<FunctionCall>> {
        let block_hash = self.rpc_client.get_best_block_hash()?;

        if self.processed_blocks.contains(&block_hash) {
            info!("The block {} has already been processed", block_hash);
            return Ok(None);
        }

        let best_block_header: Header = self.rpc_client.get_block_header(&block_hash)?;
        let block_header = BlockHeader::from(best_block_header);
        let call = BitcoinLightClientModule::create_submit_new_block_call(&block_header);
        info!(
            "BitcoinRelayer process block, hash: {}, time: {}",
            block_hash, block_header.time
        );
        self.processed_blocks.insert(block_hash);
        Ok(Some(call))
    }
}
