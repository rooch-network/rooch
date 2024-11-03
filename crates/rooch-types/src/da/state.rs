// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
/// DA server basic status
pub struct DAServerStatus {
    /// The last block number
    pub last_block_number: Option<u128>,
    /// The last tx order in the last block
    pub last_tx_order: Option<u64>,
    /// The last block number updated time(Unix timestamp in seconds)
    /// Should be closed to request time if there were new blocks.
    /// None if no blocks were received after server start.
    pub last_block_update_time: Option<u64>,
    /// The last available block number, may little behind the last block number.
    /// [0, last_avail_block_number] blocks were confirmed by DA backend.
    /// If meet error in background submitter job, it may be far behind the last block number.
    pub last_avail_block_number: Option<u128>,
    /// The last available tx order
    pub last_avail_tx_order: Option<u64>,
    /// The last available block number updated time(Unix timestamp)
    /// If both of last_avail_block_number and last_avail_tx_order are not updated for a long time,
    /// it may indicate that the background submitter job is not working:
    /// 1. DA backends collapse
    /// 2. RoochStore is not consistent (cannot get tx from DB by tx order)
    pub last_avail_block_update_time: Option<u64>,
    /// The available backends names
    pub avail_backends: Vec<String>,
}
