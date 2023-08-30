// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::jsonrpc_types::{H160View, U256View, U64View};

/// A validator withdrawal from the consensus layer.
/// See EIP-4895: Beacon chain push withdrawals as operations.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Withdrawal {
    /// Monotonically increasing identifier issued by consensus layer
    pub index: U64View,

    /// Index of validator associated with withdrawal
    #[serde(rename = "validatorIndex")]
    pub validator_index: U64View,

    /// Target address for withdrawn ether
    pub address: H160View,

    /// Value of withdrawal (in wei)
    pub amount: U256View,
}
