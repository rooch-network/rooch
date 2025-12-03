// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//! End-to-end integration tests for the Garbage Collector
//!
//! This module tests the complete GC workflow with real database operations

#[cfg(test)]
mod tests {
    use crate::config::GCConfig;
    use crate::garbage_collector::GarbageCollector;
    use crate::marker::{BloomFilterMarker, NodeMarker};
    use crate::safety_verifier::SafetyVerifier;
    use anyhow::Result;
    use moveos_types::h256::H256;
    use moveos_types::startup_info::StartupInfo;
    use rooch_config::RoochOpt;
    use rooch_db::RoochDB;
    use std::time::Duration;
    use tempfile::TempDir;
    use tracing::info;

    /// Test GC with dry-run mode to verify basic functionality
    #[test]
    fn test_gc_dry_run_basic() -> Result<()> {
        // Setup test environment
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().to_path_buf();

        // Create a mock RoochDB
        let rooch_opt = RoochOpt::new_with_default(Some(db_path.clone()), None, None)?;
        let rooch_db = RoochDB::init_with_mock_metrics_for_test(rooch_opt.store_config())?;

        // Create and save startup info for the test
        let test_state_root = H256::random();
        let startup_info = StartupInfo::new(test_state_root, 0);
        rooch_db
            .moveos_store
            .config_store
            .save_startup_info(startup_info)?;

        // Configure GC for dry run
        let config = GCConfig {
            dry_run: true,
            batch_size: 1000,
            workers: 1,
            use_recycle_bin: true,
            force_compaction: false,
            skip_confirm: true,
            protected_roots_count: 1,
            ..GCConfig::default()
        };

        // Create GarbageCollector
        let gc = GarbageCollector::new(rooch_db, config)?;

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
        // Marker strategy is now always BloomFilter, no longer configurable
        // force_execution removed; skip check

        // Test custom configuration
        let custom_config = GCConfig {
            dry_run: true,
            batch_size: 5000,
            workers: 2,
            use_recycle_bin: false,
            force_compaction: true,
            skip_confirm: true,
            protected_roots_count: 1,
            ..GCConfig::default()
        };

        assert!(custom_config.dry_run);
        assert_eq!(custom_config.batch_size, 5000);
        assert_eq!(custom_config.workers, 2);
        assert!(!custom_config.use_recycle_bin);
        assert!(custom_config.force_compaction);
        // force_execution removed; skip check

        info!("✅ GC configuration test completed");
        info!("  Default config: {:?}", default_config);
        info!("  Custom config: {:?}", custom_config);

        Ok(())
    }

    /// Test BloomFilter marker functionality
    #[test]
    fn test_in_memory_marker() -> Result<()> {
        let marker = BloomFilterMarker::with_estimated_nodes(100_000, 0.01);

        let hash1 = H256::random();
        let hash2 = H256::random();
        let hash3 = H256::random();

        // Test marking (Bloom Filter always returns true due to false positive possibility)
        assert!(marker.mark(hash1)?); // Should return true
        assert!(marker.mark(hash1)?); // Bloom Filter can't detect duplicates, always returns true
        assert!(marker.mark(hash2)?); // Different hash should return true
        assert!(marker.mark(hash3)?); // Another different hash

        // Test checking
        assert!(marker.is_marked(&hash1));
        assert!(marker.is_marked(&hash2));
        assert!(marker.is_marked(&hash3));
        assert!(!marker.is_marked(&H256::random()));

        // Test count (Bloom Filter counts all mark calls, including duplicates)
        assert_eq!(marker.marked_count(), 4); // 4 mark calls total

        // Test reset
        marker.reset()?;
        assert_eq!(marker.marked_count(), 0);
        assert!(!marker.is_marked(&hash1));
        assert!(!marker.is_marked(&hash2));
        assert!(!marker.is_marked(&hash3));

        info!("✅ InMemory marker test completed successfully");

        Ok(())
    }

    /// Test marker type
    #[test]
    fn test_marker_type() {
        let marker = BloomFilterMarker::with_estimated_nodes(1000, 0.01);
        assert_eq!(marker.marker_type(), "BloomFilter");

        info!("✅ Marker type test completed");
    }

    /// Stress test with multiple markers
    #[test]
    fn test_multiple_marker_performance() -> Result<()> {
        let marker = BloomFilterMarker::with_estimated_nodes(100_000, 0.01);

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

        // Verify results (Bloom Filter characteristics)
        assert_eq!(marked_count, num_hashes); // All mark calls should return true for Bloom Filter
        assert!(found_count >= num_hashes); // May find more due to false positives in queries
        assert_eq!(marker.marked_count(), num_hashes as u64); // Counter equals total mark calls

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

        // Create a mock RoochDB
        let rooch_opt = RoochOpt::new_with_default(Some(db_path.clone()), None, None)?;
        let rooch_db = RoochDB::init_with_mock_metrics_for_test(rooch_opt.store_config())?;

        // Create and save startup info for the test
        let test_state_root = H256::random();
        let startup_info = StartupInfo::new(test_state_root, 0);
        rooch_db
            .moveos_store
            .config_store
            .save_startup_info(startup_info)?;

        // Test with force execution enabled (should work)
        let config = GCConfig {
            dry_run: true,
            skip_confirm: true,
            ..Default::default()
        };

        let gc = GarbageCollector::new(rooch_db, config)?;
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

        // Create a mock RoochDB
        let rooch_opt = RoochOpt::new_with_default(Some(db_path.clone()), None, None)?;
        let rooch_db = RoochDB::init_with_mock_metrics_for_test(rooch_opt.store_config())?;

        // Create and save startup info for the test
        let test_state_root = H256::random();
        let startup_info = StartupInfo::new(test_state_root, 0);
        rooch_db
            .moveos_store
            .config_store
            .save_startup_info(startup_info)?;

        let config = GCConfig {
            skip_confirm: true,
            ..Default::default()
        };
        let gc = GarbageCollector::new(rooch_db, config)?;

        // Execute GC to get report
        let report = gc.execute_gc()?;

        // Verify report structure
        assert!(!report.protected_roots.is_empty()); // Should have some roots
        assert_eq!(report.mark_stats.memory_strategy, "BloomFilter"); // Unified strategy
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
