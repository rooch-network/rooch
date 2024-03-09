// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::Relayer;
use anyhow::Result;
use async_trait::async_trait;
use bitcoin::Block;
use bitcoincore_rpc::{bitcoincore_rpc_json::GetBlockHeaderResult, Auth, Client, RpcApi};
use moveos_types::{module_binding::MoveFunctionCaller, transaction::FunctionCall};
use rooch_config::BitcoinRelayerConfig;
use rooch_executor::proxy::ExecutorProxy;
use rooch_types::bitcoin::light_client::BitcoinLightClientModule;
use std::cmp::max;
use tracing::{debug, info};

pub struct BitcoinRelayer {
    start_block_height: Option<u64>,
    // only for data verify
    end_block_height: Option<u64>,
    rpc_client: Client,
    //TODO if we want make the relayer to an independent process, we need to replace the executor proxy with a rooch rpc client
    move_caller: ExecutorProxy,
    buffer: Vec<BlockResult>,
    tx_batch_size: u64,
    sync_block_interval: u64,
    latest_sync_timestamp: u64,
    sync_to_latest: bool,
}

#[derive(Debug, Clone)]
pub struct BlockResult {
    pub header_info: GetBlockHeaderResult,
    pub block: Block,
}

impl BitcoinRelayer {
    pub fn new(config: BitcoinRelayerConfig, executor: ExecutorProxy) -> Result<Self> {
        let rpc = Client::new(
            config.btc_rpc_url.as_str(),
            Auth::UserPass(config.btc_rpc_user_name, config.btc_rpc_password),
        )?;
        Ok(Self {
            start_block_height: config.btc_start_block_height,
            end_block_height: config.btc_end_block_height,
            rpc_client: rpc,
            move_caller: executor,
            buffer: vec![],
            tx_batch_size: 1000u64,
            sync_block_interval: 60u64,
            latest_sync_timestamp: 0u64,
            sync_to_latest: false,
        })
    }

    async fn sync_block(&mut self) -> Result<()> {
        if !self.buffer.is_empty() {
            return Ok(());
        }
        if self.sync_to_latest
            && (self.latest_sync_timestamp + self.sync_block_interval
                > chrono::Utc::now().timestamp() as u64)
        {
            return Ok(());
        }
        self.latest_sync_timestamp = chrono::Utc::now().timestamp() as u64;
        let bitcoin_light_client = self
            .move_caller
            .as_module_binding::<BitcoinLightClientModule>();
        let latest_block_height_in_rooch = bitcoin_light_client.get_latest_block_height()?;
        let latest_block_hash_in_bitcoin = self.rpc_client.get_best_block_hash()?;
        let latest_block_header_info = self
            .rpc_client
            .get_block_header_info(&latest_block_hash_in_bitcoin)?;
        let latest_block_height_in_bitcoin = latest_block_header_info.height as u64;
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
        let start_block_height_usize = start_block_height as usize;
        let end_block_height = self.end_block_height.unwrap_or(0) as usize;

        if start_block_height > latest_block_height_in_bitcoin {
            self.sync_to_latest = true;
            return Ok(());
        }

        let start_block_header_info = if start_block_height == latest_block_height_in_bitcoin {
            latest_block_header_info
        } else {
            let start_block_hash = self.rpc_client.get_block_hash(start_block_height)?;
            self.rpc_client.get_block_header_info(&start_block_hash)?
        };

        let start_block = self.rpc_client.get_block(&start_block_header_info.hash)?;

        let batch_size: usize = 10;
        let mut next_block_hash = start_block_header_info.next_block_hash;
        // only for data verify mode
        if !(end_block_height > 0 && start_block_height_usize > end_block_height) {
            self.buffer.push(BlockResult {
                header_info: start_block_header_info,
                block: start_block,
            });
        };
        while let Some(next_hash) = next_block_hash {
            let header_info = self.rpc_client.get_block_header_info(&next_hash)?;
            let block = self.rpc_client.get_block(&next_hash)?;
            next_block_hash = header_info.next_block_hash;
            let next_block_height = header_info.height;

            // only for data verify mode
            if (end_block_height > 0 && next_block_height > end_block_height)
                || next_block_height < start_block_height_usize
            {
                if end_block_height > 0 && start_block_height_usize <= end_block_height {
                    info!("BitcoinRelayer process should exit at height {} and start_block_height is {}, end_block_height is {} ", next_block_height, start_block_height_usize, end_block_height);
                };
                break;
            }
            self.buffer.push(BlockResult { header_info, block });
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
            let block_result = self.buffer.remove(0);
            let block_height = block_result.header_info.height;
            let block_hash = block_result.header_info.hash;
            let time = block_result.block.header.time;
            info!(
                "BitcoinRelayer process block, height: {}, hash: {}, time: {}",
                block_height, block_hash, time
            );
            debug!("GetBlockHeaderResult: {:?}", block_result);
            let call = block_result_to_call(block_result)?;
            Ok(Some(call))
        }
    }

    fn check_utxo_progress(&self) -> Result<Option<FunctionCall>> {
        let bitcoin_light_client = self
            .move_caller
            .as_module_binding::<BitcoinLightClientModule>();
        let remaining_tx_count = bitcoin_light_client.remaining_tx_count()?;
        if remaining_tx_count > 0 {
            let call = BitcoinLightClientModule::create_process_utxos_call(self.tx_batch_size);
            info!(
                "BitcoinRelayer process utxo, remaining tx count: {}",
                remaining_tx_count
            );
            Ok(Some(call))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl Relayer for BitcoinRelayer {
    async fn relay(&mut self) -> Result<Option<FunctionCall>> {
        if let Some(call) = self.check_utxo_progress()? {
            return Ok(Some(call));
        }
        self.sync_block().await?;
        if let Some(call) = self.pop_buffer()? {
            return Ok(Some(call));
        }
        Ok(None)
    }
}

fn block_result_to_call(block_result: BlockResult) -> Result<FunctionCall> {
    let block_height = block_result.header_info.height;
    let call = BitcoinLightClientModule::create_submit_new_block_call(
        block_height as u64,
        block_result.block,
    );
    Ok(call)
}
