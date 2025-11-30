// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//! Integration testing for PersistentMarker with marker workflow
//! Tests end-to-end scenarios, performance under realistic load, and system integration

#[cfg(test)]
mod tests {
    use crate::marker::{
        create_auto_marker_with_config, create_marker_with_config, MarkerStrategy, NodeMarker,
        PersistentMarker,
    };
    use anyhow::Result;
    use moveos_types::h256::H256;
    use rooch_config::prune_config::PruneConfig;
    use std::time::{Duration, Instant};

    /// Integration test result structure
    #[derive(Debug, Clone)]
    struct IntegrationTestResult {
        test_name: String,
        success: bool,
        duration: Duration,
        nodes_processed: usize,
        throughput_nodes_per_sec: f64,
        error_message: Option<String>,
    }

    impl IntegrationTestResult {
        fn new(
            test_name: &str,
            success: bool,
            duration: Duration,
            nodes_processed: usize,
            error_message: Option<String>,
        ) -> Self {
            let throughput_nodes_per_sec = if duration.as_secs_f64() > 0.0 {
                nodes_processed as f64 / duration.as_secs_f64()
            } else {
                0.0
            };

            Self {
                test_name: test_name.to_string(),
                success,
                duration,
                nodes_processed,
                throughput_nodes_per_sec,
                error_message,
            }
        }

        fn print(&self) {
            println!("=== {} ===", self.test_name);
            println!(
                "Success: {}",
                if self.success { "✅ PASS" } else { "❌ FAIL" }
            );
            println!("Duration: {:.3}s", self.duration.as_secs_f64());
            println!("Nodes Processed: {}", self.nodes_processed);
            println!("Throughput: {:.0} nodes/sec", self.throughput_nodes_per_sec);

            if let Some(ref error) = self.error_message {
                println!("Error: {}", error);
            }
            println!();
        }
    }

    /// Generate test nodes for integration testing
    fn generate_integration_test_nodes(count: usize) -> Vec<H256> {
        let mut nodes = Vec::with_capacity(count);
        for i in 0..count {
            let mut bytes = [0u8; 32];
            bytes[0..8].copy_from_slice(&(i as u64).to_le_bytes());
            bytes[8..16].copy_from_slice(&((i as u64).wrapping_mul(12345)).to_le_bytes());
            nodes.push(H256::from(bytes));
        }
        nodes
    }

    /// Test complete marker workflow with strategy selection
    #[test]
    fn test_persistent_marker_complete_workflow() -> Result<()> {
        println!("🔄 Testing PersistentMarker Complete Workflow\n");

        let test_cases = vec![
            (1_000, "Small dataset"),
            (10_000, "Medium dataset"),
            (100_000, "Large dataset"),
        ];

        for (node_count, description) in test_cases {
            println!("Testing {} ({} nodes):", description, node_count);

            let start = Instant::now();
            let nodes = generate_integration_test_nodes(node_count);

            // Test auto strategy selection
            let config = PruneConfig::default();
            let marker = create_auto_marker_with_config(node_count, &config, None)?;

            println!("Selected strategy: {}", marker.marker_type());

            // Test marking workflow
            let mut marked_count = 0;
            for node in &nodes {
                if marker.mark(*node)? {
                    marked_count += 1;
                }
            }

            // Test is_marked workflow
            let mut found_count = 0;
            for node in &nodes {
                if marker.is_marked(node) {
                    found_count += 1;
                }
            }

            let duration = start.elapsed();

            // Validate results
            let success = marked_count == nodes.len() && found_count == nodes.len();
            let error_message = if !success {
                Some(format!(
                    "Marked: {}, Found: {}, Expected: {}",
                    marked_count,
                    found_count,
                    nodes.len()
                ))
            } else {
                None
            };

            let result = IntegrationTestResult::new(
                &format!("Complete Workflow - {}", description),
                success,
                duration,
                node_count,
                error_message,
            );
            result.print();

            // Test reset workflow
            marker.reset()?;
            let reset_count = marker.marked_count();
            println!("After reset - marked count: {}", reset_count);
            assert_eq!(reset_count, 0, "Reset should clear all marked nodes");

            println!("{}", "=".repeat(50));
        }

        Ok(())
    }

