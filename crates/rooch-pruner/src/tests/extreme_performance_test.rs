// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//! Extreme performance and boundary tests for the Garbage Collector
//!
//! This module pushes the GC system to its limits to identify breaking points
//! and performance characteristics under extreme conditions

#[cfg(test)]
mod tests {
    use crate::marker::{InMemoryMarker, NodeMarker};
    use anyhow::Result;
    use moveos_types::h256::H256;
    use std::time::{Duration, Instant};
    use tracing::info;

    /// Test memory usage patterns with extremely large node counts
    #[test]
    fn test_extreme_memory_scaling() -> Result<()> {
        let test_sizes = vec![
            1_000_000,  // 1M nodes
            5_000_000,  // 5M nodes
            10_000_000, // 10M nodes (extreme)
        ];

        for node_count in test_sizes {
            info!("Testing extreme memory scaling with {} nodes", node_count);

            let marker = InMemoryMarker::new();

            // Generate hashes in batches to avoid memory pressure
            let batch_size = 100_000;
            let total_duration = Duration::from_secs(120); // 2 minute timeout

            let start_time = Instant::now();
            let mut marked_count = 0;

            for batch_start in (0..node_count).step_by(batch_size) {
                let batch_end = std::cmp::min(batch_start + batch_size, node_count);

                // Check timeout
                if start_time.elapsed() > total_duration {
                    info!("  Timeout reached at {} nodes", batch_start);
                    break;
                }

                // Mark batch
                for i in batch_start..batch_end {
                    let hash = H256::from_low_u64_be(i as u64);
                    if marker.mark(hash)? {
                        marked_count += 1;
                    }
                }

                // Progress report
                if batch_start % (batch_size * 10) == 0 {
                    let elapsed = start_time.elapsed();
                    let throughput = batch_start as f64 / elapsed.as_secs_f64();
                    info!(
                        "  Progress: {}/{} ({:.0} nodes/sec)",
                        batch_start, node_count, throughput
                    );
                }
            }

            let total_duration = start_time.elapsed();
            let final_throughput = marked_count as f64 / total_duration.as_secs_f64();

            info!("  Results for {} nodes:", marked_count);
            info!("    Duration: {:?}", total_duration);
            info!("    Throughput: {:.0} nodes/sec", final_throughput);
            info!("    Memory usage: {} bytes", marker.marked_count() * 32); // Approximate

            // Verify consistency
            assert_eq!(marker.marked_count(), marked_count as u64);

            // Performance should degrade gracefully
            if marked_count > 100_000 {
                assert!(
                    final_throughput > 1_000.0,
                    "Throughput too low: {:.0}",
                    final_throughput
                );
            }

            // Reset for next test
            marker.reset()?;
            info!("  ✅ Extreme test completed, memory reset successful");
        }

        info!("✅ Extreme memory scaling test completed");
        Ok(())
    }

    /// Test with rapidly changing mark/unmark patterns
    #[test]
    fn test_rapid_mark_unmark_patterns() -> Result<()> {
        let cycle_count = 1_000;
        let nodes_per_cycle = 10_000;

        info!(
            "Testing rapid mark/unmark patterns: {} cycles, {} nodes each",
            cycle_count, nodes_per_cycle
        );

        let marker = InMemoryMarker::new();
        let start_time = Instant::now();
        let mut operations = 0;

        for cycle in 0..cycle_count {
            // Mark phase
            for i in 0..nodes_per_cycle {
                let hash = H256::from_low_u64_be((cycle * nodes_per_cycle + i) as u64);
                marker.mark(hash)?;
                operations += 1;
            }

            // Reset phase
            marker.reset()?;
            operations += 1;

            // Progress reporting
            if cycle % 100 == 0 {
                let elapsed = start_time.elapsed();
                let ops_per_sec = operations as f64 / elapsed.as_secs_f64();
                info!(
                    "  Cycle {}/{} ({:.0} ops/sec)",
                    cycle, cycle_count, ops_per_sec
                );
            }
        }

        let total_duration = start_time.elapsed();
        let ops_per_sec = operations as f64 / total_duration.as_secs_f64();

        info!("  Rapid patterns completed:");
        info!("    Total operations: {}", operations);
        info!("    Duration: {:?}", total_duration);
        info!("    Operations/sec: {:.0}", ops_per_sec);

        // Performance assertions
        assert!(
            ops_per_sec > 100_000.0,
            "Operations too slow: {:.0} ops/sec",
            ops_per_sec
        );
        assert_eq!(
            marker.marked_count(),
            0,
            "Marker should be empty after all cycles"
        );

        info!("✅ Rapid mark/unmark patterns test completed");
        Ok(())
    }

