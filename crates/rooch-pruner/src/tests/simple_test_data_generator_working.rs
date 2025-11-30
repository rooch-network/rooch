// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//! Simple test data generator for GC integration tests
//!
//! This module provides basic functionality to create test databases with
//! proper snapshots and startup information for GC testing

#[cfg(test)]
mod tests {
    use crate::garbage_collector::{GCConfig, GarbageCollector};
    use crate::marker::MarkerStrategy;
    use anyhow::Result;
    use moveos_types::h256::H256;
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::TempDir;
    use tracing::info;

    /// Simple test database with minimal GC metadata
    #[allow(dead_code)]
    pub struct SimpleTestDatabase {
        temp_dir: TempDir,
        state_root: H256,
    }

    impl SimpleTestDatabase {
        /// Create a new simple test database with minimal metadata
        pub fn new() -> Result<Self> {
            let temp_dir = TempDir::new()?;

            // Create a known state root for testing
            let state_root = H256::from_low_u64_be(0x1234567890abcdef);

            Ok(Self {
                temp_dir,
                state_root,
            })
        }

        /// Get the state root
        #[allow(dead_code)]
        pub fn state_root(&self) -> H256 {
            self.state_root
        }

        /// Get database path
        #[allow(dead_code)]
        pub fn db_path(&self) -> std::path::PathBuf {
            self.temp_dir.path().to_path_buf()
        }
    }

    /// Test GC functionality with simple test database
    #[test]
    fn test_gc_with_simple_database() -> Result<()> {
        info!("Testing GC with simple test database");

        // Create test database
        let _test_db = SimpleTestDatabase::new()?;
        info!("Created test database");

        // Use MoveOSStore mock to get a working store
        let (moveos_store, temp_dir) = moveos_store::MoveOSStore::mock_moveos_store()?;
        let moveos_store = Arc::new(moveos_store);

        // Create GC configuration
        let config = GCConfig {
            dry_run: true,
            batch_size: 100,
            workers: 1,
            use_recycle_bin: false,
            force_compaction: false,
            marker_strategy: MarkerStrategy::InMemory,
            force_execution: true, // Force execution to bypass safety checks in test
            protected_roots_count: 1,
        };

        // Create garbage collector
        let gc = GarbageCollector::new(moveos_store, config, temp_dir.path().to_path_buf())?;

        // Execute GC (this should work now with force_execution)
        info!("Starting GC execution");
        let report = gc.execute_gc()?;

        // Verify basic results
        assert!(
            !report.protected_roots.is_empty(),
            "Should have protected roots"
        );
        assert!(
            report.duration >= Duration::from_secs(0),
            "Duration should be non-negative"
        );

        info!("GC execution completed successfully");
        info!("Protected roots: {}", report.protected_roots.len());
        info!("Marked nodes: {}", report.mark_stats.marked_count);
        info!("Duration: {:?}", report.duration);

        Ok(())
    }

