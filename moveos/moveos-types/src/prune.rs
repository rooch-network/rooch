// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

// pub const META_KEY_PHASE: &str = "phase";
// pub const META_KEY_CURSOR: &str = "cursor"; // placeholder for future use
// pub const META_KEY_BLOOM: &str = "bloom_snapshot";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrunePhase {
    BuildReach,
    SweepExpired,
    Incremental,
}