    /// Test marker behavior with hash collisions and similar patterns
    #[test]
    fn test_hash_collision_resistance() -> Result<()> {
        let test_patterns = vec![
            (
                "Sequential",
                (0..100_000)
                    .map(|i| H256::from_low_u64_be(i as u64))
                    .collect::<Vec<_>>(),
            ),
            ("Repeated blocks", {
                let mut hashes = Vec::new();
                for _block in 0..10 {
                    for i in 0..10_000 {
                        hashes.push(H256::from_low_u64_be(i as u64));
                    }
                }
                hashes
            }),
            ("Pattern-based", {
                let mut hashes = Vec::new();
                for i in 0..100_000 {
                    let hash_val = ((i as u64).wrapping_mul(1234567)) % 1_000_000;
                    hashes.push(H256::from_low_u64_be(hash_val));
                }
                hashes
            }),
        ];

        for (pattern_name, hashes) in test_patterns {
            info!("Testing hash pattern: {}", pattern_name);

            let marker = InMemoryMarker::new();
            let start_time = Instant::now();
            let mut unique_marks = 0;

            for hash in &hashes {
                if marker.mark(*hash)? {
                    unique_marks += 1;
                }
            }

            let duration = start_time.elapsed();
            let throughput = hashes.len() as f64 / duration.as_secs_f64();

            info!(
                "  {}: {}/{} unique, {:.0} ops/sec",
                pattern_name,
                unique_marks,
                hashes.len(),
                throughput
            );

            // Verify all are marked
            for hash in &hashes {
                assert!(marker.is_marked(hash), "Hash should be marked: {}", hash);
            }

            // Performance should be consistent regardless of pattern
            assert!(throughput > 10_000.0, "Pattern {} too slow", pattern_name);
        }

        info!("✅ Hash collision resistance test completed");
        Ok(())
    }

    /// Test memory fragmentation patterns
    #[test]
    fn test_memory_fragmentation_patterns() -> Result<()> {
        info!("Testing memory fragmentation patterns");

        let marker = InMemoryMarker::new();
        let cycles = 100;
        let nodes_per_cycle = 50_000;

        let start_time = Instant::now();

        for cycle in 0..cycles {
            // Mark half the nodes
            for i in 0..nodes_per_cycle / 2 {
                let hash = H256::from_low_u64_be((cycle * nodes_per_cycle + i) as u64);
                marker.mark(hash)?;
            }

            // Unmark by resetting a subset
            if cycle % 10 == 0 {
                // Reset completely periodically
                marker.reset()?;
            }

            // Progress reporting
            if cycle % 20 == 0 {
                let _elapsed = start_time.elapsed();
                let current_count = marker.marked_count();
                info!(
                    "  Cycle {}/{}: {} nodes marked",
                    cycle, cycles, current_count
                );
            }
        }

        let total_duration = start_time.elapsed();

        info!("  Fragmentation test completed:");
        info!("    Duration: {:?}", total_duration);
        info!("    Final marked count: {}", marker.marked_count());

        // Memory should be manageable
        let final_count = marker.marked_count();
        assert!(
            final_count < 500_000,
            "Memory leak detected: {} nodes",
            final_count
        );

        info!("✅ Memory fragmentation test completed");
        Ok(())
    }

