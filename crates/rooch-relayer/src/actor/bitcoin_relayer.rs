// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::Relayer;
use anyhow::Result;
use async_trait::async_trait;
use bitcoin::hashes::Hash;
use bitcoincore_rpc::{bitcoincore_rpc_json::GetBlockHeaderResult, Auth, Client, RpcApi};
use moveos_types::{module_binding::MoveFunctionCaller, transaction::FunctionCall};
use rooch_config::BitcoinRelayerConfig;
use rooch_executor::proxy::ExecutorProxy;
use rooch_types::framework::bitcoin_light_client::{BitcoinLightClientModule, BlockHeader};
use std::cmp::max;
use tracing::{debug, info};

pub struct BitcoinRelayer {
    start_block_height: Option<u64>,
    rpc_client: Client,
    //TODO if we want make the relayer to an independent process, we need to replace the executor proxy with a rooch rpc client
    move_caller: ExecutorProxy,
    buffer: Vec<GetBlockHeaderResult>,
}

impl BitcoinRelayer {
    pub fn new(config: BitcoinRelayerConfig, executor: ExecutorProxy) -> Result<Self> {
        let rpc = Client::new(
            config.btc_rpc_url.as_str(),
            Auth::UserPass(config.btc_rpc_user_name, config.btc_rpc_password),
        )?;
        Ok(Self {
            start_block_height: config.btc_start_block_height,
            rpc_client: rpc,
            move_caller: executor,
            buffer: vec![],
        })
    }

    async fn sync_block(&mut self) -> Result<()> {
        if !self.buffer.is_empty() {
            return Ok(());
        }
        let bitcoin_light_client = self
            .move_caller
            .as_module_binding::<BitcoinLightClientModule>();
        let latest_block_height_in_rooch = bitcoin_light_client.get_latest_block_height()?;
        let latest_block_hash_in_bitcoin = self.rpc_client.get_best_block_hash()?;
        let latest_block_header = self
            .rpc_client
            .get_block_header_info(&latest_block_hash_in_bitcoin)?;
        let latest_block_height_in_bitcoin = latest_block_header.height as u64;
        let start_block_height: u64 = match (self.start_block_height, latest_block_height_in_rooch)
        {
            (Some(start_block_height), Some(latest_block_height_in_rooch)) => {
                max(start_block_height, latest_block_height_in_rooch + 1)
            }
            (Some(start_block_height), None) => start_block_height,
            (None, Some(latest_block_height_in_rooch)) => latest_block_height_in_rooch + 1,
            (None, None) => {
                //if the start_block_height is None, and the latest_block_height_in_rooch is None
                //we sync from the latest block
                latest_block_height_in_bitcoin
            }
        };

        if start_block_height > latest_block_height_in_bitcoin {
            return Ok(());
        }

        let start_block = if start_block_height == latest_block_height_in_bitcoin {
            latest_block_header
        } else {
            let start_block_hash = self.rpc_client.get_block_hash(start_block_height)?;
            self.rpc_client.get_block_header_info(&start_block_hash)?
        };

        let batch_size: usize = 10;
        let mut next_block_hash = start_block.next_block_hash;
        self.buffer.push(start_block);
        while let Some(next_hash) = next_block_hash {
            let block_result = self.rpc_client.get_block_header_info(&next_hash)?;
            next_block_hash = block_result.next_block_hash;
            self.buffer.push(block_result);
            if self.buffer.len() > batch_size {
                break;
            }
        }
        Ok(())
    }

    fn pop_buffer(&mut self) -> Result<Option<FunctionCall>> {
        if self.buffer.is_empty() {
            Ok(None)
        } else {
            let block_header_result = self.buffer.remove(0);
            let block_height = block_header_result.height;
            let block_hash = block_header_result.hash;
            let time = block_header_result.time;
            info!(
                "BitcoinRelayer process block, height: {}, hash: {}, time: {}",
                block_height, block_hash, time
            );
            debug!("GetBlockHeaderResult: {:?}", block_header_result);
            let call = block_result_to_call(block_header_result)?;
            Ok(Some(call))
        }
    }
}

#[async_trait]
impl Relayer for BitcoinRelayer {
    async fn relay(&mut self) -> Result<Option<FunctionCall>> {
        self.sync_block().await?;
        self.pop_buffer()
    }
}

fn block_result_to_call(block_result: GetBlockHeaderResult) -> Result<FunctionCall> {
    let block_height = block_result.height;
    let block_hash = block_result.hash;
    let block_header = BlockHeader::try_from(block_result)?;
    let call = BitcoinLightClientModule::create_submit_new_block_call(
        block_height as u64,
        block_hash.to_byte_array().to_vec(),
        &block_header,
    );
    Ok(call)
}
