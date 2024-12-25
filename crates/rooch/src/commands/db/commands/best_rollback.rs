// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use bitcoin::hashes::Hash;
use clap::Parser;
use rooch_types::da::chunk::chunk_from_segments;
use rooch_types::da::segment::{segment_from_bytes, Segment};
use rooch_types::error::RoochResult;
use rooch_types::transaction::{LedgerTransaction, LedgerTxData};
use std::cmp::min;
use std::collections::HashSet;

/// try to find the best tx_order for rollback caused by chain reorg
#[derive(Debug, Parser)]
pub struct BestRollbackCommand {
    #[clap(long = "da-url")]
    pub da_url: String,

    #[clap(
        long = "last-block-number",
        help = "last avail block number in DA, got by `rooch rpc request --method rooch_status`"
    )]
    pub last_l2_block_number: u128,
    #[clap(
        long = "search-depth",
        help = "max blocks search depth for rollback, default is 16. If we cannot find best rollback tx_order in search depth, increase this value"
    )]
    pub search_depth: Option<u64>,
    #[clap(long = "main", help = "Bitcoin Mainnet or not. default is false")]
    pub main: bool,
}

impl BestRollbackCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let depth = self.search_depth.unwrap_or(16);
        let mut da_hashes =
            get_block_hash_from_da_rpc(&self.da_url, self.last_l2_block_number, depth as u128)
                .await?;
        da_hashes.sort_by(|a, b| b.0.cmp(&a.0)); // order by block_number desc
        let max_block_number = da_hashes.first();
        if max_block_number.is_none() {
            println!("no btc block found in DA, please increase search depth");
            return Ok(());
        }
        // compare block hash from DA and BTC, find the best rollback tx_order
        let mut best_tx_order = 0;
        let mut found_matched_block_hash = false;
        for (da_block_number, da_block_hash, previous_tx_order) in da_hashes.iter() {
            let btc_block_hash = get_block_hash_from_btc_rpc(*da_block_number, self.main).await?;
            if btc_block_hash != *da_block_hash {
                best_tx_order = min(best_tx_order, *previous_tx_order);
            } else {
                found_matched_block_hash = true;
                break;
            }
        }
        if !found_matched_block_hash {
            println!("no matched block hash found in BTC, please increase search depth");
        } else {
            println!("best rollback tx_order is {}", best_tx_order);
        }

        Ok(())
    }
}

fn filter_l1block(mut tx_list: Vec<LedgerTransaction>) -> Vec<(u64, String, u64)> {
    tx_list.reverse();
    let mut block_hashes = vec![];
    for tx in tx_list {
        let tx_order = tx.sequence_info.tx_order;
        let previous_tx_order = tx_order - 1; // if block_hash mismatched, rollback to previous tx_order
        let tx_data = tx.data;
        if let LedgerTxData::L1Block(l1_block) = tx_data {
            let block_hash = bitcoin::block::BlockHash::from_slice(&l1_block.block_hash).unwrap();
            let block_hash_str = block_hash.to_string();
            block_hashes.push((l1_block.block_height, block_hash_str, previous_tx_order));
        }
    }
    block_hashes
}

async fn get_block_hash_from_da_rpc(
    da_url: &str,
    last_block_number: u128,
    search_depth: u128,
) -> anyhow::Result<Vec<(u64, String, u64)>> {
    let mut block_hash = vec![];
    let mut block_height_set = HashSet::new();
    for i in 0..=search_depth {
        let block_number = last_block_number - i;
        let tx_list = get_tx_list_from_chunk(da_url, block_number).await?;
        let block_hash_in_chunk = filter_l1block(tx_list);
        for (block_height, block_hash_str, previous_tx_order) in block_hash_in_chunk {
            if block_height_set.contains(&block_height) {
                continue; // Reverse order, skip duplicate block_height. Higher tx_order, newer block
            }
            block_height_set.insert(block_height);
            block_hash.push((block_height, block_hash_str, previous_tx_order));
        }
    }
    Ok(block_hash)
}

async fn get_tx_list_from_chunk(
    da_url: &str,
    chunk_id: u128,
) -> anyhow::Result<Vec<LedgerTransaction>> {
    let mut segments = Vec::new();
    let segment_id: u64 = 0;
    loop {
        let segment = get_segment(da_url, chunk_id, segment_id).await?;
        let is_last = segment.is_last();
        segments.push(segment);
        if is_last {
            break;
        }
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
