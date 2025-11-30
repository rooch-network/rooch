// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::config::Config;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Garbage collection configuration (stop-the-world GC only)
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Parser)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub struct PruneConfig {
    /// Number of nodes to scan per batch (used by both GC and sweep operations).
    #[clap(long = "pruner-scan-batch", default_value_t = 10000)]
    pub scan_batch: usize,

    /// Number of deletion operations per RocksDB write batch.
    #[clap(long = "pruner-delete-batch", default_value_t = 5000)]
    pub delete_batch: usize,

    /// Bloom filter size in bits (must be power of two for fast modulo).
    #[clap(long = "pruner-bloom-bits", default_value_t = 8589934592)] // 2^33
    pub bloom_bits: usize,

    /// Create and use optional cf_reach_seen column family for cold hash spill.
    #[clap(long = "pruner-enable-reach-seen-cf", default_value_t = false)]
    pub enable_reach_seen_cf: bool,

    /// Number of recent tx_orders to protect from GC (default 30000).
    /// Set to 0 to allow aggressive GC (only protects the latest root - for testing only).
    #[clap(long = "pruner-protection-orders", default_value_t = 30000)]
    pub protection_orders: u64,

    /// Enable GC recycle bin for debugging/recovery (default: false).
    #[clap(long = "pruner-recycle-bin-enable", default_value_t = false)]
    pub recycle_bin_enable: bool,

    /// Maximum number of entries in recycle bin (default: 10000).
    #[clap(long = "pruner-recycle-bin-max-entries", default_value_t = 10000)]
    pub recycle_bin_max_entries: usize,

    /// Maximum total bytes in recycle bin (default: 100MB).
    #[clap(long = "pruner-recycle-bin-max-bytes", default_value_t = 100_000_000)]
    pub recycle_bin_max_bytes: usize,

    /// Number of recent state roots to protect from GC (default: 1 for backward compatibility).
    /// Set to higher values to protect more historical state roots.
    #[clap(long = "pruner-protected-roots-count", default_value_t = 1)]
    pub protected_roots_count: usize,

    /// PersistentMarker batch size for efficient bulk writes (default: 10000).
    /// Larger batches improve write performance but use more memory.
    #[clap(long = "pruner-marker-batch-size", default_value_t = 10000)]
    pub marker_batch_size: usize,

    /// PersistentMarker bloom filter bits (must be power of two, default: 2^20 = 1MB).
    /// Larger bloom filters reduce false positive rates but use more memory.
    #[clap(long = "pruner-marker-bloom-bits", default_value_t = 1048576)] // 2^20
    pub marker_bloom_bits: usize,

    /// PersistentMarker bloom filter hash functions (default: 4).
    /// More hash functions reduce false positive rates but increase CPU usage.
    #[clap(long = "pruner-marker-bloom-hash-fns", default_value_t = 4)]
    pub marker_bloom_hash_fns: u8,

    /// Memory threshold for selecting PersistentMarker vs InMemoryMarker (MB, default: 1024).
    /// If estimated memory usage exceeds this threshold, PersistentMarker will be selected.
    #[clap(long = "pruner-marker-memory-threshold-mb", default_value_t = 1024)]
    pub marker_memory_threshold_mb: usize,

    /// Enable automatic strategy selection based on dataset size and available memory (default: true).
    /// When enabled, automatically chooses between InMemory and Persistent markers.
    #[clap(long = "pruner-marker-auto-strategy", default_value_t = true)]
    pub marker_auto_strategy: bool,

    /// Force PersistentMarker usage regardless of dataset size (default: false).
    /// When enabled, always uses PersistentMarker even for small datasets.
    #[clap(long = "pruner-marker-force-persistent", default_value_t = false)]
    pub marker_force_persistent: bool,

    /// Temporary column family name for PersistentMarker (default: "gc_marker_temp").
    /// Used for creating the temporary RocksDB column family for marker data.
    #[clap(long = "pruner-marker-temp-cf-name", default_value = "gc_marker_temp")]
    pub marker_temp_cf_name: String,

    /// Enable enhanced error recovery for PersistentMarker (default: true).
    /// When enabled, provides more robust error handling and retry logic.
    #[clap(long = "pruner-marker-error-recovery", default_value_t = true)]
    pub marker_error_recovery: bool,
}

impl Default for PruneConfig {
    fn default() -> Self {
        Self {
            scan_batch: 10000,
            delete_batch: 5000,
            bloom_bits: 8589934592, // 2^33
            enable_reach_seen_cf: false,
            protection_orders: 30000,
            recycle_bin_enable: false,
            recycle_bin_max_entries: 10000,
            recycle_bin_max_bytes: 100_000_000, // 100MB
            protected_roots_count: 1,           // Default to backward compatibility
            // GC-specific marker configuration defaults
            marker_batch_size: 10000,
            marker_bloom_bits: 1048576, // 2^20 = 1MB
            marker_bloom_hash_fns: 4,
            marker_memory_threshold_mb: 1024, // 1GB
            marker_auto_strategy: true,
            marker_force_persistent: false,
            marker_temp_cf_name: "gc_marker_temp".to_string(),
            marker_error_recovery: true,
        }
    }
}

impl Config for PruneConfig {}

impl std::fmt::Display for PruneConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).map_err(|_| std::fmt::Error)?
        )
    }
}

impl FromStr for PruneConfig {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let deserialized: PruneConfig = serde_json::from_str(s)?;
        Ok(deserialized)
    }
}
