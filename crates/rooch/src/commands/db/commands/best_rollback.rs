// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use bitcoin::hashes::Hash;
use clap::Parser;
use rooch_types::da::chunk::chunk_from_segments;
use rooch_types::da::segment::{segment_from_bytes, Segment};
use rooch_types::error::RoochResult;
use rooch_types::transaction::{LedgerTransaction, LedgerTxData};
use std::collections::HashSet;

/// try to find the best tx_order for rollback caused by chain reorg
#[derive(Debug, Parser)]
pub struct BestRollbackCommand {
    #[clap(
        long = "da-url",
        help = "open-da rpc url. e.g., https://storage.googleapis.com/rooch-openda-testnet/7a6b74d2"
    )]
    pub da_url: String,

    #[clap(
        long = "last-block-number",
        help = "last_avail_block_number in da_info, got by `rooch rpc request --method rooch_status`"
    )]
    pub last_l2_block_number: u128,
    #[clap(
        long = "search-depth",
        help = "max l2 blocks search depth for rollback, default is 16. If we cannot find best rollback tx_order in search depth, increase this value",
        default_value = "16"
    )]
    pub search_depth: Option<u64>,
    #[clap(long = "main", help = "Bitcoin Mainnet or not. default is false")]
    pub main: bool,
}

impl BestRollbackCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let depth = self.search_depth.unwrap();
        let mut da_hashes =
            get_block_hash_from_da_rpc(&self.da_url, self.last_l2_block_number, depth as u128)
                .await?;
        da_hashes.sort_by(|a, b| a.block_height.cmp(&b.block_height)); // order by block_height
        if da_hashes.is_empty() {
            println!("no btc block found in DA, please increase search depth");
            return Ok(());
        }

        tracing::info!(
            "found {} btc blocks in DA, block_height range: {}-{}",
            da_hashes.len(),
            da_hashes.first().unwrap().block_height,
            da_hashes.last().unwrap().block_height
        );

        // compare block hash from DA and BTC, find the best rollback tx_order:
        // find the first mismatched block hash, and rollback to the previous tx_order.
        let mut best_tx_order = 0;
        let mut found_matched_block_hash = false;
        let mut found_best_tx_order = false;
        for da_block_hash in da_hashes.iter() {
            let block_height = da_block_hash.block_height;
            let rpc_block_hash = get_block_hash_from_btc_rpc(block_height, self.main).await?;
            if rpc_block_hash != da_block_hash.block_hash {
                if found_matched_block_hash {
                    best_tx_order = da_block_hash.previous_tx_order;
                    found_best_tx_order = true;
                    break;
                }
            } else {
                found_matched_block_hash = true;
            }
        }

        if !found_matched_block_hash {
            println!("no matched block hash found, please increase search depth");
            return Ok(());
        }

        if !found_best_tx_order {
            println!("all block hash matched, no need to rollback");
            return Ok(());
        } else {
            println!("best rollback tx_order: {}", best_tx_order);
        }

        Ok(())
    }
}

struct BTCBlockHash {
    block_height: u64,
    block_hash: String,
    previous_tx_order: u64, // tx before this l1block tx
}

fn filter_l1block(mut tx_list: Vec<LedgerTransaction>) -> Vec<BTCBlockHash> {
    tx_list.reverse(); // reverse order, from latest to oldest
    let mut block_hashes: Vec<BTCBlockHash> = vec![];
    for tx in tx_list {
        let tx_order = tx.sequence_info.tx_order;
        let previous_tx_order = tx_order - 1; // if block_hash mismatched, rollback to previous tx_order
        let tx_data = tx.data;
        if let LedgerTxData::L1Block(l1_block) = tx_data {
            let block_hash = bitcoin::block::BlockHash::from_slice(&l1_block.block_hash).unwrap();
            let block_hash_str = block_hash.to_string();
            block_hashes.push(BTCBlockHash {
                block_height: l1_block.block_height,
                block_hash: block_hash_str,
                previous_tx_order,
            });
        }
    }
    block_hashes
}

async fn get_block_hash_from_da_rpc(
    da_url: &str,
    last_block_number: u128,
    search_depth: u128,
) -> anyhow::Result<Vec<BTCBlockHash>> {
    let mut block_hash_with_depth: Vec<BTCBlockHash> = vec![];
    let mut block_height_set = HashSet::new();
    for i in 0..=search_depth {
        let block_number = last_block_number - i;
        let tx_list = get_tx_list_from_chunk(da_url, block_number).await?;
        let block_hash_in_chunk = filter_l1block(tx_list);
        for block_hash in block_hash_in_chunk {
            if block_height_set.contains(&block_hash.block_height) {
                continue; // Reverse order, skip duplicate block_height. Higher tx_order, newer block
            }
            block_height_set.insert(block_hash.block_height);
            block_hash_with_depth.push(block_hash);
        }
    }
    Ok(block_hash_with_depth)
}

async fn get_tx_list_from_chunk(
    da_url: &str,
    chunk_id: u128,
) -> anyhow::Result<Vec<LedgerTransaction>> {
    let mut segments = Vec::new();
    let mut segment_id: u64 = 0;
    loop {
        let segment = get_segment(da_url, chunk_id, segment_id).await?;
        let is_last = segment.is_last();
        segments.push(segment);
        if is_last {
            break;
        }
        segment_id += 1;
    }
    let chunk = chunk_from_segments(segments)?;
    let batch = chunk.get_batches().into_iter().next().unwrap();
    batch.verify(true)?;
    Ok(batch.get_tx_list())
}

async fn get_segment(
    url: &str,
    chunk_id: u128,
    segment_id: u64,
) -> anyhow::Result<Box<dyn Segment>> {
    let segment_url = format!("{}/{}_{}", url, chunk_id, segment_id);
    let res = reqwest::get(segment_url).await?;
    segment_from_bytes(&res.bytes().await?)
}

async fn get_block_hash_from_btc_rpc(block_height: u64, main: bool) -> anyhow::Result<String> {
    let url = if main {
        "https://blockstream.info/api/block-height/"
    } else {
        "https://blockstream.info/testnet/api/block-height/"
    };

    let url = format!("{}{}", url, block_height);
    let block_hash = reqwest::get(url).await.unwrap().text().await?;
    Ok(block_hash)
}
