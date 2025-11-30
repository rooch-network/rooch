// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod atomic_snapshot;
pub mod error_recovery;
pub mod garbage_collector;
pub mod historical_state;
pub mod incremental_sweep;
pub mod marker;
pub mod metrics;
pub mod pruner;
pub mod reachability;
pub mod recycle_bin;
pub mod safety_verifier;
pub mod sweep_expired;
#[cfg(test)]
mod tests;
pub mod util;
pub mod validation_tests;

// Re-export commonly used types
pub use garbage_collector::{GCConfig, GCReport, GarbageCollector, MarkStats, SweepStats};
pub use historical_state::{HistoricalStateCollector, HistoricalStateConfig};
pub use marker::{MarkerStrategy, NodeMarker};
