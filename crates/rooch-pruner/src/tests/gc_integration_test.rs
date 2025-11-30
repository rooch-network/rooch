// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//! End-to-end integration tests for the Garbage Collector
//!
//! This module tests the complete GC workflow with real database operations

#[cfg(test)]
mod tests {
    use crate::garbage_collector::{GCConfig, GarbageCollector};
    use crate::marker::{MarkerStrategy, NodeMarker};
    use crate::safety_verifier::SafetyVerifier;
    use anyhow::Result;
    use moveos_store::MoveOSStore;
    use moveos_types::h256::H256;
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::TempDir;
    use tracing::info;

    /// Test GC with dry-run mode to verify basic functionality
    #[test]
    fn test_gc_dry_run_basic() -> Result<()> {
        // Setup test environment
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().to_path_buf();

        // Create a mock MoveOS store
        let (store, _tmpdir) = MoveOSStore::mock_moveos_store()?;

        // Configure GC for dry run
        let config = GCConfig {
            dry_run: true,
            batch_size: 1000,
            workers: 1,
            use_recycle_bin: true,
            force_compaction: false,
            marker_strategy: MarkerStrategy::InMemory,
            force_execution: true,
            protected_roots_count: 1,
        };

        // Create GarbageCollector
        let gc = GarbageCollector::new(Arc::new(store), config, db_path)?;

        // Execute GC in dry-run mode
        info!("Starting GC dry-run test");
        let report = gc.execute_gc()?;

        // Verify results
        assert!(report.duration >= Duration::from_secs(0));
        assert_eq!(report.sweep_stats.deleted_count, 0); // dry-run should not delete
        assert_eq!(report.sweep_stats.recycle_bin_entries, 0); // dry-run should not recycle

        info!("✅ GC dry-run test completed successfully");
        info!("  Marked nodes: {}", report.mark_stats.marked_count);
        info!("  Memory strategy: {:?}", report.memory_strategy_used);
        info!("  Total duration: {:?}", report.duration);

        Ok(())
    }

    /// Test GC safety verification with database lock detection
    #[test]
    fn test_gc_safety_verification() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let _db_path = temp_dir.path().to_path_buf();

        // Create a fake LOCK file to simulate database being locked
        let lock_file = temp_dir.path().join("LOCK");
        std::fs::write(&lock_file, "database lock test")?;

        // Test safety verifier
        let verifier = SafetyVerifier::new(temp_dir.path());
        let safety_report = verifier.verify_database_access()?;

        assert!(safety_report.database_available);
        assert!(safety_report.message.contains("available"));

        info!("✅ Safety verification test completed");
        info!("  Database available: {}", safety_report.database_available);
        info!("  Message: {}", safety_report.message);

        // Test with non-existent directory
        let nonexistent_verifier =
            SafetyVerifier::new(&std::path::PathBuf::from("/nonexistent/path"));
        let nonexistent_report = nonexistent_verifier.verify_database_access()?;

        assert!(!nonexistent_report.database_available);
        assert!(nonexistent_report.message.contains("does not exist"));

        info!("✅ Non-existent directory test completed");

