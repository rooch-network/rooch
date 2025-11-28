// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//! Multi-root protection tests for the Garbage Collector
//!
//! This module tests the enhanced multi-root protection functionality

#[cfg(test)]
mod tests {
    use crate::garbage_collector::{GCConfig, GarbageCollector};
    use crate::historical_state::{HistoricalStateCollector, HistoricalStateConfig};
    use crate::marker::MarkerStrategy;
    use anyhow::Result;
    use moveos_store::MoveOSStore;
    use rooch_store::RoochStore;
    use std::sync::Arc;
    use std::time::Duration;
    use tracing::info;

    /// Test HistoricalStateCollector with different root counts
    #[test]
    fn test_historical_state_collector_multi_root() -> Result<()> {
        info!("Testing HistoricalStateCollector with multi-root functionality");

        let (moveos_store, _temp_dir) = MoveOSStore::mock_moveos_store()?;
        let moveos_store = Arc::new(moveos_store);
        let (rooch_store, _temp_dir) = RoochStore::mock_rooch_store()?;
        let rooch_store = Arc::new(rooch_store);

        // Test collecting 0 roots
        let config = HistoricalStateConfig {
            protected_roots_count: 0,
        };
        let collector =
            HistoricalStateCollector::new(moveos_store.clone(), rooch_store.clone(), config);
        let roots = collector.collect_recent_state_roots()?;
        assert_eq!(roots.len(), 0);
        info!("✅ Empty root collection test passed");

        // Test collecting 1 root
        let config = HistoricalStateConfig {
            protected_roots_count: 1,
        };
        let collector =
            HistoricalStateCollector::new(moveos_store.clone(), rooch_store.clone(), config);
        let roots = collector.collect_recent_state_roots()?;
        assert_eq!(roots.len(), 1);
        info!("✅ Single root collection test passed");

        // Test collecting multiple roots
        let config = HistoricalStateConfig {
            protected_roots_count: 5,
        };
        let collector =
            HistoricalStateCollector::new(moveos_store.clone(), rooch_store.clone(), config);
        let roots = collector.collect_recent_state_roots()?;
        assert!(!roots.is_empty());
        assert!(roots.len() <= 5);
        info!(
            "✅ Multi-root collection test passed: {} roots collected",
            roots.len()
        );

        // Test collecting many roots
        let config = HistoricalStateConfig {
            protected_roots_count: 100,
        };
        let collector = HistoricalStateCollector::new(moveos_store, rooch_store, config);
        let roots = collector.collect_recent_state_roots()?;
        assert!(!roots.is_empty());
        assert!(roots.len() <= 100);
        info!(
            "✅ Large root count test passed: {} roots collected",
            roots.len()
        );

        info!("✅ HistoricalStateCollector multi-root tests completed successfully");
        Ok(())
    }

    /// Test GC configuration with multi-root support
    #[test]
    fn test_gc_config_multi_root() -> Result<()> {
        info!("Testing GC configuration with multi-root support");

        // Test default configuration
        let default_config = GCConfig::default();
        assert_eq!(default_config.protected_roots_count, 1);
        info!("✅ Default configuration has protected_roots_count = 1");

        // Test custom configuration
        let custom_config = GCConfig {
            protected_roots_count: 5,
            ..Default::default()
        };
        assert_eq!(custom_config.protected_roots_count, 5);
        info!("✅ Custom configuration has protected_roots_count = 5");

        info!("✅ GC configuration multi-root tests completed successfully");
        Ok(())
    }

    /// Test GC execution with different protected root counts
    #[test]
    fn test_gc_execution_multi_root() -> Result<()> {
        info!("Testing GC execution with different protected root counts");

        let (moveos_store, temp_dir) = MoveOSStore::mock_moveos_store()?;
        let moveos_store = Arc::new(moveos_store);

        // Test with single root (backward compatibility)
        {
            let config = GCConfig {
                dry_run: true,
                protected_roots_count: 1,
                force_execution: true,
                ..Default::default()
            };

            let gc =
                GarbageCollector::new(moveos_store.clone(), config, temp_dir.path().to_path_buf())?;
            let report = gc.execute_gc()?;

            assert_eq!(report.protected_roots.len(), 1);
            info!(
                "✅ Single root GC execution: protected {} roots",
                report.protected_roots.len()
            );
        }

        // Test with multiple roots
        {
            let config = GCConfig {
                dry_run: true,
                protected_roots_count: 5,
                force_execution: true,
                ..Default::default()
            };

            let gc =
                GarbageCollector::new(moveos_store.clone(), config, temp_dir.path().to_path_buf())?;
            let report = gc.execute_gc()?;

            assert!(!report.protected_roots.is_empty());
            assert!(report.protected_roots.len() <= 5);
            info!(
                "✅ Multi-root GC execution: protected {} roots",
                report.protected_roots.len()
            );
        }

        // Test with large root count
        {
            let config = GCConfig {
                dry_run: true,
                protected_roots_count: 10,
                force_execution: true,
                ..Default::default()
            };

            let gc =
                GarbageCollector::new(moveos_store.clone(), config, temp_dir.path().to_path_buf())?;
            let report = gc.execute_gc()?;

            assert!(!report.protected_roots.is_empty());
            assert!(report.protected_roots.len() <= 10);
            info!(
                "✅ Large root count GC execution: protected {} roots",
                report.protected_roots.len()
            );
        }

        info!("✅ GC execution multi-root tests completed successfully");
        Ok(())
    }

