// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_types::h256::H256;
use rooch_config::state_prune::{ReplayReport, SnapshotMeta};
use rooch_pruner::state_prune::{
    DeduplicationStrategy, OperationStatus, OperationType, ProgressTracker, SnapshotBuilderConfig,
    SnapshotProgress, StatePruneMetadata,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_builder_config_default() {
        let config = SnapshotBuilderConfig::default();
        assert_eq!(config.batch_size, 10000);
        assert_eq!(config.workers, 4);
        assert_eq!(config.memory_limit, 16 * 1024 * 1024 * 1024);
        assert_eq!(config.progress_interval_seconds, 30);
        assert!(config.enable_progress_tracking);
        assert!(config.enable_resume);
        assert_eq!(config.max_traversal_time_hours, 24);

        // Updated defaults for scalable deduplication
        assert!(!config.enable_bloom_filter); // Disabled in favor of RocksDB strategy
        assert_eq!(config.bloom_filter_fp_rate, 0.001);
        assert_eq!(config.deduplication_batch_size, 0); // Use same as processing batch size
        assert!(config.enable_adaptive_batching);
        assert_eq!(config.memory_pressure_threshold, 0.8);

        // Verify default deduplication strategy is RocksDB
        assert!(matches!(
            config.deduplication_strategy,
            DeduplicationStrategy::RocksDB
        ));
    }

    #[test]
    fn test_snapshot_builder_config_validation() {
        let mut config = SnapshotBuilderConfig::default();

        // Valid config should pass
        assert!(config.validate().is_ok());

        // Invalid batch size
        config.batch_size = 0;
        assert!(config.validate().is_err());

        // Reset and test invalid workers
        config = SnapshotBuilderConfig::default();
        config.workers = 0;
        assert!(config.validate().is_err());

        // Reset and test invalid bloom filter fp rate
        config = SnapshotBuilderConfig::default();
        config.bloom_filter_fp_rate = 1.5;
        assert!(config.validate().is_err());

        // Reset and test invalid memory pressure threshold
        config = SnapshotBuilderConfig::default();
        config.memory_pressure_threshold = 1.5; // Invalid > 1.0
        assert!(config.validate().is_err());

        config = SnapshotBuilderConfig::default();
        config.memory_pressure_threshold = -0.1; // Invalid < 0.0
        assert!(config.validate().is_err());

        // Valid edge cases
        config = SnapshotBuilderConfig::default();
        config.memory_pressure_threshold = 0.0; // Valid (lower bound)
        assert!(config.validate().is_ok());

        config = SnapshotBuilderConfig::default();
        config.memory_pressure_threshold = 0.999; // Valid (upper bound, exclusive)
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_progress_tracker() {
        let tracker = ProgressTracker::new(10);

        // Test initial state
        assert_eq!(tracker.progress_percentage(), 0.0);
        assert_eq!(tracker.get_progress_report().total_items, 0);
        assert_eq!(tracker.get_progress_report().processed_items, 0);

        // Set total and increment processed
        tracker.set_total(100);
        tracker.increment_processed(25);

        assert_eq!(tracker.progress_percentage(), 25.0);
        assert_eq!(tracker.get_progress_report().total_items, 100);
        assert_eq!(tracker.get_progress_report().processed_items, 25);

        // Test completion
        tracker.increment_processed(75);
        assert_eq!(tracker.progress_percentage(), 100.0);
        assert!(tracker.get_progress_report().is_complete());
    }

    #[test]
    fn test_progress_tracker_report_format() {
        let tracker = ProgressTracker::new(1);
        tracker.set_total(1000);
        tracker.increment_processed(250);

        let report = tracker.get_progress_report();
        let formatted = report.format();
        assert!(formatted.contains("25.00%"));
        assert!(formatted.contains("250/1000"));
    }

    #[test]
    fn test_replay_report_creation() {
        let mut report = ReplayReport::new();

        assert_eq!(report.changesets_processed, 0);
        assert_eq!(report.nodes_updated, 0);
        assert_eq!(report.final_state_root, H256::zero());
        assert!(!report.verification_passed);
        assert_eq!(report.duration_seconds, 0);
        assert!(report.errors.is_empty());
        assert!(!report.is_success());

        // Test adding errors
        report.add_error("Test error".to_string());
        assert_eq!(report.errors.len(), 1);
        assert!(!report.is_success());

        // Test verification
        report.verification_passed = true;
        // Still false because errors remain
        assert!(!report.is_success());
    }

    #[test]
    fn test_snapshot_meta_creation() {
        let state_root = H256::random();
        let snapshot_meta = SnapshotMeta::new(1000, state_root, 50000, 250000);

        assert_eq!(snapshot_meta.tx_order, 1000);
        assert_eq!(snapshot_meta.state_root, state_root);
        assert_eq!(snapshot_meta.global_size, 50000);
        assert_eq!(snapshot_meta.node_count, 250000);
        assert_eq!(snapshot_meta.version, 1);
        assert!(snapshot_meta.created_at > 0);

        // Test validation
        assert!(snapshot_meta.validate().is_ok());
    }

    #[test]
    fn test_snapshot_meta_validation() {
        let state_root = H256::random();

        // Valid meta should pass (tx_order can be 0 per validation rules)
        let valid_meta = SnapshotMeta::new(0, state_root, 100, 200);
        assert!(valid_meta.validate().is_ok());

        // Invalid state root
        let invalid_meta = SnapshotMeta::new(1, H256::zero(), 100, 200);
        assert!(invalid_meta.validate().is_err());
    }

    #[test]
    fn test_snapshot_meta_file_operations() {
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path();

        let state_root = H256::random();
        let snapshot_meta = SnapshotMeta::new(12345, state_root, 1000, 5000);

        // Test save and load
        let saved_path = snapshot_meta.save_to_file(test_file).unwrap();
        let loaded_meta = SnapshotMeta::load_from_file(&saved_path).unwrap();

        assert_eq!(loaded_meta.tx_order, snapshot_meta.tx_order);
        assert_eq!(loaded_meta.state_root, snapshot_meta.state_root);
        assert_eq!(loaded_meta.global_size, snapshot_meta.global_size);
        assert_eq!(loaded_meta.node_count, snapshot_meta.node_count);
    }

    #[test]
    fn test_state_prune_metadata_creation() {
        let operation_type = OperationType::Snapshot {
            tx_order: 1000,
            state_root: format!("{:x}", H256::random()),
            output_dir: std::path::PathBuf::from("/test/output"),
        };

        let config = serde_json::json!({
            "test": true
        });

        let metadata = StatePruneMetadata::new(operation_type, config);

        assert!(metadata.started_at > 0);
        assert_eq!(metadata.completed_at, 0);
        assert!(matches!(metadata.status, OperationStatus::Pending));
        assert!(metadata.errors.is_empty());
    }

    #[test]
    fn test_state_prune_metadata_lifecycle() {
        let mut metadata = StatePruneMetadata::new(
            OperationType::Snapshot {
                tx_order: 0,
                state_root: format!("{:x}", H256::random()),
                output_dir: std::path::PathBuf::from("/test"),
            },
            serde_json::json!({}),
        );

        // Test progress tracking
        metadata.mark_in_progress("Test step".to_string(), 50.0);
        if let OperationStatus::InProgress { progress, .. } = metadata.status {
            assert_eq!(progress, 50.0);
        } else {
            panic!("Expected InProgress status");
        }

        // Test completion
        metadata.mark_completed();
        assert!(metadata.is_finished());
        assert!(metadata.completed_at > 0);

        // Test failure
        let mut failed_metadata = StatePruneMetadata::new(
            OperationType::Snapshot {
                tx_order: 0,
                state_root: format!("{:x}", H256::random()),
                output_dir: std::path::PathBuf::from("/test"),
            },
            serde_json::json!({}),
        );
        failed_metadata.mark_failed("Test error".to_string());
        assert!(failed_metadata.is_finished());
        assert_eq!(failed_metadata.errors.len(), 1);
        assert_eq!(failed_metadata.errors[0], "Test error");
    }

    #[test]
    fn test_state_prune_metadata_file_operations() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_metadata.json");

        let metadata = StatePruneMetadata::new(
            OperationType::Snapshot {
                tx_order: 1000,
                state_root: format!("{:x}", H256::random()),
                output_dir: std::path::PathBuf::from("/test"),
            },
            serde_json::json!({"test": true}),
        );

        // Test save and load
        metadata.save_to_file(&test_file).unwrap();
        let loaded_metadata = StatePruneMetadata::load_from_file(&test_file).unwrap();

        assert_eq!(loaded_metadata.started_at, metadata.started_at);
        assert_eq!(
            std::mem::discriminant(&loaded_metadata.status),
            std::mem::discriminant(&metadata.status)
        );

        // Cleanup
        std::fs::remove_file(&test_file).unwrap();
    }

    #[test]
    fn test_snapshot_progress_creation() {
        let state_root = H256::random();
        let worklist = vec![H256::random(), H256::random(), H256::random()];
        let statistics = rooch_pruner::state_prune::TraversalStatistics {
            nodes_visited: 100,
            bytes_processed: 50000,
        };
        let batch_buffer = vec![(H256::random(), b"test_data".to_vec())];

        let progress = SnapshotProgress {
            state_root,
            worklist: worklist.clone(),
            worklist_position: 1,
            statistics: statistics.clone(),
            batch_buffer: batch_buffer.clone(),
            current_batch_size: 1000,
            last_save_timestamp: 1234567890,
            nodes_written: 95,
            checkpoint_id: SnapshotProgress::generate_checkpoint_id(state_root, 100),
        };

        assert_eq!(progress.state_root, state_root);
        assert_eq!(progress.worklist, worklist);
        assert_eq!(progress.worklist_position, 1);
        assert_eq!(progress.statistics.nodes_visited, 100);
        assert_eq!(progress.statistics.bytes_processed, 50000);
        assert_eq!(progress.current_batch_size, 1000);
        assert_eq!(progress.nodes_written, 95);
        assert!(progress.checkpoint_id.contains(&format!("{:x}", state_root)));
        assert!(progress.checkpoint_id.contains("100"));
    }

    #[test]
    fn test_snapshot_progress_checkpoint_id() {
        let state_root = H256::random();
        let checkpoint_id = SnapshotProgress::generate_checkpoint_id(state_root, 42);

        let expected = format!("{:x}-42", state_root);
        assert_eq!(checkpoint_id, expected);
    }

    #[test]
    fn test_snapshot_progress_save_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let progress_path = temp_dir.path().join("test_progress.json");

        let state_root = H256::random();
        let original_progress = SnapshotProgress {
            state_root,
            worklist: vec![H256::random(), H256::random()],
            worklist_position: 0,
            statistics: rooch_pruner::state_prune::TraversalStatistics {
                nodes_visited: 500,
                bytes_processed: 250000,
            },
            batch_buffer: vec![(H256::random(), b"buffer_data".to_vec())],
            current_batch_size: 2000,
            last_save_timestamp: 9876543210,
            nodes_written: 480,
            checkpoint_id: SnapshotProgress::generate_checkpoint_id(state_root, 500),
        };

        // Save progress
        original_progress.save_to_file(&progress_path).unwrap();
        assert!(progress_path.exists());

        // Load progress with correct state_root
        let loaded_progress = SnapshotProgress::load_from_file(&progress_path, state_root).unwrap();
        assert!(loaded_progress.is_some());

        let loaded = loaded_progress.unwrap();
        assert_eq!(loaded.state_root, original_progress.state_root);
        assert_eq!(loaded.worklist, original_progress.worklist);
        assert_eq!(loaded.worklist_position, original_progress.worklist_position);
        assert_eq!(loaded.statistics.nodes_visited, original_progress.statistics.nodes_visited);
        assert_eq!(loaded.statistics.bytes_processed, original_progress.statistics.bytes_processed);
        assert_eq!(loaded.current_batch_size, original_progress.current_batch_size);
        assert_eq!(loaded.nodes_written, original_progress.nodes_written);
        assert_eq!(loaded.checkpoint_id, original_progress.checkpoint_id);
    }

    #[test]
    fn test_snapshot_progress_state_root_mismatch() {
        let temp_dir = tempfile::tempdir().unwrap();
        let progress_path = temp_dir.path().join("test_progress.json");

        let state_root = H256::random();
        let different_state_root = H256::random();

        let progress = SnapshotProgress {
            state_root,
            worklist: vec![H256::random()],
            worklist_position: 0,
            statistics: rooch_pruner::state_prune::TraversalStatistics::default(),
            batch_buffer: vec![],
            current_batch_size: 1000,
            last_save_timestamp: 0,
            nodes_written: 0,
            checkpoint_id: SnapshotProgress::generate_checkpoint_id(state_root, 0),
        };

        // Save progress
        progress.save_to_file(&progress_path).unwrap();

        // Try to load with different state_root - should return None
        let loaded_progress = SnapshotProgress::load_from_file(&progress_path, different_state_root).unwrap();
        assert!(loaded_progress.is_none());
    }

    #[test]
    fn test_snapshot_progress_nonexistent_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let nonexistent_path = temp_dir.path().join("nonexistent.json");

        let state_root = H256::random();
        let loaded_progress = SnapshotProgress::load_from_file(&nonexistent_path, state_root).unwrap();
        assert!(loaded_progress.is_none());
    }

    #[test]
    fn test_snapshot_progress_backup_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let progress_path = temp_dir.path().join("test_progress.json");
        let backup_path = temp_dir.path().join("test_progress.backup");

        let state_root = H256::random();
        let progress1 = SnapshotProgress {
            state_root,
            worklist: vec![H256::random()],
            worklist_position: 0,
            statistics: rooch_pruner::state_prune::TraversalStatistics {
                nodes_visited: 100,
                bytes_processed: 1000,
            },
            batch_buffer: vec![],
            current_batch_size: 1000,
            last_save_timestamp: 0,
            nodes_written: 95,
            checkpoint_id: SnapshotProgress::generate_checkpoint_id(state_root, 100),
        };

        // Save initial progress
        progress1.save_to_file(&progress_path).unwrap();

        // Create updated progress
        let progress2 = SnapshotProgress {
            state_root,
            worklist: vec![H256::random(), H256::random()],
            worklist_position: 1,
            statistics: rooch_pruner::state_prune::TraversalStatistics {
                nodes_visited: 200,
                bytes_processed: 2000,
            },
            batch_buffer: vec![(H256::random(), b"data".to_vec())],
            current_batch_size: 1500,
            last_save_timestamp: 100,
            nodes_written: 195,
            checkpoint_id: SnapshotProgress::generate_checkpoint_id(state_root, 200),
        };

        // Save updated progress - should create backup
        progress2.save_to_file(&progress_path).unwrap();

        // Verify backup exists and contains old progress
        assert!(backup_path.exists());
        let backup_progress = SnapshotProgress::load_from_file(&backup_path, state_root).unwrap();
        assert!(backup_progress.is_some());

        let backup = backup_progress.unwrap();
        assert_eq!(backup.statistics.nodes_visited, 100);
        assert_eq!(backup.nodes_written, 95);
    }
}
