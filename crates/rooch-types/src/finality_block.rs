// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_types::h256::H256;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    #[serde(rename = "block_hash")]
    pub block_hash: String,
    #[serde(rename = "block_height")]
    pub block_height: u64,
    #[serde(rename = "block_timestamp")]
    pub block_timestamp: u64,
}

// Types and Structs
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlockID {
    pub number: u64,
    pub hash: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct L1BlockRef {
    pub hash: String,
    pub number: u64,
    pub time: u64,
    pub parent_hash: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct L2BlockRef {
    pub hash: H256,
    pub number: u64,
    pub time: u64,
    pub parent_hash: H256,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainSyncStatus {
    #[serde(rename = "latest_block")]
    pub latest_block_height: u64,
    #[serde(rename = "latest_btc_finalized_block")]
    pub latest_btc_finalized_block_height: u64,
    #[serde(rename = "earliest_btc_finalized_block")]
    pub earliest_btc_finalized_block_height: u64,
    #[serde(rename = "latest_eth_finalized_block")]
    pub latest_eth_finalized_block_height: u64,
}