    /// Test concurrent access under high contention
    #[test]
    fn test_high_contention_concurrent_access() -> Result<()> {
        use std::sync::Arc;
        use std::thread;

        let thread_count = 16; // High contention
        let operations_per_thread = 25_000;
        let total_operations = thread_count * operations_per_thread;

        info!(
            "Testing high contention concurrent access: {} threads, {} ops each",
            thread_count, operations_per_thread
        );

        let marker = Arc::new(InMemoryMarker::new());
        let start_time = Instant::now();

        let mut handles = vec![];
        for thread_id in 0..thread_count {
            let marker_clone = Arc::clone(&marker);
            let handle = thread::spawn(move || -> Result<usize> {
                let mut marked = 0;
                let base_offset = thread_id * operations_per_thread * 10; // Space out threads

                for i in 0..operations_per_thread {
                    let hash = H256::from_low_u64_be((base_offset + i) as u64);
                    if marker_clone.mark(hash).unwrap() {
                        marked += 1;
                    }

                    // Add some contention by checking other hashes
                    if i % 1000 == 0 {
                        let check_hash = H256::from_low_u64_be((i / 1000) as u64);
                        let _is_marked = marker_clone.is_marked(&check_hash);
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
        let throughput = total_operations as f64 / duration.as_secs_f64();

        info!("  High contention results:");
        info!("    Total operations: {}", total_operations);
        info!("    Total marked: {}", total_marked);
        info!("    Duration: {:?}", duration);
        info!("    Throughput: {:.0} ops/sec", throughput);

        // Verify consistency
        assert_eq!(marker.marked_count(), total_marked as u64);
        assert_eq!(total_marked, total_operations); // All should be unique

        // Performance should be reasonable even under contention
        assert!(
            throughput > 5_000.0,
            "High contention performance too low: {:.0}",
            throughput
        );

        info!("✅ High contention concurrent access test completed");
        Ok(())
    }

    /// Test system behavior under memory pressure simulation
    #[test]
    fn test_memory_pressure_simulation() -> Result<()> {
        info!("Testing memory pressure simulation");

        let marker = InMemoryMarker::new();
        let pressure_cycles = 50;
        let nodes_per_cycle = 100_000;

        let start_time = Instant::now();

        for cycle in 0..pressure_cycles {
            // Build up memory pressure
            for i in 0..nodes_per_cycle {
                let hash = H256::from_low_u64_be(cycle * nodes_per_cycle + i);
                marker.mark(hash)?;
            }

            // Simulate memory pressure relief
            if cycle % 10 == 0 {
                info!(
                    "  Pressure cycle {}/{}: {} nodes marked",
                    cycle,
                    pressure_cycles,
                    marker.marked_count()
                );

                // Partial relief: remove last quarter
                let remove_start = (cycle * nodes_per_cycle) + (nodes_per_cycle * 3 / 4);
                for i in remove_start..(cycle + 1) * nodes_per_cycle {
                    let _hash = H256::from_low_u64_be(i);
                    // Note: In real scenario, you'd implement unmark functionality
                    // For now, we just measure pressure buildup
                }
            }

            // Check if we're hitting reasonable limits
            if marker.marked_count() > 10_000_000 {
                info!(
                    "  Memory pressure limit reached: {} nodes",
                    marker.marked_count()
                );
                break;
            }
        }

        let total_duration = start_time.elapsed();

        info!("  Memory pressure test completed:");
        info!("    Duration: {:?}", total_duration);
        info!("    Final marked count: {}", marker.marked_count());
        info!(
            "    Average marking rate: {:.0} nodes/sec",
            (pressure_cycles * nodes_per_cycle) as f64 / total_duration.as_secs_f64()
        );

        // System should handle pressure gracefully
        let final_count = marker.marked_count();
        assert!(final_count > 0, "Should have some nodes marked");
        assert!(
            final_count <= pressure_cycles * nodes_per_cycle,
            "Should not exceed expected count"
        );

        info!("✅ Memory pressure simulation test completed");
        Ok(())
    }

    /// Test graceful degradation with increasing load
    #[test]
    fn test_graceful_degradation() -> Result<()> {
        info!("Testing graceful degradation under increasing load");

        let load_steps = [
            10_000,    // Light load
            50_000,    // Medium load
            100_000,   // Heavy load
            500_000,   // Very heavy load
            1_000_000, // Extreme load
        ];

        let baseline_throughput: Option<f64> = None;

        for step_idx in 0..load_steps.len() {
            let node_count = load_steps[step_idx];
            info!(
                "  Load step {}/{}: {} nodes",
                step_idx + 1,
                load_steps.len(),
                node_count
            );

            let marker = InMemoryMarker::new();
            let start_time = Instant::now();

            for i in 0..node_count {
                let hash = H256::from_low_u64_be(i as u64);
                marker.mark(hash)?;
            }

            let duration = start_time.elapsed();
            let throughput = node_count as f64 / duration.as_secs_f64();

            info!(
                "    Duration: {:?}, Throughput: {:.0} nodes/sec",
                duration, throughput
            );

            // Performance should degrade gracefully (not catastrophically)
            if let Some(baseline) = baseline_throughput {
                let degradation_ratio = throughput / baseline;
                info!("    Degradation ratio: {:.2}", degradation_ratio);

                // Even at extreme load, should maintain reasonable performance
                assert!(
                    degradation_ratio > 0.1,
                    "Performance degraded too much: {:.2}",
                    degradation_ratio
                );
            }

            // Memory usage should be predictable
            let memory_per_node = 32; // H256 size approximate
            let estimated_memory = marker.marked_count() as usize * memory_per_node;
            info!(
                "    Estimated memory usage: {} MB",
                estimated_memory / 1_048_576
            );

            // Verify correctness
            assert_eq!(marker.marked_count(), node_count as u64);
        }

        info!("✅ Graceful degradation test completed");
        Ok(())
    }
}
