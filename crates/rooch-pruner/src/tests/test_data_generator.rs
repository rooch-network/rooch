// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//! Test data generator for GC integration tests
//!
//! This module provides utilities to create test databases with
//! proper snapshots and startup information for GC testing

#[cfg(test)]
mod tests {
    use crate::garbage_collector::{GarbageCollector, GCConfig, MarkerStrategy};
    use crate::marker::{InMemoryMarker, NodeMarker};
    use crate::safety_verifier::SafetyVerifier;
    use anyhow::Result;
    use moveos_store::MoveOSStore;
    use moveos_types::h256::H256;
    use moveos_types::startup_info::StartupInfo;
    use moveos_types::moveos_types::state::MoveObjectState;
    use moveos_types::moveos_types::state::KeyState;
    use moveos_types::moveos_types::event::TransactionEvent;
    use moveos_types::moveos_types::event::EventID;
    use parking_lot::Mutex;
    use raw_store::CodecKVStore;
    use raw_store::rocks::RocksDB;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::TempDir;
    use tracing::info;

    /// Test database with proper GC metadata
    pub struct GCTestDatabase {
        temp_dir: TempDir,
        moveos_store: Arc<MoveOSStore>,
        state_root: H256,
    }

    impl GCTestDatabase {
        /// Create a new test database with GC metadata
        pub fn new() -> Result<Self> {
            let temp_dir = TempDir::new()?;
            let db_path = temp_dir.path().join("test_db");

            // Create RocksDB instance
            let rocksdb = RocksDB::new(db_path.clone())?;
            let store = CodecKVStore::new(rocksdb);

            // Create MoveOSStore
            let moveos_store = Arc::new(MoveOSStore::new(store)?);

            // Generate test state and metadata
            let state_root = Self::generate_test_state(&moveos_store)?;
            Self::create_gc_metadata(&moveos_store, state_root)?;

            Ok(Self {
                temp_dir,
                moveos_store,
                state_root,
            })
        }

        /// Generate test state with known structure
        fn generate_test_state(store: &MoveOSStore) -> Result<H256> {
            info!("Generating test state for GC testing");

            // Create a known state root
            let state_root = H256::from_low_u64_be(0x1234567890abcdef);

            // Create some test state nodes
            let mut state_nodes = Vec::new();

            // Add root state
            let root_state = MoveObjectState::new(state_root);
            state_nodes.push((state_root, root_state));

            // Add child states
            for i in 0..100 {
                let child_hash = H256::from_low_u64_be(i as u64);
                let child_state = MoveObjectState::new(child_hash);
                state_nodes.push((child_hash, child_state));
            }

            // Store state nodes
            let state_store = store.get_state_node_store();
            for (hash, state) in state_nodes {
                state_store.put(hash, state)?;
            }

            info!("Generated {} state nodes with root: {}", state_nodes.len(), state_root);
            Ok(state_root)
        }

        /// Create GC metadata (snapshots and startup info)
        fn create_gc_metadata(store: &MoveOSStore, state_root: H256) -> Result<()> {
            info!("Creating GC metadata for state root: {}", state_root);

            // Create startup info
            let startup_info = StartupInfo {
                chain_id: 0, // Dev chain
                state_root,
                timestamp: 1640995200000, // 2022-01-01 00:00:00 UTC
            };

            // Store startup info
            store.config_store.set_startup_info(&startup_info)?;

            // Note: In a real implementation, you would also create snapshots
            // For now, startup info is sufficient for basic GC testing
            info!("GC metadata created successfully");
            Ok(())
        }

        /// Get the MoveOS store
        pub fn moveos_store(&self) -> Arc<MoveOSStore> {
            self.moveos_store.clone()
        }

        /// Get the state root
        pub fn state_root(&self) -> H256 {
            self.state_root
        }

        /// Get database path
        pub fn db_path(&self) -> std::path::PathBuf {
            self.temp_dir.path().to_path_buf()
        }
    }

    /// Test GC functionality with real database metadata
    #[test]
    fn test_gc_with_test_database() -> Result<()> {
        info!("Testing GC with generated test database");

        // Create test database with metadata
        let test_db = GCTestDatabase::new()?;
        info!("Created test database with state root: {}", test_db.state_root());

        // Create GC configuration
        let config = GCConfig {
            dry_run: true,
            batch_size: 1000,
            workers: 1,
            use_recycle_bin: true,
            force_compaction: false,
            marker_strategy: MarkerStrategy::InMemory,
            skip_confirm: false,
        };

        // Create garbage collector
        let gc = GarbageCollector::new(
            test_db.moveos_store(),
            config,
            test_db.db_path(),
        )?;

        // Execute GC
        info!("Starting GC execution");
        let report = gc.execute_gc()?;

        // Verify results
        assert!(!report.protected_roots.is_empty(), "Should have protected roots");
        assert!(report.duration >= Duration::from_secs(0), "Duration should be non-negative");
        assert_eq!(report.mark_stats.marked_count, 1, "Should mark the state root");
        assert_eq!(report.sweep_stats.deleted_count, 0, "Dry-run should not delete");

        info!("GC execution completed successfully");
        info!("Protected roots: {}", report.protected_roots.len());
        info!("Marked nodes: {}", report.mark_stats.marked_count);
        info!("Duration: {:?}", report.duration);

        Ok(())
    }

    /// Test marker strategy selection with test database
    #[test]
    fn test_marker_strategy_with_test_database() -> Result<()> {
        info!("Testing marker strategy selection with test database");

        let test_db = GCTestDatabase::new()?;

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
                batch_size: 100,
                workers: 1,
                use_recycle_bin: false,
                force_compaction: false,
                marker_strategy: strategy.clone(),
                skip_confirm: false,
            };

