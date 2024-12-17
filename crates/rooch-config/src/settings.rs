// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// 15 minutes > Bitcoin block interval, Rooch block must be made in this interval
// at least has Bitcoin block transaction
// large interval give enough time window to submit block to DA
// pub const ROOCH_BATCH_INTERVAL: u64 = 1000 * 60 * 15;
pub const ROOCH_BATCH_INTERVAL: u64 = 1000*60*1;
// 5 seconds, check avail block to propose interval
pub const PROPOSER_CHECK_INTERVAL: u64 = 5;
