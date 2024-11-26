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