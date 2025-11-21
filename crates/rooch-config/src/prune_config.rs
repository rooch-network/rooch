// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::config::Config;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Pruning-related configuration (v1+v2 hybrid)
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Parser)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub struct PruneConfig {
    /// Enable or disable background pruner.
    #[clap(long, default_value_t = false)]
    pub enable: bool,

    /// Whether boot cleanup (v1 DFS) already finished.
    /// Normally maintained automatically – manual override only for debugging.
    #[clap(long, default_value_t = false)]
    pub boot_cleanup_done: bool,

    /// Number of nodes to scan per DFS batch.
    #[clap(long, default_value_t = 10000)]
    pub scan_batch: usize,

    /// Number of deletion operations per RocksDB write batch.
    #[clap(long, default_value_t = 5000)]
    pub delete_batch: usize,

    /// Background tick interval (seconds).
    #[clap(long, default_value_t = 60)]
    pub interval_s: u64,

    /// Bloom filter size in bits (must be power of two for fast modulo).
    // #[clap(long, default_value_t = 4_194_304)]  // 2^22
    #[clap(long, default_value_t = 8589934592)] // 2^33
    pub bloom_bits: usize,

    /// Create and use optional cf_reach_seen column family for cold hash spill.
    #[clap(long, default_value_t = false)]
    pub enable_reach_seen_cf: bool,

    /// Window size in days for reachable roots (default 30).
    #[clap(long, default_value_t = 30)]
    pub window_days: u64,

    /// Enable incremental sweep phase for continuous cleanup (default true).
    #[clap(long, default_value_t = true)]
    pub enable_incremental_sweep: bool,

    /// Batch size for incremental sweep operations (default 1000).
    #[clap(long, default_value_t = 1000)]
    pub incremental_sweep_batch: usize,
}

impl Default for PruneConfig {
    fn default() -> Self {
        Self {
            enable: false,
            boot_cleanup_done: false,
            scan_batch: 10000,
            delete_batch: 5000,
            interval_s: 60,
            bloom_bits: 8589934592, // 2^33
            enable_reach_seen_cf: false,
            window_days: 30,
            enable_incremental_sweep: true,
            incremental_sweep_batch: 1000,
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
