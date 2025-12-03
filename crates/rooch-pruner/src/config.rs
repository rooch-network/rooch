// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use super::marker::MarkerStrategy;
use super::recycle_bin::RecycleBinConfig;

/// Unified Garbage Collector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// Recycle bin configuration with strong backup defaults
    #[serde(default)]
    pub recycle_bin: RecycleBinConfig,

    /// Whether to trigger RocksDB compaction after GC
    pub force_compaction: bool,

    /// Skip user confirmation prompts (use with caution)
    pub skip_confirm: bool,

    // === Core GC Configuration ===
    /// Number of nodes to scan per batch (used by both GC and sweep operations)
    pub scan_batch: usize,

    /// Number of deletion operations per RocksDB write batch
    pub batch_size: usize,

    /// Bloom filter size in bits (must be power of two for fast modulo)
    pub bloom_bits: usize,

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

impl Default for GCConfig {
    fn default() -> Self {
        Self {
            // === Runtime Configuration ===
            dry_run: false,
            workers: num_cpus::get(),
            use_recycle_bin: true,
            recycle_bin: RecycleBinConfig::default(),
            force_compaction: false,
            skip_confirm: false,

            // === Core GC Configuration ===
            scan_batch: 1000,
            batch_size: 10_000,
            bloom_bits: 1 << 20, // 1M bits
            protected_roots_count: 1,

            // === Marker Configuration ===
            marker_strategy: crate::marker::MarkerStrategy::Auto,
            marker_batch_size: 10000,
            marker_bloom_bits: 1 << 20, // 1M bits
            marker_bloom_hash_fns: 4,
            marker_memory_threshold_mb: 2048, // 2GB
            marker_auto_strategy: true,
            marker_force_persistent: false,
            marker_temp_cf_name: "gc_marker_temp".to_string(),
            marker_error_recovery: true,
        }
    }
}

impl GCConfig {
    pub fn new() -> Self {
        Self::default()
    }
}
