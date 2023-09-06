// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::lib::{U64, Address, U256};

/// A validator withdrawal from the consensus layer.
/// See EIP-4895: Beacon chain push withdrawals as operations.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Withdrawal {
    /// Monotonically increasing identifier issued by consensus layer
    pub index: U64,

    /// Index of validator associated with withdrawal
    #[serde(rename = "validatorIndex")]
    pub validator_index: U64,

    /// Target address for withdrawn ether
    pub address: Address,

    /// Value of withdrawal (in wei)
    pub amount: U256,
}