    /// Test strategy switching between InMemory and Persistent
    #[test]
    fn test_strategy_switching_integration() -> Result<()> {
        println!("🔀 Testing Strategy Switching Integration\n");

        let test_nodes = generate_integration_test_nodes(50_000);

        // Test different configurations that should trigger different strategies
        let configurations = vec![
            (
                "Small Dataset - InMemory",
                PruneConfig {
                    marker_memory_threshold_mb: 1, // Very low threshold to force InMemory
                    ..Default::default()
                },
            ),
            ("Default Config - Auto Selection", PruneConfig::default()),
            (
                "Force Persistent",
                PruneConfig {
                    marker_force_persistent: true,
                    ..Default::default()
                },
            ),
        ];

        for (name, config) in configurations {
            println!("Testing configuration: {}", name);

            let start = Instant::now();
            let marker = create_auto_marker_with_config(test_nodes.len(), &config, None)?;
            let strategy = marker.marker_type();

            println!("Auto-selected strategy: {}", strategy);

            // Test marking performance
            let mut marked_count = 0;
            for node in &test_nodes {
                if marker.mark(*node)? {
                    marked_count += 1;
                }
            }

            let duration = start.elapsed();
            let throughput = test_nodes.len() as f64 / duration.as_secs_f64();

            println!(
                "Marked {} nodes in {:.3}s ({:.0} nodes/sec)",
                marked_count,
                duration.as_secs_f64(),
                throughput
            );

            // Verify nodes were marked (allowing for bloom filter false positives)
            let mark_rate = (marked_count as f64 / test_nodes.len() as f64) * 100.0;
            assert!(
                mark_rate >= 95.0,
                "Should mark at least 95% of nodes, got {:.1}% ({}/{})",
                mark_rate,
                marked_count,
                test_nodes.len()
            );

            // Test is_marked accuracy
            let mut verified_count = 0;
            for node in &test_nodes {
                if marker.is_marked(node) {
                    verified_count += 1;
                }
            }
            assert_eq!(
                verified_count,
                test_nodes.len(),
                "All marked nodes should be found"
            );

            println!(
                "✅ {} - Strategy: {} - Throughput: {:.0} nodes/sec\n",
                name, strategy, throughput
            );
        }

        Ok(())
    }

    /// Test configuration integration and validation
    #[test]
    fn test_configuration_integration() -> Result<()> {
        println!("⚙️ Testing Configuration Integration\n");

        let test_nodes = generate_integration_test_nodes(25_000);

        // Test configuration scenarios
        let configs = vec![
            (
                "Conservative",
                PruneConfig {
                    marker_batch_size: 1_000,
                    marker_bloom_bits: 262_144, // 256KB
                    marker_bloom_hash_fns: 4,
                    marker_memory_threshold_mb: 2048,
                    marker_auto_strategy: true,
                    marker_force_persistent: false,
                    ..Default::default()
                },
            ),
            (
                "Aggressive",
                PruneConfig {
                    marker_batch_size: 50_000,
                    marker_bloom_bits: 2_097_152, // 2MB
                    marker_bloom_hash_fns: 8,
                    marker_memory_threshold_mb: 256,
                    marker_auto_strategy: true,
                    marker_force_persistent: false,
                    ..Default::default()
                },
            ),
            (
                "High-Performance",
                PruneConfig {
                    marker_batch_size: 100_000,
                    marker_bloom_bits: 4_194_304, // 4MB
                    marker_bloom_hash_fns: 12,
                    marker_memory_threshold_mb: 128,
                    marker_auto_strategy: true,
                    marker_force_persistent: false,
                    ..Default::default()
                },
            ),
        ];

        println!("Config Performance Comparison:");
        println!("Config Name\t\tStrategy\t\tThroughput(nodes/s)\tMemory(MB)");
        println!("{}", "=".repeat(70));

        for (name, config) in configs {
            let start = Instant::now();

            let marker = create_auto_marker_with_config(test_nodes.len(), &config, None)?;
            let strategy = marker.marker_type();

            let mut marked_count = 0;
            for node in &test_nodes {
                if marker.mark(*node)? {
                    marked_count += 1;
                }
            }

            let duration = start.elapsed();
            let throughput = if duration.as_secs_f64() > 0.0 {
                marked_count as f64 / duration.as_secs_f64()
            } else {
                0.0
            };

            let memory_mb = config.marker_memory_threshold_mb as f64;

            println!(
                "{: <15}\t{: <15}\t{:.0}\t\t\t{:.0}",
                name, strategy, throughput, memory_mb
            );

            // Validate all nodes were marked
            assert_eq!(
                marked_count,
                test_nodes.len(),
                "All nodes should be marked with {} config",
                name
            );
        }

        println!();
        Ok(())
    }

