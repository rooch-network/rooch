// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//! Performance tests for the Garbage Collector
//!
//! This module tests the performance characteristics of the GC system

#[cfg(test)]
mod tests {
    use crate::config::GCConfig;
    use crate::garbage_collector::GarbageCollector;
    use crate::marker::{InMemoryMarker, MarkerStrategy, NodeMarker};
    use anyhow::Result;
    use moveos_types::h256::H256;
    use moveos_types::startup_info::StartupInfo;
    use rooch_config::RoochOpt;
    use rooch_db::RoochDB;
    use std::time::{Duration, Instant};
    use tempfile::TempDir;
    use tracing::info;

    /// Test memory usage with different node counts
    #[test]
    fn test_memory_scaling() -> Result<()> {
        let test_sizes = vec![1_000, 10_000, 100_000];

        for node_count in test_sizes {
            info!("Testing memory scaling with {} nodes", node_count);

            let marker = InMemoryMarker::new();
            let hashes: Vec<H256> = (0..node_count).map(|_| H256::random()).collect();

            let start_time = Instant::now();

            // Mark all hashes
            for hash in &hashes {
                marker.mark(*hash)?;
            }

            let duration = start_time.elapsed();
            let throughput = node_count as f64 / duration.as_secs_f64();

            // Performance assertions
            assert!(duration < Duration::from_secs(10));
            assert!(throughput > 1_000.0); // Should handle at least 1K ops/sec

            info!(
                "  Marked {} nodes in {:?} ({:.0} nodes/sec)",
                node_count, duration, throughput
            );
            info!("  Current marked count: {}", marker.marked_count());

            // Verify all marked
            assert_eq!(marker.marked_count(), node_count as u64);

            // Reset for next iteration
            marker.reset()?;
        }

        info!("✅ Memory scaling test completed successfully");
        Ok(())
    }

    /// Test different marker strategies performance
    #[test]
    fn test_marker_strategy_performance() -> Result<()> {
        let node_count = 50_000;
        let strategies = vec![
            MarkerStrategy::InMemory,
            // Note: Persistent strategy would require real database setup
        ];

        for strategy in strategies {
            info!("Testing {:?} strategy with {} nodes", strategy, node_count);

            let temp_dir = TempDir::new()?;
            let db_path = temp_dir.path().to_path_buf();

            let config = GCConfig {
                dry_run: true,
                marker_strategy: strategy,
                batch_size: 10_000,
                workers: 1,
                use_recycle_bin: true,
                force_compaction: false,
                skip_confirm: true,
                protected_roots_count: 1,
                ..GCConfig::default()
            };

            let rooch_opt = RoochOpt::new_with_default(Some(db_path.clone()), None, None)?;
            let rooch_db = RoochDB::init_with_mock_metrics_for_test(rooch_opt.store_config())?;

            // Create and save startup info for the test
            let test_state_root = H256::random();
            let startup_info = StartupInfo::new(test_state_root, 0);
            rooch_db
                .moveos_store
                .config_store
                .save_startup_info(startup_info)?;

            let gc = GarbageCollector::new(rooch_db, config)?;

            let start_time = Instant::now();
            let report = gc.execute_gc()?;
            let duration = start_time.elapsed();

            // Performance assertions
            assert!(duration < Duration::from_secs(30)); // Should complete within 30 seconds

            info!("  {:?} strategy completed in {:?}", strategy, duration);
            info!("  Marked {} nodes", report.mark_stats.marked_count);
            info!("  Memory strategy used: {:?}", report.memory_strategy_used);
            info!("  Mark phase duration: {:?}", report.mark_stats.duration);
        }

        info!("✅ Marker strategy performance test completed");
        Ok(())
    }