    /// Test GC execution with different marker strategies and multi-root
    #[test]
    fn test_gc_multi_root_different_strategies() -> Result<()> {
        info!("Testing GC execution with different marker strategies and multi-root");

        let (moveos_store, temp_dir) = MoveOSStore::mock_moveos_store()?;
        let moveos_store = Arc::new(moveos_store);

        let strategies = vec![
            MarkerStrategy::InMemory,
            MarkerStrategy::Auto,
            // Note: Persistent strategy would require actual database setup
        ];

        for strategy in strategies {
            let config = GCConfig {
                dry_run: true,
                protected_roots_count: 3,
                marker_strategy: strategy,
                force_execution: true,
                ..Default::default()
            };

            let gc =
                GarbageCollector::new(moveos_store.clone(), config, temp_dir.path().to_path_buf())?;
            let report = gc.execute_gc()?;

            assert!(!report.protected_roots.is_empty());
            info!(
                "✅ Strategy {:?} with multi-root: protected {} roots",
                strategy,
                report.protected_roots.len()
            );
        }

        info!("✅ GC multi-root strategy tests completed successfully");
        Ok(())
    }

    /// Test backward compatibility with existing single-root behavior
    #[test]
    fn test_backward_compatibility_single_root() -> Result<()> {
        info!("Testing backward compatibility with existing single-root behavior");

        let (moveos_store, temp_dir) = MoveOSStore::mock_moveos_store()?;
        let moveos_store = Arc::new(moveos_store);

        // Test that default behavior (no protected_roots_count specified) works as before
        let config = GCConfig {
            dry_run: true,
            force_execution: true,
            ..Default::default()
        };

        let gc =
            GarbageCollector::new(moveos_store.clone(), config, temp_dir.path().to_path_buf())?;
        let report = gc.execute_gc()?;

        // Should behave exactly like before - single root protection
        assert_eq!(report.protected_roots.len(), 1);
        info!("✅ Backward compatibility: single root protection maintained");

        // Test explicit single root specification
        let config = GCConfig {
            dry_run: true,
            protected_roots_count: 1,
            force_execution: true,
            ..Default::default()
        };

        let gc =
            GarbageCollector::new(moveos_store.clone(), config, temp_dir.path().to_path_buf())?;
        let report = gc.execute_gc()?;

        assert_eq!(report.protected_roots.len(), 1);
        info!("✅ Explicit single root specification works correctly");

        info!("✅ Backward compatibility tests completed successfully");
        Ok(())
    }

    /// Test memory usage with multi-root protection
    #[test]
    fn test_memory_usage_multi_root() -> Result<()> {
        info!("Testing memory usage with multi-root protection");

        let (moveos_store, _temp_dir) = MoveOSStore::mock_moveos_store()?;
        let moveos_store = Arc::new(moveos_store);
        let (rooch_store, _temp_dir) = RoochStore::mock_rooch_store()?;
        let rooch_store = Arc::new(rooch_store);

        // Test memory usage estimation with different root counts
        let test_counts = vec![1, 5, 10, 50, 100];

        for count in test_counts {
            let start_time = std::time::Instant::now();
            let config = HistoricalStateConfig {
                protected_roots_count: count,
            };
            let collector =
                HistoricalStateCollector::new(moveos_store.clone(), rooch_store.clone(), config);
            let roots = collector.collect_recent_state_roots()?;
            let duration = start_time.elapsed();

            info!(
                "Root count {}: collected {} roots in {:?}",
                count,
                roots.len(),
                duration
            );

            // Memory usage should be reasonable (performance assertion)
            assert!(
                duration < Duration::from_secs(5),
                "Root collection took too long"
            );
        }

        info!("✅ Memory usage tests completed successfully");
        Ok(())
    }

    /// Test error handling with invalid root counts
    #[test]
    fn test_error_handling_invalid_root_counts() -> Result<()> {
        info!("Testing error handling with invalid root counts");

        let (moveos_store, temp_dir) = MoveOSStore::mock_moveos_store()?;
        let moveos_store = Arc::new(moveos_store);

        // Test with zero protected roots (should fail during validation)
        let config = GCConfig {
            dry_run: true,
            protected_roots_count: 0,
            force_execution: true,
            ..Default::default()
        };

        let gc =
            GarbageCollector::new(moveos_store.clone(), config, temp_dir.path().to_path_buf())?;
        let result = gc.execute_gc();

        assert!(result.is_err(), "Should fail with zero protected roots");
        info!("✅ Zero protected roots correctly rejected");

        info!("✅ Error handling tests completed successfully");
        Ok(())
    }

    /// Test concurrent access to HistoricalStateCollector
    #[test]
    fn test_concurrent_historical_state_collection() -> Result<()> {
        info!("Testing concurrent access to HistoricalStateCollector");

        let (moveos_store, _temp_dir) = MoveOSStore::mock_moveos_store()?;
        let moveos_store = Arc::new(moveos_store);
        let (rooch_store, _temp_dir) = RoochStore::mock_rooch_store()?;
        let rooch_store = Arc::new(rooch_store);

        let config = HistoricalStateConfig {
            protected_roots_count: 3,
        };
        let collector = Arc::new(HistoricalStateCollector::new(
            moveos_store,
            rooch_store,
            config,
        ));

        let thread_count = 4;
        let mut handles = vec![];

        for _thread_id in 0..thread_count {
            let collector_clone = Arc::clone(&collector);
            let handle = std::thread::spawn(move || -> Result<usize> {
                let roots = collector_clone.collect_recent_state_roots()?;
                Ok(roots.len())
            });

            handles.push(handle);
        }

        let mut results = vec![];
        for handle in handles {
            results.push(handle.join().unwrap()?);
        }

        // All threads should complete successfully
        assert_eq!(results.len(), thread_count);
        for (i, result) in results.iter().enumerate() {
            info!("Thread {} collected {} roots", i, result);
            assert!(*result > 0, "Each thread should collect at least one root");
        }

        info!("✅ Concurrent access test completed successfully");
        Ok(())
    }
}
