// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use super::marker::MarkerStrategy;

/// Unified Garbage Collector configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub struct GCConfig {
    // === Runtime Configuration ===
    /// Whether to run in dry-run mode (scan and report without deleting)
    pub dry_run: bool,

    /// Number of worker threads for parallel processing
    pub workers: usize,

    /// Whether to use recycle bin for deleted nodes
    pub use_recycle_bin: bool,

    /// Whether to trigger RocksDB compaction after GC
    pub force_compaction: bool,

    /// Force execution without safety confirmations
    pub force_execution: bool,

    // === Core GC Configuration ===
    /// Number of nodes to scan per batch (used by both GC and sweep operations)
    pub scan_batch: usize,

    /// Number of deletion operations per RocksDB write batch
    pub batch_size: usize,

    /// Bloom filter size in bits (must be power of two for fast modulo)
    pub bloom_bits: usize,

    /// Number of recent tx_orders to protect from GC (default 30000)
    /// Set to 0 to allow aggressive GC (only protects the latest root - for testing only)
    pub protection_orders: u64,

    /// Number of recent state roots to protect from GC (default: 1 for backward compatibility)
    pub protected_roots_count: usize,

    // === Marker Strategy Configuration ===
    /// Marker strategy selection (auto/memory/persistent)
    pub marker_strategy: MarkerStrategy,

    /// PersistentMarker batch size for efficient bulk writes
    pub marker_batch_size: usize,

    /// PersistentMarker bloom filter bits (must be power of two, default: 2^20 = 1MB)
    pub marker_bloom_bits: usize,

    /// PersistentMarker bloom filter hash functions (default: 4)
    pub marker_bloom_hash_fns: u8,

    /// Memory threshold for selecting PersistentMarker vs InMemoryMarker (MB)
    pub marker_memory_threshold_mb: usize,

    /// Enable automatic strategy selection based on dataset size
    pub marker_auto_strategy: bool,

    /// Force PersistentMarker usage regardless of dataset size
    pub marker_force_persistent: bool,

    /// Temporary column family name for PersistentMarker
    pub marker_temp_cf_name: String,

    /// Enable enhanced error recovery for PersistentMarker
    pub marker_error_recovery: bool,
}

impl GCConfig {
    pub fn new() -> Self {
        Self::default()
    }
}