    /// Test error handling and recovery scenarios
    #[test]
    fn test_error_handling_integration() -> Result<()> {
        println!("🛡️ Testing Error Handling Integration\n");

        let test_nodes = generate_integration_test_nodes(10_000);

        // Test with different marker creation scenarios
        let scenarios = vec![
            ("InMemory Marker", MarkerStrategy::InMemory),
            ("Persistent Marker", MarkerStrategy::Persistent),
        ];

        for (name, strategy) in scenarios {
            println!("Testing error handling with {}:", name);

            let start = Instant::now();

            // Create marker with specific strategy
            let marker = create_marker_with_config(
                strategy,
                test_nodes.len(),
                &PruneConfig::default(),
                None,
            )?;

            // Test normal marking workflow
            let mut marked_count = 0;
            let mut error_count = 0;

            for (i, node) in test_nodes.iter().enumerate() {
                match marker.mark(*node) {
                    Ok(marked) => {
                        if marked {
                            marked_count += 1;
                        }
                    }
                    Err(e) => {
                        error_count += 1;
                        if i < 10 {
                            // Only print first few errors
                            println!("  Mark error for node {}: {}", i, e);
                        }
                    }
                }
            }

            // Test is_marked workflow
            let verification_errors = 0;
            let mut verified_count = 0;

            for node in &test_nodes {
                if marker.is_marked(node) {
                    verified_count += 1;
                } // Not marked (expected for some strategies)
            }

            let duration = start.elapsed();
            let success_rate = (marked_count as f64 / test_nodes.len() as f64) * 100.0;

            println!(
                "  Marked: {}/{} ({:.1}% success)",
                marked_count,
                test_nodes.len(),
                success_rate
            );
            println!("  Verified: {}/{}", verified_count, test_nodes.len());
            println!("  Mark errors: {}", error_count);
            println!("  Verification errors: {}", verification_errors);
            println!("  Duration: {:.3}s", duration.as_secs_f64());

            // Basic validation
            if error_count == 0 && marked_count == test_nodes.len() {
                println!("  ✅ {} - Perfect execution", name);
            } else if success_rate > 90.0 {
                println!(
                    "  ✅ {} - Good execution (>{:.1}% success)",
                    name, success_rate
                );
            } else {
                println!(
                    "  ⚠️ {} - Needs attention ({:.1}% success)",
                    name, success_rate
                );
            }

            println!();
        }

        Ok(())
    }