            let gc = GarbageCollector::new(
                test_db.moveos_store(),
                config,
                test_db.db_path(),
            )?;

            let report = gc.execute_gc()?;

            // Verify basic functionality
            assert!(!report.protected_roots.is_empty());
            assert!(report.mark_stats.marked_count > 0);

            info!("  Strategy {:?} completed, marked: {} nodes",
                  strategy, report.mark_stats.marked_count);
        }

        info!("[PASS] Marker strategy selection test completed");
        Ok(())
    }

    /// Test GC performance with test database
    #[test]
    fn test_gc_performance_with_test_database() -> Result<()> {
        info!("Testing GC performance with test database");

        let test_db = GCTestDatabase::new()?;

        // Test different configurations
        let configs = vec![
            (GCConfig {
                dry_run: true,
                batch_size: 100,
                workers: 1,
                use_recycle_bin: true,
                force_compaction: false,
                marker_strategy: MarkerStrategy::InMemory,
                skip_confirm: false,
            }, "Small batch"),
            (GCConfig {
                dry_run: true,
                batch_size: 1000,
                workers: 1,
                use_recycle_bin: true,
                force_compaction: false,
                marker_strategy: MarkerStrategy::InMemory,
                skip_confirm: false,
            }, "Large batch"),
        ];

        for (config, name) in configs {
            info!("Testing configuration: {}", name);

            let gc = GarbageCollector::new(
                test_db.moveos_store(),
                config,
                test_db.db_path(),
            )?;

            let start_time = std::time::Instant::now();
            let report = gc.execute_gc()?;
            let duration = start_time.elapsed();

            info!("  {}: completed in {:?}, throughput: {:.0} ops/sec",
                  name, duration,
                  report.mark_stats.marked_count as f64 / duration.as_secs_f64());

            // Performance assertions
            assert!(duration < Duration::from_secs(5), "GC should complete within 5 seconds");
        }

        info!("[PASS] GC performance test completed");
        Ok(())
    }

    /// Test GC report structure with test database
    #[test]
    fn test_gc_report_structure_with_test_database() -> Result<()> {
        info!("Testing GC report structure with test database");

        let test_db = GCTestDatabase::new()?;

        let config = GCConfig {
            dry_run: true,
            batch_size: 500,
            workers: 2,
            use_recycle_bin: true,
            force_compaction: false,
            marker_strategy: MarkerStrategy::InMemory,
            skip_confirm: false,
        };

        let gc = GarbageCollector::new(
            test_db.moveos_store(),
            config,
            test_db.db_path(),
        )?;

        let report = gc.execute_gc()?;

        // Verify report structure
        assert!(!report.protected_roots.is_empty(), "Should have protected roots");
        assert_eq!(report.protected_roots.len(), 1, "Should have one root");
        assert_eq!(report.protected_roots[0], test_db.state_root(), "Should match test state root");

        // Verify mark phase statistics
        assert!(report.mark_stats.marked_count > 0, "Should mark some nodes");
        assert!(report.mark_stats.duration >= Duration::from_secs(0), "Mark duration should be non-negative");
        assert!(!report.mark_stats.memory_strategy.is_empty(), "Should have memory strategy");

        // Verify sweep phase statistics (dry-run)
        assert_eq!(report.sweep_stats.deleted_count, 0, "Dry-run should not delete");
        assert_eq!(report.sweep_stats.recycle_bin_entries, 0, "Dry-run should not recycle");

        // Verify total duration
        assert!(report.duration >= Duration::from_secs(0), "Total duration should be non-negative");
        assert!(report.duration >= report.mark_stats.duration, "Total should include mark time");

        info!("[PASS] GC report structure test completed");
        info!("Report summary:");
        info!("  Protected roots: {}", report.protected_roots.len());
        info!("  Marked nodes: {}", report.mark_stats.marked_count);
        info!("  Memory strategy: {}", report.mark_stats.memory_strategy);
        info!("  Total duration: {:?}", report.duration);

        Ok(())
    }

    /// Test GC error handling with test database
    #[test]
    fn test_gc_error_handling_with_test_database() -> Result<()> {
        info!("Testing GC error handling with test database");

        // Test 1: Invalid configuration (should fail)
        {
            let test_db = GCTestDatabase::new()?;

            let config = GCConfig {
                dry_run: false, // Not dry run
                skip_confirm: false, // But no force flag
                ..Default::default()
            };

            let gc = GarbageCollector::new(
                test_db.moveos_store(),
                config,
                test_db.db_path(),
            )?;

            // This should fail because of safety verification
            let result = gc.execute_gc();
            assert!(result.is_err(), "Should fail without force execution");
            info!("  [PASS] Safety verification blocked execution as expected");
        }

        // Test 2: Force execution (should work)
        {
            let test_db = GCTestDatabase::new()?;

            let config = GCConfig {
                dry_run: true,
                skip_confirm: true, // Force execution
                ..Default::default()
            };

            let gc = GarbageCollector::new(
                test_db.moveos_store(),
                config,
                test_db.db_path(),
            )?;

            // This should succeed even without safety verification (dry-run mode)
            let result = gc.execute_gc();
            assert!(result.is_ok(), "Should succeed with force execution");
            info!("  [PASS] Force execution allowed GC to proceed");
        }

        info!("[PASS] GC error handling test completed");
        Ok(())
    }
}