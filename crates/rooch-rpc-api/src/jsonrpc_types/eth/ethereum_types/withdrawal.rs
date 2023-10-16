// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{H160View, StrView};
use ethers::types::{U256, U64};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A validator withdrawal from the consensus layer.
/// See EIP-4895: Beacon chain push withdrawals as operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Withdrawal {
    /// Monotonically increasing identifier issued by consensus layer
    pub index: StrView<U64>,

    /// Index of validator associated with withdrawal
    #[serde(rename = "validatorIndex")]
    pub validator_index: StrView<U64>,

    /// Target address for withdrawn ether
    pub address: H160View,

    /// Value of withdrawal (in wei)
    pub amount: StrView<U256>,
}
