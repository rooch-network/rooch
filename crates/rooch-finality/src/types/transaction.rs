// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionInfo {
    #[serde(rename = "txHash")]
    pub tx_hash: String,
    #[serde(rename = "blockHash")]
    pub block_hash: String,
    pub status: FinalityStatus,
    #[serde(rename = "blockTimestamp")]
    pub block_timestamp: u64,
    #[serde(rename = "blockHeight")]
    pub block_height: u64,
    #[serde(rename = "babylonFinalized")]
    pub babylon_finalized: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FinalityStatus {
    Pending,
    Unsafe,
    #[serde(rename = "btc finalized")]
    BitcoinFinalized,
    Safe,
    Finalized,
}
