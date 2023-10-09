// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::u256::U256;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::jsonrpc_types::StrView;

use super::ethereum_types::block::BlockNumber;

/// Account information.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EthFeeHistory {
    pub oldest_block: BlockNumber,
    pub base_fee_per_gas: Vec<StrView<U256>>,
    pub gas_used_ratio: Vec<f64>,
    pub reward: Option<Vec<Vec<StrView<U256>>>>,
}