    /// Test GC performance with different batch sizes
    #[test]
    fn test_batch_size_performance() -> Result<()> {
        let node_count = 20_000;
        let batch_sizes = vec![100, 1_000, 10_000];

        for batch_size in batch_sizes {
            info!(
                "Testing batch size {} with {} nodes",
                batch_size, node_count
            );

            let temp_dir = TempDir::new()?;
            let db_path = temp_dir.path().to_path_buf();

            let config = GCConfig {
                dry_run: true,
                batch_size,
                workers: 1,
                use_recycle_bin: true,
                force_compaction: false,
                marker_strategy: MarkerStrategy::InMemory,
                skip_confirm: true,
                protected_roots_count: 1,
                ..GCConfig::default()
            };

            let rooch_opt = RoochOpt::new_with_default(Some(db_path.clone()), None, None)?;
            let rooch_db = RoochDB::init_with_mock_metrics_for_test(rooch_opt.store_config())?;

            // Create and save startup info for the test
            let test_state_root = H256::random();
            let startup_info = StartupInfo::new(test_state_root, 0);
            rooch_db
                .moveos_store
                .config_store
                .save_startup_info(startup_info)?;

            let gc = GarbageCollector::new(rooch_db, config)?;

            let start_time = Instant::now();
            let report = gc.execute_gc()?;
            let duration = start_time.elapsed();

            info!("  Batch size {} completed in {:?}", batch_size, duration);
            info!("  Marked {} nodes", report.mark_stats.marked_count);

            // All configurations should work
            assert!(duration < Duration::from_secs(20));
            assert!(report.mark_stats.marked_count > 0);
        }

        info!("✅ Batch size performance test completed");
        Ok(())
    }

    /// Test GC with different worker counts
    #[test]
    fn test_worker_count_scaling() -> Result<()> {
        let worker_counts = vec![1, 2, 4];

        for workers in worker_counts {
            info!("Testing with {} workers", workers);

            let temp_dir = TempDir::new()?;
            let db_path = temp_dir.path().to_path_buf();

            let config = GCConfig {
                dry_run: true,
                batch_size: 5_000,
                workers,
                use_recycle_bin: true,
                force_compaction: false,
                marker_strategy: MarkerStrategy::InMemory,
                skip_confirm: true,
                protected_roots_count: 1,
                ..GCConfig::default()
            };

            let rooch_opt = RoochOpt::new_with_default(Some(db_path.clone()), None, None)?;
            let rooch_db = RoochDB::init_with_mock_metrics_for_test(rooch_opt.store_config())?;

            // Create and save startup info for the test
            let test_state_root = H256::random();
            let startup_info = StartupInfo::new(test_state_root, 0);
            rooch_db
                .moveos_store
                .config_store
                .save_startup_info(startup_info)?;

            let gc = GarbageCollector::new(rooch_db, config)?;

            let start_time = Instant::now();
            let report = gc.execute_gc()?;
            let duration = start_time.elapsed();

            info!("  {} workers completed in {:?}", workers, duration);
            info!("  Marked {} nodes", report.mark_stats.marked_count);

            // All worker counts should work
            assert!(duration < Duration::from_secs(30));
        }

        info!("✅ Worker count scaling test completed");
        Ok(())
    }

    /// Stress test with high volume of operations
    #[test]
    fn test_high_volume_stress() -> Result<()> {
        let node_count = 200_000; // Large number for stress test

        info!("Starting stress test with {} nodes", node_count);

        let marker = InMemoryMarker::new();
        let hashes: Vec<H256> = (0..node_count).map(|_| H256::random()).collect();

        let start_time = Instant::now();
        let mut marked_count = 0;

        // Mark all hashes with progress tracking
        for (i, hash) in hashes.iter().enumerate() {
            if marker.mark(*hash)? {
                marked_count += 1;
            }

            // Progress reporting every 50K operations
            if i % 50_000 == 0 && i > 0 {
                let elapsed = start_time.elapsed();
                let current_throughput = i as f64 / elapsed.as_secs_f64();
                info!(
                    "  Progress: {}/{} nodes ({:.0} nodes/sec)",
                    i, node_count, current_throughput
                );
            }
        }

        let total_duration = start_time.elapsed();
        let overall_throughput = node_count as f64 / total_duration.as_secs_f64();

        // Performance assertions for stress test
        assert_eq!(marked_count, node_count);
        assert!(total_duration < Duration::from_secs(60)); // Should complete within 1 minute
        assert!(overall_throughput > 10_000.0); // Should handle at least 10K ops/sec

        info!("  Stress test completed:");
        info!("    Total nodes: {}", marked_count);
        info!("    Total duration: {:?}", total_duration);
        info!(
            "    Overall throughput: {:.0} nodes/sec",
            overall_throughput
        );
        info!("    Final marked count: {}", marker.marked_count());

        // Verify consistency
        let check_start = Instant::now();
        let mut found_count = 0;
        for hash in &hashes {
            if marker.is_marked(hash) {
                found_count += 1;
            }
        }
        let check_duration = check_start.elapsed();

        assert_eq!(found_count, node_count);
        assert!(check_duration < Duration::from_secs(10));

        info!(
            "    Verification: found {}/{} in {:?}",
            found_count, node_count, check_duration
        );

        info!("✅ High volume stress test completed successfully");
        Ok(())
    }

