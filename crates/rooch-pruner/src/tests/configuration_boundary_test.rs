// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//! Configuration Boundary Testing for PersistentMarker
//! Tests extreme parameter values, edge cases, and configuration validation

#[cfg(test)]
mod tests {
    use crate::marker::{create_auto_marker_with_config, create_marker_with_config};
    use anyhow::Result;
    use moveos_types::h256::H256;
    use rooch_config::prune_config::PruneConfig;
    use std::time::Instant;

    /// Configuration boundary test results
    #[derive(Debug, Clone)]
    struct BoundaryTestResult {
        test_name: String,
        config_param: String,
        test_value: String,
        success: bool,
        error_message: Option<String>,
        throughput_nodes_per_sec: Option<f64>,
    }

    impl BoundaryTestResult {
        fn new(
            test_name: &str,
            config_param: &str,
            test_value: &str,
            success: bool,
            error_message: Option<String>,
        ) -> Self {
            Self {
                test_name: test_name.to_string(),
                config_param: config_param.to_string(),
                test_value: test_value.to_string(),
                success,
                error_message,
                throughput_nodes_per_sec: None,
            }
        }

        fn with_performance(mut self, throughput_nodes_per_sec: f64) -> Self {
            self.throughput_nodes_per_sec = Some(throughput_nodes_per_sec);
            self
        }

        fn print(&self) {
            println!("=== {} ===", self.test_name);
            println!("Parameter: {} = {}", self.config_param, self.test_value);
            println!(
                "Success: {}",
                if self.success { "✅ PASS" } else { "❌ FAIL" }
            );

            if let Some(ref error) = self.error_message {
                println!("Error: {}", error);
            }

            if let Some(throughput) = self.throughput_nodes_per_sec {
                println!("Throughput: {:.0} nodes/sec", throughput);
            }
            println!();
        }
    }

    /// Test configuration boundary for marker batch size
    fn test_batch_size_boundary(batch_size: usize) -> Result<BoundaryTestResult> {
        let config = PruneConfig {
            marker_batch_size: batch_size,
            ..Default::default()
        };

        let test_nodes = vec![H256::random(); 1000];

        let result = create_marker_with_config(
            crate::marker::MarkerStrategy::Persistent,
            1000,
            &config,
            None,
        );

        match result {
            Ok(marker) => {
                // Test marking performance
                let mark_start = Instant::now();
                let mut marked_count = 0;
                for node in &test_nodes {
                    if marker.mark(*node).unwrap() {
                        marked_count += 1;
                    }
                }
                let mark_duration = mark_start.elapsed();

                let throughput = if mark_duration.as_secs_f64() > 0.0 {
                    marked_count as f64 / mark_duration.as_secs_f64()
                } else {
                    0.0
                };

                Ok(BoundaryTestResult::new(
                    "Batch Size Boundary Test",
                    "marker_batch_size",
                    &batch_size.to_string(),
                    true,
                    None,
                )
                .with_performance(throughput))
            }
            Err(e) => Ok(BoundaryTestResult::new(
                "Batch Size Boundary Test",
                "marker_batch_size",
                &batch_size.to_string(),
                false,
                Some(e.to_string()),
            )),
        }
    }

    /// Test configuration boundary for bloom filter size
    fn test_bloom_bits_boundary(bloom_bits: usize) -> Result<BoundaryTestResult> {
        let config = PruneConfig {
            marker_bloom_bits: bloom_bits,
            ..Default::default()
        };

        let test_nodes = vec![H256::random(); 500];

        let result = create_marker_with_config(
            crate::marker::MarkerStrategy::Persistent,
            500,
            &config,
            None,
        );

        match result {
            Ok(marker) => {
                let mark_start = Instant::now();
                let mut marked_count = 0;
                for node in &test_nodes {
                    if marker.mark(*node).unwrap() {
                        marked_count += 1;
                    }
                }
                let mark_duration = mark_start.elapsed();

                let throughput = if mark_duration.as_secs_f64() > 0.0 {
                    marked_count as f64 / mark_duration.as_secs_f64()
                } else {
                    0.0
                };

                Ok(BoundaryTestResult::new(
                    "Bloom Filter Bits Boundary Test",
                    "marker_bloom_bits",
                    &bloom_bits.to_string(),
                    true,
                    None,
                )
                .with_performance(throughput))
            }
            Err(e) => Ok(BoundaryTestResult::new(
                "Bloom Filter Bits Boundary Test",
                "marker_bloom_bits",
                &bloom_bits.to_string(),
                false,
                Some(e.to_string()),
            )),
        }
    }

