// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_config::state_prune::StatePruneConfig;
use serde::{Deserialize, Serialize};

/// Deduplication strategy for node processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeduplicationStrategy {
    /// Use memory-based HashSet (fastest, but OOM risk)
    Memory,
    /// Use BloomFilter for initial filtering
    BloomFilter,
    /// Use RocksDB-based persistent deduplication (most scalable)
    RocksDB,
    /// Hybrid approach: BloomFilter + RocksDB
    Hybrid,
}

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

    /// Deduplication strategy to use
    pub deduplication_strategy: DeduplicationStrategy,

    /// Enable bloom filter for node deduplication (legacy, replaced by deduplication_strategy)
    pub enable_bloom_filter: bool,

    /// Bloom filter expected false positive rate
    pub bloom_filter_fp_rate: f64,

    /// Batch size for deduplication checks (0 = same as processing batch size)
    pub deduplication_batch_size: usize,

    /// Enable adaptive batch sizing based on memory pressure
    pub enable_adaptive_batching: bool,

    /// Memory pressure threshold (percentage) to trigger batch size reduction
    pub memory_pressure_threshold: f64,
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
            deduplication_strategy: DeduplicationStrategy::RocksDB,
            enable_bloom_filter: false, // Disabled in favor of RocksDB strategy
            bloom_filter_fp_rate: 0.001,
            deduplication_batch_size: 0, // Use same as processing batch size
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
            workers: config.parallel_workers,
            memory_limit: config.memory_limit,
            enable_progress_tracking: config.snapshot.enable_progress_tracking,
            progress_interval_seconds: config.snapshot.progress_interval_seconds,
            enable_resume: config.snapshot.enable_resume,
            max_traversal_time_hours: config.snapshot.max_traversal_time_hours,
            deduplication_strategy: DeduplicationStrategy::RocksDB,
            enable_bloom_filter: false, // Default to RocksDB strategy
            bloom_filter_fp_rate: 0.001,
            deduplication_batch_size: 0, // Use same as processing batch size
            enable_adaptive_batching: true,
            memory_pressure_threshold: 0.8,
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

        if !(0.0..1.0).contains(&self.memory_pressure_threshold) {
            return Err(anyhow::anyhow!(
                "Memory pressure threshold must be between 0 and 1"
            ));
        }

        // Validate strategy-specific settings
        match self.deduplication_strategy {
            DeduplicationStrategy::Memory => {
                tracing::warn!("Memory deduplication strategy selected - beware of OOM risk with large datasets");
            }
            DeduplicationStrategy::BloomFilter | DeduplicationStrategy::Hybrid => {
                if self.enable_bloom_filter && !(0.0..1.0).contains(&self.bloom_filter_fp_rate) {
                    return Err(anyhow::anyhow!(
                        "Bloom filter false positive rate must be between 0 and 1 when enabled"
                    ));
                }
            }
            DeduplicationStrategy::RocksDB => {
                // RocksDB strategy validation
                tracing::info!("RocksDB deduplication strategy selected - recommended for large datasets");
            }
        }

        Ok(())
    }

    /// Get effective deduplication batch size
    pub fn get_deduplication_batch_size(&self) -> usize {
        if self.deduplication_batch_size > 0 {
            self.deduplication_batch_size
        } else {
            self.batch_size
        }
    }

    /// Check if adaptive batching should be enabled based on strategy
    pub fn should_use_adaptive_batching(&self) -> bool {
        self.enable_adaptive_batching
            && matches!(self.deduplication_strategy, DeduplicationStrategy::RocksDB | DeduplicationStrategy::Hybrid)
    }
}