    /// Test memory efficiency and cleanup
    #[test]
    fn test_memory_efficiency() -> Result<()> {
        let node_count = 100_000;

        info!("Testing memory efficiency with {} nodes", node_count);

        let marker = InMemoryMarker::new();
        let hashes: Vec<H256> = (0..node_count).map(|_| H256::random()).collect();

        // Mark all hashes
        for hash in &hashes {
            marker.mark(*hash)?;
        }

        info!("  Memory efficiency test: marked {} nodes", node_count);

        // Verify all are marked
        assert_eq!(marker.marked_count(), node_count as u64);

        // Reset and check cleanup
        marker.reset()?;

        // Verify marker state is properly reset
        assert_eq!(marker.marked_count(), 0);

        // Verify that all hashes are no longer marked after reset
        for hash in hashes.iter().take(1000) {
            // Check a sample for efficiency
            assert!(!marker.is_marked(hash));
        }

        info!("✅ Memory efficiency test completed");
        info!("  ✅ Marker properly reset {} nodes", node_count);
        Ok(())
    }

    /// Test concurrent access to marker (if supported)
    #[test]
    fn test_concurrent_marking() -> Result<()> {
        use std::sync::Arc;
        use std::thread;

        let node_count = 50_000;
        let thread_count = 4;
        let nodes_per_thread = node_count / thread_count;

        info!("Testing concurrent marking with {} threads", thread_count);

        let marker = Arc::new(InMemoryMarker::new());
        let start_time = Instant::now();

        let mut handles = vec![];

        for thread_id in 0..thread_count {
            let marker_clone = Arc::clone(&marker);
            let handle = thread::spawn(move || -> Result<usize> {
                let mut marked = 0;
                let start_idx = thread_id * nodes_per_thread;

                for i in 0..nodes_per_thread {
                    let hash = H256::from_low_u64_be((start_idx + i) as u64);
                    if marker_clone.mark(hash).unwrap() {
                        marked += 1;
                    }
                }

                Ok(marked)
            });

            handles.push(handle);
        }

        let mut total_marked = 0;
        for handle in handles {
            total_marked += handle.join().unwrap()?;
        }

        let duration = start_time.elapsed();

        assert_eq!(total_marked, node_count);
        assert_eq!(marker.marked_count(), node_count as u64);
        assert!(duration < Duration::from_secs(30));

        info!("  Concurrent marking completed:");
        info!("    Threads: {}", thread_count);
        info!("    Total nodes: {}", total_marked);
        info!("    Duration: {:?}", duration);
        info!(
            "    Throughput: {:.0} nodes/sec",
            total_marked as f64 / duration.as_secs_f64()
        );

        info!("✅ Concurrent marking test completed");
        Ok(())
    }

    /// Test performance with different hash patterns
    #[test]
    fn test_hash_pattern_performance() -> Result<()> {
        let node_count = 20_000;

        let patterns = vec![
            (
                "Sequential",
                (0..node_count)
                    .map(|i| H256::from_low_u64_be(i as u64))
                    .collect::<Vec<_>>(),
            ),
            (
                "Random",
                (0..node_count).map(|_| H256::random()).collect::<Vec<_>>(),
            ),
            ("Mixed", {
                let mut hashes = Vec::new();
                for i in 0..node_count {
                    if i % 2 == 0 {
                        hashes.push(H256::from_low_u64_be(i as u64));
                    } else {
                        hashes.push(H256::random());
                    }
                }
                hashes
            }),
        ];

        for (pattern_name, hashes) in patterns {
            info!(
                "Testing {} pattern with {} hashes",
                pattern_name, node_count
            );

            let marker = InMemoryMarker::new();

            let start_time = Instant::now();
            for hash in &hashes {
                marker.mark(*hash)?;
            }
            let duration = start_time.elapsed();

            // Verification
            for hash in &hashes {
                assert!(marker.is_marked(hash));
            }

            let throughput = node_count as f64 / duration.as_secs_f64();

            info!(
                "  {} pattern: {:?} ({:.0} nodes/sec)",
                pattern_name, duration, throughput
            );
            assert!(throughput > 5_000.0); // Should handle at least 5K ops/sec
        }

        info!("✅ Hash pattern performance test completed");
        Ok(())
    }
}
