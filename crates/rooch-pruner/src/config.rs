// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

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

    // === Marker Configuration ===
    /// Bloom filter size in bits (must be power of two, optional - uses dynamic calculation if 0)
    pub marker_bloom_bits: usize,

    /// Bloom filter hash functions (default: 4)
    pub marker_bloom_hash_fns: u8,

    /// Target false positive rate for Bloom filter (default: 0.01 = 1%)
    pub marker_target_fp_rate: f64,
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
            marker_bloom_bits: 0, // 0 = use dynamic calculation
            marker_bloom_hash_fns: 4,
            marker_target_fp_rate: 0.01, // 1% false positive rate
        }
    }
}

impl GCConfig {
    pub fn new() -> Self {
        Self::default()
    }
}
