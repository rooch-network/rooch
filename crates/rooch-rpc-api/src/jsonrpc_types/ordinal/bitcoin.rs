// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::jsonrpc_types::StrView;

use super::{hash_types::BlockHashView, network::NetworkView, rpc::SoftforkView};

/// Models the result of "getblockchaininfo"
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub struct GetBlockchainInfoResult {
    /// Current network name as defined in BIP70 (main, test, signet, regtest)
    pub chain: NetworkView,
    /// The current number of blocks processed in the server
    pub blocks: StrView<u64>,
    /// The current number of headers we have validated
    pub headers: StrView<u64>,
    /// The hash of the currently best block
    pub best_block_hash: BlockHashView,
    /// The current difficulty
    pub difficulty: StrView<f64>,
    /// Median time for the current best block
    pub median_time: StrView<u64>,
    /// Estimate of verification progress [0..1]
    pub verification_progress: StrView<f64>,
    /// Estimate of whether this node is in Initial Block Download mode
    pub initial_block_download: bool,
    /// Total amount of work in active chain, in hexadecimal
    pub chain_work: StrView<Vec<u8>>,
    /// The estimated size of the block and undo files on disk
    pub size_on_disk: StrView<u64>,
    /// If the blocks are subject to pruning
    pub pruned: bool,
    /// Lowest-height complete block stored (only present if pruning is enabled)
    pub prune_height: Option<StrView<u64>>,
    /// Whether automatic pruning is enabled (only present if pruning is enabled)
    pub automatic_pruning: Option<bool>,
    /// The target size used by pruning (only present if automatic pruning is enabled)
    pub prune_target_size: Option<StrView<u64>>,
    /// Status of softforks in progress
    pub softforks: HashMap<String, SoftforkView>,
    /// Any network and blockchain warnings.
    pub warnings: String,
}
