// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_config::state_prune::{ReplayReport, SnapshotMeta};
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::PathBuf;

/// State prune operation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatePruneMetadata {
    /// Operation type
    pub operation_type: OperationType,

    /// Timestamp when operation started
    pub started_at: u64,

    /// Timestamp when operation completed (0 if not completed)
    pub completed_at: u64,

    /// Operation status
    pub status: OperationStatus,

    /// Error messages (if any)
    pub errors: Vec<String>,

    /// Configuration used for the operation
    pub config: serde_json::Value,

    /// Statistics
    pub statistics: OperationStatistics,
}

/// Types of state prune operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    /// Snapshot creation operation
    Snapshot {
        /// Target tx_order
        tx_order: u64,
        /// State root
        state_root: String,
        /// Output directory
        output_dir: PathBuf,
    },
    /// Replay operation
    Replay {
        /// Snapshot path
        snapshot_path: PathBuf,
        /// From tx_order
        from_order: u64,
        /// To tx_order
        to_order: u64,
        /// Output directory
        output_dir: PathBuf,
    },
}

/// Operation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationStatus {
    /// Operation is pending
    Pending,
    /// Operation is in progress
    InProgress {
        /// Progress percentage
        progress: f64,
        /// Current step
        current_step: String,
    },
    /// Operation completed successfully
    Completed,
    /// Operation failed
    Failed,
    /// Operation was cancelled
    Cancelled,
}

/// Operation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationStatistics {
    /// Total nodes processed
    pub nodes_processed: u64,

    /// Total bytes processed
    pub bytes_processed: u64,

    /// Peak memory usage in bytes
    pub peak_memory_bytes: u64,

    /// Total duration in seconds
    pub duration_seconds: u64,

    /// Additional metrics
    pub custom_metrics: serde_json::Value,
}

impl Default for OperationStatistics {
    fn default() -> Self {
        Self {
            nodes_processed: 0,
            bytes_processed: 0,
            peak_memory_bytes: 0,
            duration_seconds: 0,
            custom_metrics: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

impl StatePruneMetadata {
    /// Create new operation metadata
    pub fn new(operation_type: OperationType, config: serde_json::Value) -> Self {
        Self {
            operation_type,
            started_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            completed_at: 0,
            status: OperationStatus::Pending,
            errors: Vec::new(),
            config,
            statistics: OperationStatistics::default(),
        }
    }

    /// Mark operation as in progress
    pub fn mark_in_progress(&mut self, current_step: String, progress: f64) {
        self.status = OperationStatus::InProgress {
            progress,
            current_step,
        };
    }

    /// Mark operation as completed
    pub fn mark_completed(&mut self) {
        self.completed_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.status = OperationStatus::Completed;
    }

    /// Mark operation as failed
    pub fn mark_failed(&mut self, error: String) {
        self.errors.push(error);
        self.completed_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.status = OperationStatus::Failed;
    }

    /// Add error
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    /// Update statistics
    pub fn update_statistics<F>(&mut self, update_fn: F)
    where
        F: FnOnce(&mut OperationStatistics),
    {
        update_fn(&mut self.statistics);
    }

    /// Get operation duration in seconds
    pub fn duration_seconds(&self) -> u64 {
        let end_time = if self.completed_at > 0 {
            self.completed_at
        } else {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        };
        end_time.saturating_sub(self.started_at)
    }

    /// Check if operation is finished (completed, failed, or cancelled)
    pub fn is_finished(&self) -> bool {
        matches!(
            self.status,
            OperationStatus::Completed | OperationStatus::Failed | OperationStatus::Cancelled
        )
    }

    /// Save metadata to file
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Load metadata from file
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let metadata: StatePruneMetadata = serde_json::from_str(&content)?;
        Ok(metadata)
    }

    /// Generate metadata file name
    pub fn file_name(&self) -> String {
        let timestamp = self.started_at;
        match &self.operation_type {
            OperationType::Snapshot { tx_order, .. } => {
                format!("snapshot_meta_{}_{}.json", tx_order, timestamp)
            }
            OperationType::Replay {
                from_order,
                to_order,
                ..
            } => {
                format!("replay_meta_{}_{}_{}.json", from_order, to_order, timestamp)
            }
        }
    }
}

/// Convert from SnapshotMeta to StatePruneMetadata
impl From<(&SnapshotMeta, PathBuf)> for StatePruneMetadata {
    fn from((snapshot_meta, output_dir): (&SnapshotMeta, PathBuf)) -> Self {
        let operation_type = OperationType::Snapshot {
            tx_order: snapshot_meta.tx_order,
            state_root: format!("{:x}", snapshot_meta.state_root),
            output_dir,
        };

        let config = serde_json::json!({
            "tx_order": snapshot_meta.tx_order,
            "state_root": format!("{:x}", snapshot_meta.state_root),
            "global_size": snapshot_meta.global_size,
            "node_count": snapshot_meta.node_count,
            "version": snapshot_meta.version
        });

        Self::new(operation_type, config)
    }
}

/// Convert from ReplayReport to StatePruneMetadata
impl From<(&ReplayReport, PathBuf, u64, u64)> for StatePruneMetadata {
    fn from(
        (replay_report, snapshot_path, from_order, to_order): (&ReplayReport, PathBuf, u64, u64),
    ) -> Self {
        let operation_type = OperationType::Replay {
            snapshot_path: snapshot_path.clone(),
            from_order,
            to_order,
            output_dir: snapshot_path.clone(), // This should be set properly in actual implementation
        };

        let config = serde_json::json!({
            "from_order": from_order,
            "to_order": to_order,
            "snapshot_path": snapshot_path,
            "verify_final_state_root": replay_report.verification_passed
        });

        let mut metadata = Self::new(operation_type, config);
        metadata.statistics = OperationStatistics {
            nodes_processed: replay_report.nodes_updated,
            bytes_processed: replay_report.statistics.data_size_bytes,
            peak_memory_bytes: replay_report.statistics.peak_memory_bytes,
            duration_seconds: replay_report.duration_seconds,
            custom_metrics: serde_json::json!({
                "objects_created": replay_report.statistics.objects_created,
                "objects_updated": replay_report.statistics.objects_updated,
                "objects_deleted": replay_report.statistics.objects_deleted,
                "changesets_processed": replay_report.changesets_processed
            }),
        };

        if replay_report.is_success() {
            metadata.mark_completed();
        } else {
            metadata.mark_failed("Replay verification failed".to_string());
            metadata.errors = replay_report.errors.clone();
        }

        metadata
    }
}
