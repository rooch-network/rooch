// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use primitive_types::H256;
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

/// Snapshot persisted at the end of BuildReach so that SweepExpired
/// can operate on an identical view of the chain state.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PruneSnapshot {
    /// The global state root recorded from StartupInfo during BuildReach.
    pub state_root: H256,
    /// The latest sequencer order at the same moment; used to define cutoff.
    pub latest_order: u64,
}
