// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_config::state_prune::StatePruneConfig;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Configuration for snapshot builder operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotBuilderConfig {
    /// Batch size for processing nodes
    pub batch_size: usize,

    /// Memory limit in bytes (0 = no limit)
    pub memory_limit: u64,

    /// Skip deduplication lookups when writing nodes (faster, more writes)
    #[serde(default)]
    pub skip_dedup: bool,

    /// Skip final compact/cleanup for the snapshot DB (faster, larger on-disk)
    #[serde(default)]
    pub skip_final_compact: bool,

    /// Disable auto compactions while building snapshot to avoid background I/O
    #[serde(default)]
    pub disable_auto_compactions: bool,

    /// Progress reporting interval in seconds
    pub progress_interval_seconds: u64,

    /// Enable resume from interrupted operations
    pub enable_resume: bool,

    /// Enable adaptive batch sizing based on memory pressure
    pub enable_adaptive_batching: bool,

    /// Memory pressure threshold (ratio from 0.0 to 1.0) to trigger batch size reduction
    pub memory_pressure_threshold: f64,
}

impl Default for SnapshotBuilderConfig {
    fn default() -> Self {
        Self {
            batch_size: 10000,
            memory_limit: 16 * 1024 * 1024 * 1024, // 16GB
            skip_dedup: false,
            skip_final_compact: false,
            disable_auto_compactions: true,
            progress_interval_seconds: 30,
            enable_resume: true,
            enable_adaptive_batching: true,
            memory_pressure_threshold: 0.8, // 80% memory usage triggers reduction
        }
    }
}

impl SnapshotBuilderConfig {
    /// Create from StatePruneConfig
    pub fn from_state_prune_config(config: &StatePruneConfig) -> Self {
        Self {
            batch_size: config.batch_size,
            memory_limit: config.memory_limit,
            skip_dedup: false,
            skip_final_compact: false,
            disable_auto_compactions: true,
            progress_interval_seconds: config.snapshot.progress_interval_seconds,
            enable_resume: config.snapshot.enable_resume,
            enable_adaptive_batching: true,
            memory_pressure_threshold: 0.8,
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.batch_size == 0 {
            return Err(anyhow::anyhow!("Batch size must be greater than 0"));
        }

        if self.progress_interval_seconds == 0 {
            return Err(anyhow::anyhow!("Progress interval must be greater than 0"));
        }

        if !(0.0..=1.0).contains(&self.memory_pressure_threshold) {
            return Err(anyhow::anyhow!(
                "Memory pressure threshold must be between 0.0 and 1.0 inclusive"
            ));
        }

        info!("Using RocksDB deduplication - recommended for large datasets");

        Ok(())
    }

    /// Check if adaptive batching should be enabled
    pub fn should_use_adaptive_batching(&self) -> bool {
        self.enable_adaptive_batching
    }
}
