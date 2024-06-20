// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::messages::{GetReadyL1BlockMessage, GetReadyL1TxsMessage, SyncTick};
use crate::actor::bitcoin_client_proxy::BitcoinClientProxy;
use anyhow::Result;
use async_trait::async_trait;
use bitcoin::hashes::Hash;
use bitcoin::Block;
use bitcoincore_rpc::bitcoincore_rpc_json::GetBlockHeaderResult;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use moveos_types::module_binding::MoveFunctionCaller;
use rooch_config::BitcoinRelayerConfig;
use rooch_executor::proxy::ExecutorProxy;
use rooch_types::{
    bitcoin::{pending_block::PendingBlockModule, BitcoinModule},
    multichain_id::RoochMultiChainID,
    transaction::{L1Block, L1BlockWithBody, L1Transaction},
};
use tracing::{debug, error, info};

pub struct BitcoinRelayer {
    genesis_block_height: u64,
    // only for data import
    end_block_height: Option<u64>,
    rpc_client: BitcoinClientProxy,
    move_caller: ExecutorProxy,
    buffer: Vec<BlockResult>,
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
    pub fn new(
        config: BitcoinRelayerConfig,
        rpc_client: BitcoinClientProxy,
        executor: ExecutorProxy,
    ) -> Result<Self> {
        let bitcoin_module = executor.as_module_binding::<BitcoinModule>();
        let genesis_block_height = bitcoin_module.get_genesis_block_height()?;
        let sync_block_interval = config.btc_sync_block_interval.unwrap_or(60u64);

        Ok(Self {
            genesis_block_height,
            end_block_height: config.btc_end_block_height,
            rpc_client,
            move_caller: executor,
            buffer: vec![],
            sync_block_interval,
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
        let pending_block_module = self.move_caller.as_module_binding::<PendingBlockModule>();
        let latest_block_height_in_rooch = pending_block_module.get_latest_block_height()?;
        let latest_block_hash_in_bitcoin = self.rpc_client.get_best_block_hash().await?;
        let latest_block_header_info = self
            .rpc_client
            .get_block_header_info(latest_block_hash_in_bitcoin)
            .await?;
        let latest_block_height_in_bitcoin = latest_block_header_info.height as u64;
        let start_block_height: u64 = match latest_block_height_in_rooch {
            Some(latest_block_height_in_rooch) => latest_block_height_in_rooch + 1,
            None => {
                // if the latest block height in rooch is None, then the genesis block height should be used
                self.genesis_block_height
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
            let start_block_hash = self.rpc_client.get_block_hash(start_block_height).await?;
            self.rpc_client
                .get_block_header_info(start_block_hash)
                .await?
        };

        let start_block = self
            .rpc_client
            .get_block(start_block_header_info.hash)
            .await?;

        let batch_size: usize = 10;
        let mut next_block_hash = start_block_header_info.next_block_hash;
        // only for bitcoin block data import
        if !(end_block_height > 0 && start_block_height_usize > end_block_height) {
            self.buffer.push(BlockResult {
                header_info: start_block_header_info,
                block: start_block,
            });
        };
        while let Some(next_hash) = next_block_hash {
            let header_info = self.rpc_client.get_block_header_info(next_hash).await?;
            let block = self.rpc_client.get_block(next_hash).await?;
            next_block_hash = header_info.next_block_hash;
            let next_block_height = header_info.height;

            // only for bitcoin block data import
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

    fn pop_buffer(&mut self) -> Result<Option<L1BlockWithBody>> {
        if self.buffer.is_empty() {
            Ok(None)
        } else {
            let block_result = self.buffer.remove(0);
            let block_height = block_result.header_info.height;
            let block_hash = block_result.header_info.hash;
            let time = block_result.block.header.time;
            let tx_size = block_result.block.txdata.len();
            info!(
                "BitcoinRelayer process block, height: {}, hash: {}, tx_size: {}, time: {}",
                block_height, block_hash, tx_size, time
            );
            debug!("GetBlockHeaderResult: {:?}", block_result);

            let block_height = block_result.header_info.height;
            let block_body = rooch_types::bitcoin::types::Block::from(block_result.block);

            Ok(Some(L1BlockWithBody {
                block: L1Block {
                    chain_id: RoochMultiChainID::Bitcoin.multichain_id(),
                    block_height: block_height as u64,
                    block_hash: block_hash.to_byte_array().to_vec(),
                },
                block_body: block_body.encode(),
            }))
        }
    }

    fn get_ready_l1_txs(&mut self) -> Result<Vec<L1Transaction>> {
        let pending_block_module = self.move_caller.as_module_binding::<PendingBlockModule>();
        let pending_txs = pending_block_module.get_ready_pending_txs()?;
        match pending_txs {
            Some(pending_txs) => {
                let block_hash = pending_txs.block_hash;
                let mut txs = pending_txs.txs;
                if txs.len() > 1 {
                    // move coinbase tx to the end
                    let coinbase_tx = txs.remove(0);
                    txs.push(coinbase_tx);
                }
                let l1_txs = txs
                    .into_iter()
                    .map(|txid| {
                        L1Transaction::new(
                            RoochMultiChainID::Bitcoin.multichain_id(),
                            block_hash.to_vec(),
                            txid.to_vec(),
                        )
                    })
                    .collect();
                Ok(l1_txs)
            }
            None => Ok(vec![]),
        }
    }
}

#[async_trait]
impl Actor for BitcoinRelayer {
    async fn started(&mut self, _ctx: &mut ActorContext) {}
}

#[async_trait]
impl Handler<SyncTick> for BitcoinRelayer {
    async fn handle(&mut self, _message: SyncTick, _ctx: &mut ActorContext) {
        if let Err(e) = self.sync_block().await {
            error!("BitcoinRelayer sync block error: {:?}", e);
        }
    }
}

#[async_trait]
impl Handler<GetReadyL1BlockMessage> for BitcoinRelayer {
    async fn handle(
        &mut self,
        _message: GetReadyL1BlockMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Option<L1BlockWithBody>> {
        self.pop_buffer()
    }
}

#[async_trait]
impl Handler<GetReadyL1TxsMessage> for BitcoinRelayer {
    async fn handle(
        &mut self,
        _message: GetReadyL1TxsMessage,
        _ctx: &mut ActorContext,
    ) -> Result<Vec<L1Transaction>> {
        self.get_ready_l1_txs()
    }
}
