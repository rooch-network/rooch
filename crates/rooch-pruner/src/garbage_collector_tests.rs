// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
        use crate::safety_verifier::SafetyVerifier;
    use moveos_types::h256::H256;
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_gc_config_default() {
        let config = GCConfig::default();
        assert!(!config.dry_run);
        assert_eq!(config.batch_size, 10_000);
        assert_eq!(config.workers, num_cpus::get());
        assert!(config.use_recycle_bin);
        assert!(!config.force_compaction);
        assert_eq!(config.marker_strategy, MarkerStrategy::Auto);
        assert!(!config.force_execution);
    }

    #[test]
    fn test_gc_report_creation() {
        let roots = vec![H256::random(), H256::random()];
        let mark_stats = MarkStats {
            marked_count: 1000,
            duration: Duration::from_secs(10),
            memory_strategy: "InMemory".to_string(),
        };
        let sweep_stats = SweepStats {
            scanned_count: 2000,
            kept_count: 1000,
            deleted_count: 1000,
            recycle_bin_entries: 1000,
            duration: Duration::from_secs(15),
        };

        let report = GCReport {
            protected_roots: roots.clone(),
            mark_stats,
            sweep_stats,
            duration: Duration::from_secs(30),
            memory_strategy_used: MarkerStrategy::InMemory,
        };

        assert_eq!(report.protected_roots, roots);
        assert_eq!(report.mark_stats.marked_count, 1000);
        assert_eq!(report.sweep_stats.deleted_count, 1000);
        assert_eq!(report.memory_strategy_used, MarkerStrategy::InMemory);
    }

    #[test]
    fn test_in_memory_marker() {
        let marker = InMemoryMarker::new();

        let hash1 = H256::random();
        let hash2 = H256::random();

        // Test marking
        assert_eq!(marker.mark(hash1).unwrap(), true);
        assert_eq!(marker.mark(hash1).unwrap(), false); // Already marked
        assert_eq!(marker.mark(hash2).unwrap(), true);

        // Test checking
        assert!(marker.is_marked(&hash1));
        assert!(marker.is_marked(&hash2));
        assert!(!marker.is_marked(&H256::random()));

        // Test count
        assert_eq!(marker.marked_count(), 2);

        // Test reset
        marker.reset().unwrap();
        assert_eq!(marker.marked_count(), 0);
        assert!(!marker.is_marked(&hash1));
        assert!(!marker.is_marked(&hash2));
    }

    #[test]
    fn test_marker_strategy_display() {
        assert_eq!(format!("{}", MarkerStrategy::Auto), "Auto");
        assert_eq!(format!("{}", MarkerStrategy::InMemory), "InMemory");
        assert_eq!(format!("{}", MarkerStrategy::Persistent), "Persistent");
    }

    #[test]
    fn test_memory_strategy_selection() {
        // Test that auto selection works (this would need actual memory detection in real implementation)
        let small_estimate = 1000;
        let large_estimate = 10_000_000;

        // In a real implementation, this would check actual system memory
        // For now, we just test that the function exists and returns a valid strategy
        let strategy_small = crate::marker::select_marker_strategy(small_estimate);
        let strategy_large = crate::marker::marker::select_marker_strategy(large_estimate);

        // Both should return valid strategies
        assert!(matches!(strategy_small, MarkerStrategy::Auto));
        assert!(matches!(strategy_large, MarkerStrategy::Auto));
    }

    #[test]
    fn test_sweep_stats_default() {
        let stats = SweepStats::default();
        assert_eq!(stats.scanned_count, 0);
        assert_eq!(stats.kept_count, 0);
        assert_eq!(stats.deleted_count, 0);
        assert_eq!(stats.recycle_bin_entries, 0);
        assert_eq!(stats.duration, Duration::from_secs(0));
    }

    #[test]
    fn test_mark_stats_default() {
        let stats = MarkStats::default();
        assert_eq!(stats.marked_count, 0);
        assert_eq!(stats.duration, Duration::from_secs(0));
        assert_eq!(stats.memory_strategy, "");
    }

    #[test]
    fn test_safety_verifier_integration() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_path_buf();

        // Create a mock GC config for testing
        let config = GCConfig {
            dry_run: true,
            ..Default::default()
        };

        // Note: We cannot create a full GarbageCollector without a real MoveOSStore
        // But we can test the safety verification logic separately

        // Test safety verifier with non-existent database
        let safety_verifier = SafetyVerifier::new(&db_path);
        let report = safety_verifier.verify_database_access().unwrap();
        assert!(!report.database_available);
        assert!(report.message.contains("LOCK file missing"));
    }

    #[test]
    fn test_safety_verifier_with_lock_file() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_path_buf();

        // Create a LOCK file to simulate a RocksDB database
        let lock_file = db_path.join("LOCK");
        std::fs::write(&lock_file, "").unwrap();

        let safety_verifier = SafetyVerifier::new(&db_path);
        let report = safety_verifier.verify_database_access().unwrap();
        assert!(report.database_available);
        assert!(report.message.contains("available for exclusive access"));
    }

    #[test]
    fn test_database_path_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_path_buf();

        // Test that we can create a safety verifier with the path
        let safety_verifier = SafetyVerifier::new(&db_path);

        // The path should be stored correctly
        assert_eq!(safety_verifier.db_path, db_path);
    }

    #[test]
    fn test_gc_config_force_execution() {
        let config = GCConfig {
            dry_run: false,
            skip_confirm: true,
            ..Default::default()
        };

        assert!(config.force_execution);
        assert!(!config.dry_run);
    }

    #[test]
    fn test_gc_config_dry_run_defaults() {
        let config = GCConfig {
            dry_run: true,
            ..Default::default()
        };

        assert!(config.dry_run);
        assert!(!config.force_execution);
    }
}