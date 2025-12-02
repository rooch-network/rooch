// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//! Performance benchmarks for PersistentMarker vs InMemoryMarker
//! Tests throughput, memory usage, and scaling characteristics

#[cfg(test)]
mod tests {
    use crate::config::GCConfig as PruneConfig;
    use crate::marker::{
        create_auto_marker_with_config, InMemoryMarker, NodeMarker, PersistentMarker,
    };
    use anyhow::Result;
    use moveos_types::h256::H256;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    /// Performance test results structure
    #[derive(Debug, Clone)]
    struct BenchmarkResult {
        marker_type: String,
        node_count: usize,
        duration: Duration,
        throughput_nodes_per_sec: f64,
        marked_count: u64,
        memory_estimate_mb: f64,
        batch_flushes: u32,
    }

    impl BenchmarkResult {
        fn new(
            marker_type: &str,
            node_count: usize,
            duration: Duration,
            marked_count: u64,
            memory_estimate_mb: f64,
            batch_flushes: u32,
        ) -> Self {
            let throughput_nodes_per_sec = if duration.as_secs_f64() > 0.0 {
                node_count as f64 / duration.as_secs_f64()
            } else {
                0.0
            };

            Self {
                marker_type: marker_type.to_string(),
                node_count,
                duration,
                throughput_nodes_per_sec,
                marked_count,
                memory_estimate_mb,
                batch_flushes,
            }
        }

        fn print(&self) {
            println!("=== {} Performance Benchmark ===", self.marker_type);
            println!("Node Count: {}", self.node_count);
            println!("Duration: {:.3}s", self.duration.as_secs_f64());
            println!("Throughput: {:.0} nodes/sec", self.throughput_nodes_per_sec);
            println!("Marked Count: {}", self.marked_count);
            println!("Memory Estimate: {:.1}MB", self.memory_estimate_mb);
            println!("Batch Flushes: {}", self.batch_flushes);
            println!(
                "Mark Efficiency: {:.2}%",
                (self.marked_count as f64 / self.node_count as f64) * 100.0
            );
            println!();
        }
    }

    /// Generate test nodes for benchmarking
    fn generate_test_nodes(count: usize) -> Vec<H256> {
        let mut nodes = Vec::with_capacity(count);
        for i in 0..count {
            // Generate deterministic but varied hashes
            let mut bytes = [0u8; 32];
            bytes[0..8].copy_from_slice(&(i as u64).to_le_bytes());
            nodes.push(H256::from(bytes));
        }
        nodes
    }

    /// Benchmark InMemoryMarker performance
    fn benchmark_inmemory_marker(node_count: usize) -> Result<BenchmarkResult> {
        let nodes = generate_test_nodes(node_count);
        let marker = InMemoryMarker::with_capacity(node_count);

        let start = Instant::now();
        let mut marked_count = 0;

        for node in &nodes {
            if marker.mark(*node)? {
                marked_count += 1;
            }
        }

        let duration = start.elapsed();
        let memory_estimate = marker.estimate_memory_usage() as f64 / (1024.0 * 1024.0);

        Ok(BenchmarkResult::new(
            "InMemoryMarker",
            node_count,
            duration,
            marked_count,
            memory_estimate,
            0, // No batch flushes for InMemoryMarker
        ))
    }

    /// Benchmark PersistentMarker performance
    fn benchmark_persistent_marker(
        node_count: usize,
        batch_size: usize,
    ) -> Result<BenchmarkResult> {
        let nodes = generate_test_nodes(node_count);
        let marker = PersistentMarker::with_batch_size("benchmark_cf".to_string(), batch_size)?;

        let start = Instant::now();
        let mut marked_count = 0;

        for node in &nodes {
            if marker.mark(*node)? {
                marked_count += 1;
            }
        }

        let duration = start.elapsed();
        let memory_estimate = (node_count * 64) as f64 / (1024.0 * 1024.0); // 64 bytes per node estimate

        // Calculate approximate number of batch flushes
        let batch_flushes = (node_count / batch_size) as u32;

        Ok(BenchmarkResult::new(
            "PersistentMarker",
            node_count,
            duration,
            marked_count,
            memory_estimate,
            batch_flushes,
        ))
    }

    /// Benchmark with configuration-driven marker creation
    fn benchmark_config_driven_marker(
        node_count: usize,
        config: &PruneConfig,
    ) -> Result<BenchmarkResult> {
        let nodes = generate_test_nodes(node_count);
        let marker = create_auto_marker_with_config(node_count, config, None)?;

        let start = Instant::now();
        let mut marked_count = 0;

        for node in &nodes {
            if marker.mark(*node)? {
                marked_count += 1;
            }
        }

        let duration = start.elapsed();
        let memory_estimate = (node_count * 64) as f64 / (1024.0 * 1024.0); // 64 bytes per node estimate

        Ok(BenchmarkResult::new(
            marker.marker_type(),
            node_count,
            duration,
            marked_count,
            memory_estimate,
            0, // Hard to track exact flushes without instrumentation
        ))
    }

