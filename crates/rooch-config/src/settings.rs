// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// 15 minutes > Bitcoin block interval, Rooch block must be made in this interval
// at least has Bitcoin block transaction
pub const ROOCH_BATCH_INTERVAL: u64 = 1000 * 60 * 15;
// 5 seconds, check avail block to propose interval
pub const PROPOSER_CHECK_INTERVAL: u64 = 5;
