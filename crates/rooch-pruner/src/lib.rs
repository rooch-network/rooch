// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// GC-only modules after removing live pruner components
pub mod config; // Unified GC configuration
pub mod garbage_collector; // Core GC implementation
pub mod historical_state; // Historical state collection for multi-root GC protection
pub mod marker; // Node marking strategies for GC
pub mod reachability; // Reachability analysis for GC
pub mod recycle_bin; // Recycle bin for debugging/recovery
pub mod safety_verifier; // Safety verification for GC operations
pub mod state_prune; // State prune functionality
#[cfg(test)]
mod tests;

#[cfg(test)]
mod scalable_dedup_test;
pub mod util; // Utility functions for node traversal

// Re-export commonly used GC types
pub use config::GCConfig;
pub use garbage_collector::{GCReport, GarbageCollector, MarkStats, SweepStats};
pub use historical_state::{HistoricalStateCollector, HistoricalStateConfig};
pub use marker::{BloomFilterMarker, NodeMarker};