    /// Test configuration boundary for bloom filter hash functions
    fn test_bloom_hash_fns_boundary(hash_fns: u8) -> Result<BoundaryTestResult> {
        let config = PruneConfig {
            marker_bloom_hash_fns: hash_fns,
            ..Default::default()
        };

        let test_nodes = vec![H256::random(); 300];

        let result = create_marker_with_config(
            crate::marker::MarkerStrategy::Persistent,
            300,
            &config,
            None,
        );

        match result {
            Ok(marker) => {
                let mark_start = Instant::now();
                let mut marked_count = 0;
                for node in &test_nodes {
                    if marker.mark(*node).unwrap() {
                        marked_count += 1;
                    }
                }
                let mark_duration = mark_start.elapsed();

                let throughput = if mark_duration.as_secs_f64() > 0.0 {
                    marked_count as f64 / mark_duration.as_secs_f64()
                } else {
                    0.0
                };

                Ok(BoundaryTestResult::new(
                    "Bloom Hash Functions Boundary Test",
                    "marker_bloom_hash_fns",
                    &hash_fns.to_string(),
                    true,
                    None,
                )
                .with_performance(throughput))
            }
            Err(e) => Ok(BoundaryTestResult::new(
                "Bloom Hash Functions Boundary Test",
                "marker_bloom_hash_fns",
                &hash_fns.to_string(),
                false,
                Some(e.to_string()),
            )),
        }
    }

    /// Test configuration boundary for memory threshold
    fn test_memory_threshold_boundary(memory_threshold_mb: usize) -> Result<BoundaryTestResult> {
        let config = PruneConfig {
            marker_memory_threshold_mb: memory_threshold_mb,
            ..Default::default()
        };

        // Test with different dataset sizes - just test the first case
        let (node_count, description) = (1000, "Small dataset");
        let result = create_auto_marker_with_config(node_count, &config, None);

        match result {
            Ok(marker) => {
                let strategy_type = marker.marker_type();
                let test_nodes = vec![H256::random(); 100.min(node_count)];
                let mark_start = Instant::now();
                for node in &test_nodes {
                    marker.mark(*node).unwrap();
                }
                let mark_duration = mark_start.elapsed();
                let throughput = if mark_duration.as_secs_f64() > 0.0 {
                    test_nodes.len() as f64 / mark_duration.as_secs_f64()
                } else {
                    0.0
                };

                Ok(BoundaryTestResult::new(
                    "Memory Threshold Boundary Test",
                    "marker_memory_threshold_mb",
                    &format!(
                        "{} ({} -> {})",
                        memory_threshold_mb, description, strategy_type
                    ),
                    true,
                    None,
                )
                .with_performance(throughput))
            }
            Err(e) => Ok(BoundaryTestResult::new(
                "Memory Threshold Boundary Test",
                "marker_memory_threshold_mb",
                &format!("{} ({})", memory_threshold_mb, description),
                false,
                Some(e.to_string()),
            )),
        }
    }

    /// Test invalid configuration scenarios
    fn test_invalid_configurations() -> Result<Vec<BoundaryTestResult>> {
        let mut results = Vec::new();

        // Test 1: Zero batch size
        let zero_batch_config = PruneConfig {
            marker_batch_size: 0,
            ..Default::default()
        };
        let result = create_auto_marker_with_config(1000, &zero_batch_config, None);
        results.push(BoundaryTestResult::new(
            "Invalid Config: Zero Batch Size",
            "marker_batch_size",
            "0",
            result.is_ok(),
            if result.is_err() {
                Some(format!("Error: {:?}", result.err()))
            } else {
                None
            },
        ));

        // Test 2: Zero bloom filter bits
        let zero_bloom_config = PruneConfig {
            marker_bloom_bits: 0,
            ..Default::default()
        };
        let result = create_auto_marker_with_config(1000, &zero_bloom_config, None);
        results.push(BoundaryTestResult::new(
            "Invalid Config: Zero Bloom Bits",
            "marker_bloom_bits",
            "0",
            result.is_ok(),
            if result.is_err() {
                Some(format!("Error: {:?}", result.err()))
            } else {
                None
            },
        ));

        // Test 3: Zero hash functions
        let zero_hash_config = PruneConfig {
            marker_bloom_hash_fns: 0,
            ..Default::default()
        };
        let result = create_auto_marker_with_config(1000, &zero_hash_config, None);
        results.push(BoundaryTestResult::new(
            "Invalid Config: Zero Hash Functions",
            "marker_bloom_hash_fns",
            "0",
            result.is_ok(),
            if result.is_err() {
                Some(format!("Error: {:?}", result.err()))
            } else {
                None
            },
        ));

        // Test 4: Non-power-of-two bloom bits (should still work but may be less efficient)
        let non_pow2_bloom_config = PruneConfig {
            marker_bloom_bits: 1000000, // Not power of two
            ..Default::default()
        };
        let result = create_auto_marker_with_config(1000, &non_pow2_bloom_config, None);
        results.push(BoundaryTestResult::new(
            "Non-Optimal Config: Non-Power-of-Two Bloom Bits",
            "marker_bloom_bits",
            "1000000",
            result.is_ok(),
            if result.is_err() {
                Some(format!("Error: {:?}", result.err()))
            } else {
                None
            },
        ));

        Ok(results)
    }