    #[test]
    fn test_basic_performance_comparison() -> Result<()> {
        println!("🚀 Starting Basic Performance Comparison Benchmark\n");

        let node_counts = vec![1_000, 10_000, 100_000];

        for &node_count in &node_counts {
            println!("Testing with {} nodes:", node_count);

            // Benchmark InMemoryMarker
            let inmemory_result = benchmark_inmemory_marker(node_count)?;
            inmemory_result.print();

            // Benchmark PersistentMarker with default batch size
            let persistent_result = benchmark_persistent_marker(node_count, 10_000)?;
            persistent_result.print();

            // Performance comparison
            let speedup = persistent_result.throughput_nodes_per_sec
                / inmemory_result.throughput_nodes_per_sec;
            println!("Speedup (Persistent/InMemory): {:.2}x", speedup);

            let memory_ratio =
                persistent_result.memory_estimate_mb / inmemory_result.memory_estimate_mb;
            println!("Memory Ratio (Persistent/InMemory): {:.2}x", memory_ratio);
            println!("{}\n", "=".repeat(50));
        }

        Ok(())
    }

    #[test]
    fn test_batch_size_performance_impact() -> Result<()> {
        println!("🎯 Testing Batch Size Performance Impact\n");

        let node_count = 100_000;
        let batch_sizes = vec![1_000, 5_000, 10_000, 20_000, 50_000];

        for &batch_size in &batch_sizes {
            println!("Testing with batch size {}:", batch_size);

            let result = benchmark_persistent_marker(node_count, batch_size)?;
            result.print();

            // Calculate batch efficiency
            let avg_batch_time = if result.batch_flushes > 0 {
                result.duration.as_millis() as f64 / result.batch_flushes as f64
            } else {
                0.0
            };
            println!("Avg Batch Time: {:.1}ms", avg_batch_time);
            println!("{}\n", "=".repeat(50));
        }

        Ok(())
    }

    #[test]
    fn test_configuration_driven_performance() -> Result<()> {
        println!("⚙️ Testing Configuration-Driven Performance\n");

        let node_count = 200_000;

        // Test different configurations
        let configs = vec![
            (
                "Conservative",
                PruneConfig {
                    marker_batch_size: 1_000,
                    marker_memory_threshold_mb: 512,
                    ..Default::default()
                },
            ),
            ("Default", PruneConfig::default()),
            (
                "High-Performance",
                PruneConfig {
                    marker_batch_size: 50_000,
                    marker_memory_threshold_mb: 2048,
                    ..Default::default()
                },
            ),
        ];

        for (name, config) in configs {
            println!("Testing {} configuration:", name);

            let result = benchmark_config_driven_marker(node_count, &config)?;
            result.print();
            println!("{}\n", "=".repeat(50));
        }

        Ok(())
    }

    #[test]
    fn test_memory_efficiency_scaling() -> Result<()> {
        println!("💾 Testing Memory Efficiency Scaling\n");

        let node_counts = vec![10_000, 50_000, 100_000, 500_000, 1_000_000];

        for &node_count in &node_counts {
            println!("Testing memory efficiency with {} nodes:", node_count);

            // InMemoryMarker memory usage
            let inmemory_memory = node_count * 64; // 64 bytes per node estimate
            let inmemory_mb = inmemory_memory as f64 / (1024.0 * 1024.0);

            // PersistentMarker memory usage (bloom filter + batch buffer)
            let persistent_memory = 128 * 1024 + (10_000 * 32); // 128KB bloom + batch buffer
            let persistent_mb = persistent_memory as f64 / (1024.0 * 1024.0);

            println!("InMemoryMarker Estimate: {:.1}MB", inmemory_mb);
            println!("PersistentMarker Estimate: {:.1}MB", persistent_mb);
            println!(
                "Memory Savings: {:.1}%",
                ((inmemory_mb - persistent_mb) / inmemory_mb) * 100.0
            );
            println!(
                "Nodes per MB (Persistent): {:.0}",
                node_count as f64 / persistent_mb
            );
            println!("{}\n", "=".repeat(50));
        }

        Ok(())
    }

