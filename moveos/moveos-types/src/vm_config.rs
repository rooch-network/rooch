// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//TODO this config should be an on-chain config
pub struct VMConfig {
    pub max_gas_amount: u64,
}

impl VMConfig {
    pub const DEFAULT_MAX_GAS_AMOUNT: u64 = 100000000u64;
}
