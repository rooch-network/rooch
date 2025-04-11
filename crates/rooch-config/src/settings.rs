// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// 15 minutes > Bitcoin block interval, Rooch block must be made in this interval
// at least has Bitcoin block transaction
// large interval give enough time window to submit block to DA
pub const ROOCH_BATCH_INTERVAL: u64 = 1000 * 60 * 15;
// 5 seconds, check avail block to propose interval
pub const PROPOSER_CHECK_INTERVAL: u64 = 5;

/// weather enable multi coin store
pub const ENABLE_MULTI_COIN_STORE: bool = true;

/// Check if V2 system is enabled
pub fn is_multi_coin_store_enabled() -> bool {
    // Logic to check if multi coin store is enabled
    ENABLE_MULTI_COIN_STORE
}
