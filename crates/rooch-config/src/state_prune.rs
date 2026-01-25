// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use moveos_types::h256::H256;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for State Prune operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatePruneConfig {
    /// Working directory for temporary files and intermediate results
    pub work_dir: PathBuf,

    /// Batch size for processing nodes during snapshot creation
    pub batch_size: usize,

    /// Memory limit in bytes for operations (0 = no limit)
    pub memory_limit: u64,

    /// Number of parallel workers
    pub parallel_workers: usize,

    /// Whether to skip confirmation prompts
    pub skip_confirm: bool,

    /// Whether to keep backup after successful operation
    pub keep_backup: bool,

    /// Configuration for snapshot creation
    pub snapshot: SnapshotConfig,

    /// Configuration for replay operations
    pub replay: ReplayConfig,
}

/// Configuration for snapshot creation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotConfig {
    /// Default output directory for snapshots
    pub default_output_dir: PathBuf,

    /// Maximum number of nodes to process in one batch
    pub max_batch_size: usize,

    /// Progress reporting interval in seconds
    pub progress_interval_seconds: u64,

    /// Enable resume from interrupted operations
    pub enable_resume: bool,
}

/// Configuration for replay operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayConfig {
    /// Default batch size for changeset processing
    pub default_batch_size: usize,

    /// Enable verification of final state root
    pub verify_final_state_root: bool,

    /// Enable intermediate checkpoints during replay
    pub enable_checkpoints: bool,

    /// Checkpoint interval in number of changesets
    pub checkpoint_interval: usize,

    /// Maximum retry attempts for failed operations
    pub max_retry_attempts: usize,

    /// Whether to validate state after each batch
    pub validate_after_batch: bool,

    /// History pruning configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history_prune: Option<HistoryPruneConfig>,
}

/// History pruning configuration for replay operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryPruneConfig {
    /// Enable history pruning during replay
    pub enabled: bool,

    /// Keep data from this tx_order (inclusive)
    pub retain_from: u64,

    /// Alternative: retain window size (calculate retain_from as to_order + 1 - window)
    pub retain_window: Option<u64>,

    /// Comma-separated list of column families to prune
    pub prune_cfs: Vec<String>,

    /// Dry-run mode: only report would-be deletions without writing
    pub dry_run: bool,
}

/// Snapshot metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMeta {
    /// Transaction order when snapshot was created
    pub tx_order: u64,

    /// State root hash at the time of snapshot
    pub state_root: H256,

    /// Number of global objects
    pub global_size: u64,

    /// Number of SMT nodes in snapshot
    pub node_count: u64,

    /// Snapshot creation timestamp
    pub created_at: u64,

    /// Snapshot version
    pub version: u32,

    /// Additional metadata
    pub metadata: serde_json::Value,
}

/// Replay operation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayReport {
    /// Number of changesets processed
    pub changesets_processed: u64,

    /// Number of nodes updated
    pub nodes_updated: u64,

    /// Final state root
    pub final_state_root: H256,

    /// Whether verification passed
    pub verification_passed: bool,

    /// Time taken for replay (in seconds)
    pub duration_seconds: u64,

    /// Error messages (if any)
    pub errors: Vec<String>,

    /// Additional statistics
    pub statistics: ReplayStatistics,

    /// History pruning report (if enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history_prune_report: Option<HistoryPruneReport>,
}

/// History pruning operation report
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HistoryPruneReport {
    /// Whether history pruning was enabled
    pub enabled: bool,

    /// Retain from tx_order (inclusive)
    pub retain_from: u64,

    /// Column families that were pruned
    pub cfs_pruned: Vec<String>,

    /// Total records deleted
    pub records_deleted: u64,

    /// Estimated bytes freed
    pub bytes_estimated: u64,

    /// Dry-run mode
    pub dry_run: bool,

    /// Per-CF statistics
    pub cf_stats: Vec<HistoryPruneCFStats>,
}

/// Per-column-family pruning statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryPruneCFStats {
    /// Column family name
    pub cf_name: String,

    /// Number of records deleted
    pub records_deleted: u64,

    /// Estimated bytes freed
    pub bytes_estimated: u64,
}

/// Replay operation statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReplayStatistics {
    /// Number of objects created
    pub objects_created: u64,

    /// Number of objects updated
    pub objects_updated: u64,

    /// Number of objects deleted
    pub objects_deleted: u64,

    /// Total data size processed (in bytes)
    pub data_size_bytes: u64,

    /// Memory usage peak (in bytes)
    pub peak_memory_bytes: u64,
}