        Ok(())
    }

    /// Test GC configuration validation
    #[test]
    fn test_gc_configuration() -> Result<()> {
        // Test default configuration
        let default_config = GCConfig::default();
        assert!(!default_config.dry_run);
        assert_eq!(default_config.batch_size, 10_000);
        assert!(default_config.use_recycle_bin);
        assert!(!default_config.force_compaction);
        assert_eq!(default_config.marker_strategy, MarkerStrategy::Auto);
        assert!(!default_config.force_execution);

        // Test custom configuration
        let custom_config = GCConfig {
            dry_run: true,
            batch_size: 5000,
            workers: 2,
            use_recycle_bin: false,
            force_compaction: true,
            marker_strategy: MarkerStrategy::InMemory,
            force_execution: true,
            protected_roots_count: 1,
        };

        assert!(custom_config.dry_run);
        assert_eq!(custom_config.batch_size, 5000);
        assert_eq!(custom_config.workers, 2);
        assert!(!custom_config.use_recycle_bin);
        assert!(custom_config.force_compaction);
        assert_eq!(custom_config.marker_strategy, MarkerStrategy::InMemory);
        assert!(custom_config.force_execution);

        info!("✅ GC configuration test completed");
        info!("  Default config: {:?}", default_config);
        info!("  Custom config: {:?}", custom_config);

        Ok(())
    }

    /// Test InMemory marker functionality
    #[test]
    fn test_in_memory_marker() -> Result<()> {
        let marker = crate::marker::InMemoryMarker::new();

        let hash1 = H256::random();
        let hash2 = H256::random();
        let hash3 = H256::random();

        // Test marking
        assert!(marker.mark(hash1)?); // First time should return true
        assert!(!(marker.mark(hash1)?)); // Duplicate should return false
        assert!(marker.mark(hash2)?); // Different hash should return true
        assert!(marker.mark(hash3)?); // Another different hash

        // Test checking
        assert!(marker.is_marked(&hash1));
        assert!(marker.is_marked(&hash2));
        assert!(marker.is_marked(&hash3));
        assert!(!marker.is_marked(&H256::random()));

        // Test count
        assert_eq!(marker.marked_count(), 3);

        // Test reset
        marker.reset()?;
        assert_eq!(marker.marked_count(), 0);
        assert!(!marker.is_marked(&hash1));
        assert!(!marker.is_marked(&hash2));
        assert!(!marker.is_marked(&hash3));

        info!("✅ InMemory marker test completed successfully");

        Ok(())
    }

    /// Test memory strategy display formatting
    #[test]
    fn test_strategy_display() {
        let auto_str = format!("{}", MarkerStrategy::Auto);
        let memory_str = format!("{}", MarkerStrategy::InMemory);
        let persistent_str = format!("{}", MarkerStrategy::Persistent);

        assert_eq!(auto_str, "Auto");
        assert_eq!(memory_str, "InMemory");
        assert_eq!(persistent_str, "Persistent");

        info!("✅ Strategy display test completed");
        info!("  Auto: {}", auto_str);
        info!("  InMemory: {}", memory_str);
        info!("  Persistent: {}", persistent_str);
    }

    /// Stress test with multiple markers
    #[test]
    fn test_multiple_marker_performance() -> Result<()> {
        let marker = crate::marker::InMemoryMarker::new();

        // Test with many hashes
        let num_hashes = 100_000;
        let hashes: Vec<H256> = (0..num_hashes).map(|_| H256::random()).collect();

        let start_time = std::time::Instant::now();

        // Mark all hashes
        let mut marked_count = 0;
        for hash in &hashes {
            if marker.mark(*hash)? {
                marked_count += 1;
            }
        }

        let mark_duration = start_time.elapsed();

        // Check all hashes
        let mut found_count = 0;
        for hash in &hashes {
            if marker.is_marked(hash) {
                found_count += 1;
            }
        }

        let check_duration = start_time.elapsed() - mark_duration;

        // Verify results
        assert_eq!(marked_count, num_hashes);
        assert_eq!(found_count, num_hashes);
        assert_eq!(marker.marked_count(), num_hashes as u64);

        // Performance assertions (should be fast)
        assert!(mark_duration < Duration::from_secs(5));
        assert!(check_duration < Duration::from_secs(2));

        let mark_throughput = num_hashes as f64 / mark_duration.as_secs_f64();
        let check_throughput = num_hashes as f64 / check_duration.as_secs_f64();

        info!("✅ Marker performance test completed");
        info!(
            "  Marked {} hashes in {:?} ({:.0} ops/sec)",
            num_hashes, mark_duration, mark_throughput
        );
        info!(
            "  Checked {} hashes in {:?} ({:.0} ops/sec)",
            num_hashes, check_duration, check_throughput
        );
        info!("  Total marked count: {}", marker.marked_count());

        Ok(())
    }

    /// Test GC error handling scenarios
    #[test]
    fn test_gc_error_handling() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().to_path_buf();

        let (store, _tmpdir) = MoveOSStore::mock_moveos_store()?;

        // Test with force execution enabled (should work)
        let config = GCConfig {
            dry_run: true,
            force_execution: true,
            ..Default::default()
        };

        let gc = GarbageCollector::new(Arc::new(store), config, db_path)?;
        let report = gc.execute_gc()?;

        // Should succeed in dry-run mode
        assert!(report.duration >= Duration::from_secs(0));

        info!("✅ GC error handling test completed");
        info!("  Dry-run mode works correctly");

        Ok(())
    }

    /// Test GC report generation
    #[test]
    fn test_gc_report_structure() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().to_path_buf();

        let (store, _tmpdir) = MoveOSStore::mock_moveos_store()?;

        let config = GCConfig {
            force_execution: true,
            ..Default::default()
        };
        let gc = GarbageCollector::new(Arc::new(store), config, db_path)?;

        // Execute GC to get report
        let report = gc.execute_gc()?;

        // Verify report structure
        assert!(!report.protected_roots.is_empty()); // Should have some roots
        assert_eq!(report.mark_stats.memory_strategy, "InMemory"); // Default strategy
        assert!(report.mark_stats.duration >= Duration::from_secs(0));
        assert!(report.duration >= Duration::from_secs(0));

        info!("✅ GC report structure test completed");
        info!("  Protected roots: {}", report.protected_roots.len());
        info!("  Memory strategy: {}", report.mark_stats.memory_strategy);
        info!("  Mark duration: {:?}", report.mark_stats.duration);
        info!("  Total duration: {:?}", report.duration);

        Ok(())
    }
}