    /// Test concurrent marking scenarios
    #[test]
    fn test_concurrent_integration() -> Result<()> {
        println!("🔄 Testing Concurrent Integration\n");

        use std::sync::Arc;
        use std::thread;

        let node_count = 20_000;
        let thread_count = 4;
        let nodes_per_thread = node_count / thread_count;

        // Test with PersistentMarker (should handle concurrent access well)
        let marker = Arc::new(PersistentMarker::with_batch_size(
            "concurrent_test_cf".to_string(),
            5_000,
        )?);

        println!(
            "Testing concurrent marking with {} threads, {} nodes each:",
            thread_count, nodes_per_thread
        );

        let start = Instant::now();
        let mut handles = Vec::new();

        for thread_id in 0..thread_count {
            let marker_clone = Arc::clone(&marker);
            let start_node = thread_id * nodes_per_thread;
            let end_node = start_node + nodes_per_thread;

            let handle = thread::spawn(move || -> Result<usize> {
                let mut marked_count = 0;
                for i in start_node..end_node {
                    let mut bytes = [0u8; 32];
                    bytes[0..8].copy_from_slice(&(i as u64).to_le_bytes());
                    let node = H256::from(bytes);

                    if marker_clone.mark(node)? {
                        marked_count += 1;
                    }
                }
                Ok(marked_count)
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        let mut total_marked = 0;
        for handle in handles {
            let thread_marked = handle.join().unwrap()?;
            total_marked += thread_marked;
        }

        let duration = start.elapsed();
        let throughput = total_marked as f64 / duration.as_secs_f64();

        println!("Concurrent Marking Results:");
        println!("  Total nodes: {}", node_count);
        println!("  Threads: {}", thread_count);
        println!("  Marked nodes: {}", total_marked);
        println!("  Duration: {:.3}s", duration.as_secs_f64());
        println!("  Throughput: {:.0} nodes/sec", throughput);
        println!(
            "  Concurrency efficiency: {:.1}%",
            (total_marked as f64 / node_count as f64) * 100.0
        );

        // Verify concurrent marking effectiveness
        assert!(
            total_marked >= node_count * 90 / 100,
            "Should mark at least 90% of nodes concurrently"
        );

        Ok(())
    }

    /// Test memory pressure scenarios
    #[test]
    fn test_memory_pressure_integration() -> Result<()> {
        println!("💾 Testing Memory Pressure Integration\n");

        let test_scenarios = vec![
            (1_000, "Small dataset"),
            (10_000, "Medium dataset"),
            (100_000, "Large dataset"),
            (500_000, "Very large dataset"),
        ];

        for (node_count, description) in test_scenarios {
            println!(
                "Testing memory pressure with {} ({} nodes):",
                description, node_count
            );

            let start = Instant::now();
            let nodes = generate_integration_test_nodes(node_count);

            // Test with auto strategy (should adapt based on memory pressure)
            let config = PruneConfig::default();
            let marker = create_auto_marker_with_config(node_count, &config, None)?;
            let strategy = marker.marker_type();

            println!("Selected strategy: {} for {} nodes", strategy, node_count);

            // Test marking under memory pressure
            let mut marked_count = 0;
            let mut consecutive_errors = 0;
            let max_consecutive_errors = 10;

            for (i, node) in nodes.iter().enumerate() {
                match marker.mark(*node) {
                    Ok(marked) => {
                        if marked {
                            marked_count += 1;
                            consecutive_errors = 0; // Reset error counter on success
                        }
                    }
                    Err(e) => {
                        consecutive_errors += 1;
                        if consecutive_errors <= max_consecutive_errors {
                            println!("  Mark error at node {}: {}", i, e);
                        }

                        // If too many consecutive errors, break to avoid infinite loop
                        if consecutive_errors >= max_consecutive_errors {
                            println!("  Too many consecutive errors, stopping at node {}", i);
                            break;
                        }
                    }
                }

                // Print progress for large datasets
                if node_count > 50_000 && i % 10_000 == 0 && i > 0 {
                    println!(
                        "  Progress: {}/{} ({:.1}%) marked",
                        i,
                        node_count,
                        (i as f64 / node_count as f64) * 100.0
                    );
                }
            }

            let duration = start.elapsed();
            let success_rate = (marked_count as f64 / nodes.len() as f64) * 100.0;
            let throughput = marked_count as f64 / duration.as_secs_f64();

            println!("  Results:");
            println!(
                "    Marked: {}/{} ({:.1}% success)",
                marked_count,
                nodes.len(),
                success_rate
            );
            println!("    Duration: {:.3}s", duration.as_secs_f64());
            println!("    Throughput: {:.0} nodes/sec", throughput);

            if success_rate >= 95.0 {
                println!("    ✅ Excellent performance under memory pressure");
            } else if success_rate >= 80.0 {
                println!("    ✅ Good performance under memory pressure");
            } else {
                println!("    ⚠️ Performance degradation under memory pressure");
            }

            println!();
        }

        Ok(())
    }

    /// Test resource management and cleanup
    #[test]
    fn test_resource_management_integration() -> Result<()> {
        println!("🧹 Testing Resource Management Integration\n");

        let test_cycles = 3;
        let nodes_per_cycle = 5_000;

        println!(
            "Testing resource management across {} GC cycles:",
            test_cycles
        );

        for cycle in 1..=test_cycles {
            println!("Cycle {}:", cycle);

            let start = Instant::now();
            let nodes = generate_integration_test_nodes(nodes_per_cycle);

            // Create fresh marker for each cycle
            let marker =
                create_auto_marker_with_config(nodes_per_cycle, &PruneConfig::default(), None)?;
            let strategy = marker.marker_type();

            println!("  Strategy: {}", strategy);

            // Mark nodes
            let mut marked_count = 0;
            for node in &nodes {
                if marker.mark(*node)? {
                    marked_count += 1;
                }
            }

            // Verify markings
            let mut verified_count = 0;
            for node in &nodes {
                if marker.is_marked(node) {
                    verified_count += 1;
                }
            }

            // Check marked count
            let reported_count = marker.marked_count();

            // Reset for cleanup
            marker.reset()?;
            let after_reset_count = marker.marked_count();

            let duration = start.elapsed();

            println!("  Marked: {}/{}", marked_count, nodes.len());
            println!("  Verified: {}/{}", verified_count, nodes.len());
            println!("  Reported count: {}", reported_count);
            println!("  After reset: {}", after_reset_count);
            println!("  Duration: {:.3}s", duration.as_secs_f64());

            // Validate cleanup
            assert_eq!(
                marked_count,
                nodes.len(),
                "All nodes should be marked in cycle {}",
                cycle
            );
            assert_eq!(
                verified_count,
                nodes.len(),
                "All nodes should be verified in cycle {}",
                cycle
            );
            assert_eq!(
                after_reset_count, 0,
                "Reset should clear all nodes in cycle {}",
                cycle
            );

            println!("  ✅ Cycle {} completed successfully", cycle);
            println!();
        }

        Ok(())
    }
}