impl Default for StatePruneConfig {
    fn default() -> Self {
        Self {
            work_dir: PathBuf::from("/tmp/rooch-state-prune"),
            batch_size: 10000,
            memory_limit: 16 * 1024 * 1024 * 1024, // 16GB
            parallel_workers: 4,
            skip_confirm: false,
            keep_backup: true,
            snapshot: SnapshotConfig::default(),
            replay: ReplayConfig::default(),
        }
    }
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            default_output_dir: PathBuf::from("./snapshots"),
            max_batch_size: 50000,
            progress_interval_seconds: 30,
            enable_resume: true,
        }
    }
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            default_batch_size: 1000,
            verify_final_state_root: true,
            enable_checkpoints: true,
            checkpoint_interval: 10000,
            max_retry_attempts: 3,
            validate_after_batch: true,
            history_prune: None,
        }
    }
}

impl StatePruneConfig {
    /// Create a new configuration with custom settings
    pub fn new(work_dir: PathBuf) -> Self {
        Self {
            work_dir,
            ..Default::default()
        }
    }

    /// Validate configuration settings
    pub fn validate(&self) -> Result<()> {
        // Validate work directory
        if self.work_dir.as_os_str().is_empty() {
            return Err(anyhow::anyhow!("Work directory cannot be empty"));
        }

        // Validate batch sizes
        if self.batch_size == 0 {
            return Err(anyhow::anyhow!("Batch size must be greater than 0"));
        }

        if self.snapshot.max_batch_size == 0 {
            return Err(anyhow::anyhow!(
                "Snapshot max batch size must be greater than 0"
            ));
        }

        if self.replay.default_batch_size == 0 {
            return Err(anyhow::anyhow!(
                "Replay default batch size must be greater than 0"
            ));
        }

        // Validate worker count
        if self.parallel_workers == 0 {
            return Err(anyhow::anyhow!("Parallel workers must be greater than 0"));
        }

        // Validate memory limit
        if self.memory_limit > 0 && self.memory_limit < 1024 * 1024 * 1024 {
            // 1GB minimum
            return Err(anyhow::anyhow!(
                "Memory limit must be at least 1GB or 0 for unlimited"
            ));
        }

        Ok(())
    }

    /// Load configuration from a file
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: StatePruneConfig = serde_json::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Save configuration to a file
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        self.validate()?;
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get work directory path
    pub fn work_dir(&self) -> &PathBuf {
        &self.work_dir
    }

    /// Ensure work directory exists
    pub fn ensure_work_dir(&self) -> Result<()> {
        std::fs::create_dir_all(&self.work_dir)?;
        Ok(())
    }
}

impl SnapshotMeta {
    /// Create new snapshot metadata
    pub fn new(tx_order: u64, state_root: H256, global_size: u64, node_count: u64) -> Self {
        Self {
            tx_order,
            state_root,
            global_size,
            node_count,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            version: 1,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    /// Validate snapshot metadata
    pub fn validate(&self) -> Result<()> {
        // tx_order = 0 is allowed for snapshots (represents "unspecified" or "latest")
        // For other contexts where tx_order must be > 0, additional validation should be performed

        if self.state_root == H256::zero() {
            return Err(anyhow::anyhow!("State root cannot be zero"));
        }

        if self.created_at == 0 {
            return Err(anyhow::anyhow!("Creation timestamp cannot be 0"));
        }

        Ok(())
    }

    /// Get snapshot file name
    pub fn file_name(&self) -> String {
        format!("snapshot_{}_{}.json", self.tx_order, self.state_root)
    }

    /// Save metadata to file
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, dir: P) -> Result<PathBuf> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;

        let file_path = dir.join(self.file_name());
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&file_path, content)?;
        Ok(file_path)
    }

    /// Load metadata from file
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let meta: SnapshotMeta = serde_json::from_str(&content)?;
        meta.validate()?;
        Ok(meta)
    }
}

impl ReplayReport {
    /// Create new replay report
    pub fn new() -> Self {
        Self {
            changesets_processed: 0,
            nodes_updated: 0,
            final_state_root: H256::zero(),
            verification_passed: false,
            duration_seconds: 0,
            errors: Vec::new(),
            statistics: ReplayStatistics::default(),
            history_prune_report: None,
        }
    }

    /// Add error to report
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    /// Check if replay was successful
    pub fn is_success(&self) -> bool {
        self.errors.is_empty() && self.verification_passed
    }

    /// Save report to file
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

impl Default for ReplayReport {
    fn default() -> Self {
        Self::new()
    }
}
