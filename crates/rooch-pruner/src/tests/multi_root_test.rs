// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//! Multi-root protection tests for the Garbage Collector
//!
//! This module tests the enhanced multi-root protection functionality

#[cfg(test)]
mod tests {
    use crate::config::GCConfig;
    use crate::garbage_collector::GarbageCollector;
    use crate::marker::MarkerStrategy;
    use anyhow::Result;
    use moveos_store::MoveOSStore;
    use std::sync::Arc;
    use tracing::info;

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
                skip_confirm: true,
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
                skip_confirm: true,
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
                skip_confirm: true,
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
                skip_confirm: true,
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
            skip_confirm: true,
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
            skip_confirm: true,
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
            skip_confirm: true,
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
}