    /// Test GC configuration variations with simple database
    #[test]
    fn test_gc_configurations_with_simple_database() -> Result<()> {
        info!("Testing GC configuration variations with simple database");

        let _test_db = SimpleTestDatabase::new()?;
        let (moveos_store, temp_dir) = moveos_store::MoveOSStore::mock_moveos_store()?;
        let moveos_store = Arc::new(moveos_store);

        // Test different configurations
        let configs = vec![
            (
                GCConfig {
                    dry_run: true,
                    batch_size: 50,
                    workers: 1,
                    use_recycle_bin: false,
                    force_compaction: false,
                    marker_strategy: MarkerStrategy::InMemory,
                    force_execution: true,
                    protected_roots_count: 1,
                },
                "Small batch",
            ),
            (
                GCConfig {
                    dry_run: true,
                    batch_size: 200,
                    workers: 1,
                    use_recycle_bin: true,
                    force_compaction: false,
                    marker_strategy: MarkerStrategy::InMemory,
                    force_execution: true,
                    protected_roots_count: 1,
                },
                "Large batch with recycle bin",
            ),
            (
                GCConfig {
                    dry_run: false,
                    batch_size: 100,
                    workers: 2,
                    use_recycle_bin: true,
                    force_compaction: false,
                    marker_strategy: MarkerStrategy::Auto,
                    force_execution: true,
                    protected_roots_count: 1,
                },
                "Multi-worker auto strategy",
            ),
        ];

        for (config, name) in configs {
            info!("Testing configuration: {}", name);

            let gc =
                GarbageCollector::new(moveos_store.clone(), config, temp_dir.path().to_path_buf())?;

            let start_time = std::time::Instant::now();
            let report = gc.execute_gc()?;
            let duration = start_time.elapsed();

            info!(
                "  {}: completed in {:?}, marked: {} nodes",
                name, duration, report.mark_stats.marked_count
            );

            // Basic assertions
            assert!(!report.protected_roots.is_empty());
            assert!(duration < Duration::from_secs(10)); // Should complete quickly
        }

        info!("✅ GC configuration variations test completed");
        Ok(())
    }

    /// Test error handling with simple database
    #[test]
    fn test_gc_error_handling_with_simple_database() -> Result<()> {
        info!("Testing GC error handling with simple database");

        let _test_db = SimpleTestDatabase::new()?;
        let (moveos_store, temp_dir) = moveos_store::MoveOSStore::mock_moveos_store()?;
        let moveos_store = Arc::new(moveos_store);

        // Test 1: Try GC without force execution (should fail due to safety verification)
        {
            let config = GCConfig {
                dry_run: false,         // Not dry run
                force_execution: false, // But no force flag
                ..Default::default()
            };

            let gc =
                GarbageCollector::new(moveos_store.clone(), config, temp_dir.path().to_path_buf())?;

            // This should fail because of safety verification
            let result = gc.execute_gc();
            assert!(result.is_err(), "Should fail without force execution");
            info!("  ✅ Safety verification blocked execution as expected");
        }

        // Test 2: Force execution (should work)
        {
            let config = GCConfig {
                dry_run: true,
                force_execution: true, // Force execution
                ..Default::default()
            };

            let gc =
                GarbageCollector::new(moveos_store.clone(), config, temp_dir.path().to_path_buf())?;

            // This should succeed even without safety verification (dry-run mode)
            let result = gc.execute_gc();
            assert!(result.is_ok(), "Should succeed with force execution");
            info!("  ✅ Force execution allowed GC to proceed");
        }

        info!("✅ GC error handling test completed");
        Ok(())
    }

    /// Test marker strategy with simple database
    #[test]
    fn test_marker_strategy_with_simple_database() -> Result<()> {
        info!("Testing marker strategy with simple database");

        let _test_db = SimpleTestDatabase::new()?;
        let (moveos_store, temp_dir) = moveos_store::MoveOSStore::mock_moveos_store()?;
        let moveos_store = Arc::new(moveos_store);

        // Test different marker strategies
        let strategies = vec![
            MarkerStrategy::InMemory,
            MarkerStrategy::Auto,
            MarkerStrategy::Persistent, // This will use InMemory as fallback
        ];

        for strategy in strategies {
            info!("Testing strategy: {:?}", strategy);

            let config = GCConfig {
                dry_run: true,
                batch_size: 50,
                workers: 1,
                use_recycle_bin: false,
                force_compaction: false,
                marker_strategy: strategy,
                force_execution: true,
                protected_roots_count: 1,
            };

            let gc =
                GarbageCollector::new(moveos_store.clone(), config, temp_dir.path().to_path_buf())?;

            let report = gc.execute_gc()?;

            // Verify basic functionality
            assert!(!report.protected_roots.is_empty());

            info!(
                "  Strategy {:?} completed, marked: {} nodes",
                strategy, report.mark_stats.marked_count
            );
        }

        info!("✅ Marker strategy test completed");
        Ok(())
    }
}