    #[test]
    fn test_strategy_selection_performance() -> Result<()> {
        println!("🧠 Testing Strategy Selection Performance\n");

        let test_cases = vec![
            (10_000, "Small dataset (should be InMemory)"),
            (100_000, "Medium dataset"),
            (1_000_000, "Large dataset (should be Persistent)"),
        ];

        for (node_count, description) in test_cases {
            println!("{}:", description);

            let config = PruneConfig::default();
            let marker = create_auto_marker_with_config(node_count, &config, None)?;

            let start = Instant::now();
            let nodes = generate_test_nodes(node_count.min(10000)); // Limit for test performance

            for node in &nodes {
                if marker.mark(*node)? {
                    // marked_count would be incremented here
                }
            }

            let duration = start.elapsed();
            let throughput = nodes.len() as f64 / duration.as_secs_f64();

            println!("Selected Strategy: {}", marker.marker_type());
            println!("Test Nodes: {}", nodes.len());
            println!("Throughput: {:.0} nodes/sec", throughput);
            println!("Duration: {:.3}s", duration.as_secs_f64());
            println!("{}\n", "=".repeat(50));
        }

        Ok(())
    }

    #[test]
    fn test_concurrent_marking_performance() -> Result<()> {
        println!("🔄 Testing Concurrent Marking Performance\n");

        use std::sync::atomic::{AtomicU64, Ordering};
        use std::thread;

        let node_count = 50_000;
        let thread_count = 4;
        let nodes_per_thread = node_count / thread_count;

        let marker = Arc::new(PersistentMarker::with_batch_size(
            "concurrent_cf".to_string(),
            5_000,
        )?);
        let marked_total = Arc::new(AtomicU64::new(0));

        let start = Instant::now();

        let mut handles = Vec::new();

        for thread_id in 0..thread_count {
            let marker_clone = Arc::clone(&marker);
            let marked_total_clone = Arc::clone(&marked_total);
            let start_node = thread_id * nodes_per_thread;
            let end_node = start_node + nodes_per_thread;

            let handle = thread::spawn(move || -> Result<()> {
                for i in start_node..end_node {
                    let mut bytes = [0u8; 32];
                    bytes[0..8].copy_from_slice(&(i as u64).to_le_bytes());
                    let node = H256::from(bytes);

                    if marker_clone.mark(node)? {
                        marked_total_clone.fetch_add(1, Ordering::Relaxed);
                    }
                }
                Ok(())
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap()?;
        }

        let duration = start.elapsed();
        let total_marked = marked_total.load(Ordering::Relaxed);
        let throughput = total_marked as f64 / duration.as_secs_f64();

        println!("Concurrent Marking Results:");
        println!("Threads: {}", thread_count);
        println!("Total Nodes: {}", node_count);
        println!("Marked Nodes: {}", total_marked);
        println!("Duration: {:.3}s", duration.as_secs_f64());
        println!("Throughput: {:.0} nodes/sec", throughput);
        println!(
            "Concurrency Efficiency: {:.2}%",
            (total_marked as f64 / node_count as f64) * 100.0
        );

        Ok(())
    }

    #[test]
    fn test_is_marked_performance() -> Result<()> {
        println!("🔍 Testing is_marked() Performance\n");

        let node_count = 100_000;
        let nodes = generate_test_nodes(node_count);

        // Test with InMemoryMarker
        let inmemory_marker = InMemoryMarker::with_capacity(node_count);
        for node in &nodes {
            inmemory_marker.mark(*node)?;
        }

        let start = Instant::now();
        let mut inmemory_found_count = 0;
        for node in &nodes {
            if inmemory_marker.is_marked(node) {
                inmemory_found_count += 1;
            }
        }
        let inmemory_duration = start.elapsed();

        // Test with PersistentMarker
        let persistent_marker = PersistentMarker::new("is_marked_cf".to_string())?;
        for node in &nodes {
            persistent_marker.mark(*node)?;
        }

        let start = Instant::now();
        let mut persistent_found_count = 0;
        for node in &nodes {
            if persistent_marker.is_marked(node) {
                persistent_found_count += 1;
            }
        }
        let persistent_duration = start.elapsed();

        println!("is_marked() Performance Results:");
        println!(
            "InMemoryMarker - Found: {}, Duration: {:.3}s, Throughput: {:.0} lookups/sec",
            inmemory_found_count,
            inmemory_duration.as_secs_f64(),
            nodes.len() as f64 / inmemory_duration.as_secs_f64()
        );
        println!(
            "PersistentMarker - Found: {}, Duration: {:.3}s, Throughput: {:.0} lookups/sec",
            persistent_found_count,
            persistent_duration.as_secs_f64(),
            nodes.len() as f64 / persistent_duration.as_secs_f64()
        );

        Ok(())
    }
}