    #[test]
    fn test_batch_size_boundaries() -> Result<()> {
        println!("🔧 Testing Batch Size Boundaries\n");

        let batch_sizes = vec![
            0,        // Invalid: zero
            1,        // Minimum valid
            10,       // Small
            100,      // Small-medium
            1000,     // Medium
            10000,    // Default
            100000,   // Large
            1000000,  // Very large
            10000000, // Extreme
        ];

        for batch_size in batch_sizes {
            let result = test_batch_size_boundary(batch_size)?;
            result.print();
        }

        Ok(())
    }

    #[test]
    fn test_bloom_filter_boundaries() -> Result<()> {
        println!("🌸 Testing Bloom Filter Boundaries\n");

        let bloom_bits_values = vec![
            0,          // Invalid: zero
            1,          // Minimum
            1024,       // 1KB
            65536,      // 64KB
            1048576,    // 1MB (default)
            16777216,   // 16MB
            134217728,  // 128MB
            1073741824, // 1GB
        ];

        for bloom_bits in bloom_bits_values {
            let result = test_bloom_bits_boundary(bloom_bits)?;
            result.print();
        }

        Ok(())
    }

    #[test]
    fn test_bloom_hash_function_boundaries() -> Result<()> {
        println!("🔀 Testing Bloom Hash Function Boundaries\n");

        let hash_fns_values = vec![
            0,  // Invalid: zero
            1,  // Minimum
            2,  // Small
            4,  // Default
            6,  // Medium
            8,  // Good
            10, // High
            12, // Very high
            16, // Extreme
            20, // Probably too high
        ];

        for hash_fns in hash_fns_values {
            let result = test_bloom_hash_fns_boundary(hash_fns)?;
            result.print();
        }

        Ok(())
    }

    #[test]
    fn test_memory_threshold_boundaries() -> Result<()> {
        println!("💾 Testing Memory Threshold Boundaries\n");

        let memory_thresholds = vec![
            0,    // Invalid: zero
            1,    // Minimum: 1MB
            16,   // Very small
            64,   // Small
            256,  // Medium-small
            512,  // Medium
            1024, // Default: 1GB
            2048, // Large
            4096, // Very large
            8192, // Extreme
        ];

        for memory_threshold_mb in memory_thresholds {
            let result = test_memory_threshold_boundary(memory_threshold_mb)?;
            result.print();
        }

        Ok(())
    }

    #[test]
    fn test_invalid_configuration_handling() -> Result<()> {
        println!("⚠️ Testing Invalid Configuration Handling\n");

        let results = test_invalid_configurations()?;

        for result in results {
            result.print();
        }

        Ok(())
    }

    #[test]
    fn test_configuration_performance_impact() -> Result<()> {
        println!("📊 Testing Configuration Performance Impact\n");

        let configurations = vec![
            (
                "Conservative",
                PruneConfig {
                    marker_batch_size: 1000,
                    marker_bloom_bits: 262144, // 256KB
                    marker_bloom_hash_fns: 4,
                    marker_memory_threshold_mb: 2048,
                    marker_auto_strategy: true,
                    marker_force_persistent: false,
                    ..Default::default()
                },
            ),
            ("Default", PruneConfig::default()),
            (
                "Aggressive",
                PruneConfig {
                    marker_batch_size: 100000,
                    marker_bloom_bits: 4194304, // 4MB
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
                    marker_batch_size: 500000,
                    marker_bloom_bits: 8388608, // 8MB
                    marker_bloom_hash_fns: 12,
                    marker_memory_threshold_mb: 128,
                    marker_auto_strategy: true,
                    marker_force_persistent: false,
                    ..Default::default()
                },
            ),
        ];

        let test_nodes = vec![H256::random(); 10000];

        println!("Configuration Performance Comparison:");
        println!("Config Name\t\tThroughput(nodes/s)\tBatch Size\tBloom Bits\tHash Fns\tStrategy");
        println!("{}", "=".repeat(90));

        for (name, config) in configurations {
            let _start = Instant::now();

            let marker = create_auto_marker_with_config(10000, &config, None)?;

            let mark_start = Instant::now();
            let mut marked_count = 0;
            for node in &test_nodes {
                if marker.mark(*node)? {
                    marked_count += 1;
                }
            }
            let mark_duration = mark_start.elapsed();

            let throughput = if mark_duration.as_secs_f64() > 0.0 {
                marked_count as f64 / mark_duration.as_secs_f64()
            } else {
                0.0
            };

            let strategy = marker.marker_type();

            println!(
                "{: <15}\t{:.0}\t\t\t{}\t\t{}\t{}\t{}",
                name,
                throughput,
                config.marker_batch_size,
                config.marker_bloom_bits,
                config.marker_bloom_hash_fns,
                strategy
            );
        }

        println!();
        Ok(())
    }
}
