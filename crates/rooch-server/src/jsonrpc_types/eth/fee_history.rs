// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use serde::{Serialize, Deserialize};
use ethers::types::{U256, BlockNumber};

/// Account information.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthFeeHistory {
    pub oldest_block: BlockNumber,
    pub base_fee_per_gas: Vec<U256>,
    pub gas_used_ratio: Vec<f64>,
    pub reward: Option<Vec<Vec<U256>>>,
}