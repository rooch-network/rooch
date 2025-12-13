// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_config::state_prune::StatePruneConfig;
use serde::{Deserialize, Serialize};

/// Configuration for snapshot builder operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotBuilderConfig {
    /// Batch size for processing nodes
    pub batch_size: usize,

    /// Number of parallel workers
    pub workers: usize,

    /// Memory limit in bytes (0 = no limit)
    pub memory_limit: u64,

    /// Enable progress tracking
    pub enable_progress_tracking: bool,

    /// Progress reporting interval in seconds
    pub progress_interval_seconds: u64,

    /// Enable resume from interrupted operations
    pub enable_resume: bool,

    /// Maximum traversal time in hours
    pub max_traversal_time_hours: u64,

    /// Enable bloom filter for node deduplication
    pub enable_bloom_filter: bool,

    /// Bloom filter expected false positive rate
    pub bloom_filter_fp_rate: f64,
}

impl Default for SnapshotBuilderConfig {
    fn default() -> Self {
        Self {
            batch_size: 10000,
            workers: 4,
            memory_limit: 16 * 1024 * 1024 * 1024, // 16GB
            enable_progress_tracking: true,
            progress_interval_seconds: 30,
            enable_resume: true,
            max_traversal_time_hours: 24,
            enable_bloom_filter: true,
            bloom_filter_fp_rate: 0.001,
        }
    }
}

impl SnapshotBuilderConfig {
    /// Create from StatePruneConfig
    pub fn from_state_prune_config(config: &StatePruneConfig) -> Self {
        Self {
            batch_size: config.batch_size,
            workers: config.parallel_workers,
            memory_limit: config.memory_limit,
            enable_progress_tracking: config.snapshot.enable_progress_tracking,
            progress_interval_seconds: config.snapshot.progress_interval_seconds,
            enable_resume: config.snapshot.enable_resume,
            max_traversal_time_hours: config.snapshot.max_traversal_time_hours,
            enable_bloom_filter: true,
            bloom_filter_fp_rate: 0.001,
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.batch_size == 0 {
            return Err(anyhow::anyhow!("Batch size must be greater than 0"));
        }

        if self.workers == 0 {
            return Err(anyhow::anyhow!("Workers must be greater than 0"));
        }

        if self.progress_interval_seconds == 0 {
            return Err(anyhow::anyhow!("Progress interval must be greater than 0"));
        }

        if self.max_traversal_time_hours == 0 {
            return Err(anyhow::anyhow!("Max traversal time must be greater than 0"));
        }

        if !(0.0..1.0).contains(&self.bloom_filter_fp_rate) {
            return Err(anyhow::anyhow!(
                "Bloom filter false positive rate must be between 0 and 1"
            ));
        }

        Ok(())
    }
}
