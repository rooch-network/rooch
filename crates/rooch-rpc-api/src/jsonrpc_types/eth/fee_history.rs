// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::ethereum_types::block::BlockNumber;
use crate::jsonrpc_types::StrView;
use ethers::types::U256;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Account information.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EthFeeHistory {
    pub oldest_block: StrView<BlockNumber>,
    pub base_fee_per_gas: Vec<StrView<U256>>,
    pub gas_used_ratio: Vec<f64>,
    pub reward: Option<Vec<Vec<StrView<U256>>>>,
}
