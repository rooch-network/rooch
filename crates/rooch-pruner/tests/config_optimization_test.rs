// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_common::bloom_filter::BloomFilter;
use parking_lot::Mutex;
use rand::SeedableRng;
use std::sync::Arc;
use std::time::{Duration, Instant};
use rooch_config::prune_config::PruneConfig;

/// Test different bloom filter configurations for optimal performance
#[test]
fn test_bloom_filter_configuration_optimization() {
    let test_cases = vec![
        (1_000_000, 4, "small_4hash"),
        (1_000_000, 8, "small_8hash"),
        (10_000_000, 4, "medium_4hash"),
        (10_000_000, 8, "medium_8hash"),
        (100_000_000, 4, "large_4hash"),
        (100_000_000, 8, "large_8hash"),
    ];

    let mut rng = rand::rngs::StdRng::seed_from_u64(42);

    for (bits, hash_funcs, name) in test_cases {
        println!("Testing bloom filter: {} bits, {} hash functions - {}", bits, hash_funcs, name);

        let bloom = Arc::new(Mutex::new(BloomFilter::new(bits, hash_funcs)));
        let test_count = 100000; // Fixed test size for comparison

        // Benchmark insertion time
        let start_time = Instant::now();
        for _ in 0..test_count {
            let hash = moveos_types::h256::H256::random_using(&mut rng);
            bloom.lock().insert(&hash);
        }
        let insertion_time = start_time.elapsed();

        // Benchmark lookup time
        let start_time = Instant::now();
        for _ in 0..test_count {
            let hash = moveos_types::h256::H256::random_using(&mut rng);
            bloom.lock().contains(&hash);
        }
        let lookup_time = start_time.elapsed();

        // Calculate memory usage
        let memory_bytes = bits / 8 + (bits % 8 != 0) as usize;
        let memory_mb = memory_bytes as f64 / 1024.0 / 1024.0;

        println!("  Insertion time: {:?}", insertion_time);
        println!("  Lookup time: {:?}", lookup_time);
        println!("  Memory usage: {:.2} MB", memory_mb);
        println!("  Insertion rate: {:.0} ops/sec", test_count as f64 / insertion_time.as_secs_f64());
        println!("  Lookup rate: {:.0} ops/sec", test_count as f64 / lookup_time.as_secs_f64());
        println!();
    }
}

#[test]
fn test_batch_size_optimization() {
    let batch_sizes = vec![100, 500, 1000, 2000, 5000, 10000];
    let total_operations = 100000;

    for batch_size in batch_sizes {
        println!("Testing batch size: {}", batch_size);

        let start_time = Instant::now();
        let mut processed = 0;

        while processed < total_operations {
            // Simulate batch processing
            let current_batch = std::cmp::min(batch_size, total_operations - processed);

            // Simulate some work
            let _dummy: Vec<u64> = (0..current_batch).map(|i| i * i).collect();

            processed += current_batch;

            // Simulate flush time
            std::thread::sleep(Duration::from_micros(100));
        }

        let total_time = start_time.elapsed();
        let throughput = total_operations as f64 / total_time.as_secs_f64();

        println!("  Total time: {:?}", total_time);
        println!("  Throughput: {:.0} ops/sec", throughput);
        println!();
    }
}

#[test]
fn test_scan_batch_vs_performance() {
    let scan_batches = vec![1000, 5000, 10000, 20000, 50000];
    let test_data_size = 100000;

    for scan_batch in scan_batches {
        println!("Testing scan batch size: {}", scan_batch);

        let start_time = Instant::now();
        let mut processed = 0;

        while processed < test_data_size {
            let current_batch = std::cmp::min(scan_batch, test_data_size - processed);

            // Simulate DFS scan
            let _visited: Vec<usize> = (0..current_batch).collect();

            processed += current_batch;
        }

        let scan_time = start_time.elapsed();
        let scan_rate = test_data_size as f64 / scan_time.as_secs_f64();

        println!("  Scan time: {:?}", scan_time);
        println!("  Scan rate: {:.0} nodes/sec", scan_rate);
        println!();
    }
}

#[test]
fn test_memory_pressure_scenarios() {
    use std::sync::atomic::{AtomicU64, Ordering};

    let scenarios = vec![
        ("low_memory", 100_000, 1_000_000),
        ("medium_memory", 500_000, 5_000_000),
        ("high_memory", 1_000_000, 10_000_000),
    ];

    for (name, node_count, bloom_bits) in scenarios {
        println!("Testing memory scenario: {}", name);

        let global_counter = AtomicU64::new(0);

        // Estimate memory usage
        let bloom_memory_mb = bloom_bits as f64 / 8.0 / 1024.0 / 1024.0;
        let node_memory_mb = node_count as f64 * 64.0 / 1024.0 / 1024.0; // Estimate 64 bytes per node
        let total_memory_mb = bloom_memory_mb + node_memory_mb;

        println!("  Estimated bloom filter memory: {:.2} MB", bloom_memory_mb);
        println!("  Estimated node memory: {:.2} MB", node_memory_mb);
        println!("  Total estimated memory: {:.2} MB", total_memory_mb);

        // Test memory allocation performance
        let start_time = Instant::now();

        // Allocate bloom filter
        let bloom = Arc::new(Mutex::new(BloomFilter::new(bloom_bits, 4)));

        // Simulate node storage
        let nodes: Vec<moveos_types::h256::H256> = (0..node_count)
            .map(|_| {
                global_counter.fetch_add(1, Ordering::Relaxed);
                moveos_types::h256::H256::random()
            })
            .collect();

        let allocation_time = start_time.elapsed();

        // Test insertion performance with allocated memory
        let start_time = Instant::now();
        for node in &nodes {
            bloom.lock().insert(node);
        }
        let insertion_time = start_time.elapsed();

        println!("  Allocation time: {:?}", allocation_time);
        println!("  Insertion time: {:?}", insertion_time);
        println!("  Memory allocation rate: {:.0} MB/sec", total_memory_mb / allocation_time.as_secs_f64());
        println!();
    }
}

#[test]
fn test_concurrent_performance() {
    use std::sync::Arc;
    use std::thread;

    let thread_counts = vec![1, 2, 4, 8];
    let operations_per_thread = 50000;

    for thread_count in thread_counts {
        println!("Testing with {} threads", thread_count);

        let bloom = Arc::new(Mutex::new(BloomFilter::new(10_000_000, 4)));
        let start_time = Instant::now();

        let handles: Vec<_> = (0..thread_count)
            .map(|_| {
                let bloom = bloom.clone();
                thread::spawn(move || {
                    let mut rng = rand::rngs::StdRng::seed_from_u64(42);

                    for _ in 0..operations_per_thread {
                        let hash = moveos_types::h256::H256::random_using(&mut rng);
                        bloom.lock().insert(&hash);
                        bloom.lock().contains(&hash);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let total_time = start_time.elapsed();
        let total_operations = (thread_count * operations_per_thread) as f64;
        let throughput = total_operations / total_time.as_secs_f64();

        println!("  Total time: {:?}", total_time);
        println!("  Total operations: {}", total_operations as u64);
        println!("  Throughput: {:.0} ops/sec", throughput);
        println!("  Throughput per thread: {:.0} ops/sec", throughput / thread_count as f64);
        println!();
    }
}

#[test]
fn test_configuration_recommendations() {
    println!("Configuration Recommendations:");
    println!("============================");

    // Small deployments (< 1M nodes)
    println!("Small Deployments (< 1M nodes):");
    println!("  - Bloom filter bits: 8M (2^23)");
    println!("  - Scan batch: 1,000");
    println!("  - Delete batch: 500");
    println!("  - Interval: 60 seconds");
    println!();

    // Medium deployments (1M - 10M nodes)
    println!("Medium Deployments (1M - 10M nodes):");
    println!("  - Bloom filter bits: 80M (2^26)");
    println!("  - Scan batch: 5,000");
    println!("  - Delete batch: 2,000");
    println!("  - Interval: 30 seconds");
    println!();

    // Large deployments (> 10M nodes)
    println!("Large Deployments (> 10M nodes):");
    println!("  - Bloom filter bits: 800M (2^29)");
    println!("  - Scan batch: 10,000");
    println!("  - Delete batch: 5,000");
    println!("  - Interval: 15 seconds");
    println!();

    // Memory considerations
    println!("Memory Considerations:");
    println!("  - Bloom filter: ~1 bit per expected node");
    println!("  - Additional overhead: 10-20%% for bloom filter structures");
    println!("  - Working memory: 100-500MB for batch processing");
    println!();
}

fn create_optimized_config(node_count: usize) -> PruneConfig {
    let bloom_bits = match node_count {
        0..=1_000_000 => 8_000_000,      // 8M bits = 1MB
        1_000_001..=10_000_000 => 80_000_000, // 80M bits = 10MB
        _ => 800_000_000,                     // 800M bits = 100MB
    };

    let scan_batch = match node_count {
        0..=1_000_000 => 1_000,
        1_000_001..=10_000_000 => 5_000,
        _ => 10_000,
    };

    let delete_batch = scan_batch / 2;
    let interval_s = match node_count {
        0..=1_000_000 => 60,
        1_000_001..=10_000_000 => 30,
        _ => 15,
    };

    PruneConfig {
        enable: true,
        boot_cleanup_done: false,
        scan_batch,
        delete_batch,
        interval_s,
        bloom_bits,
        enable_reach_seen_cf: false,
        window_days: 30,
    }
}

#[test]
fn test_optimized_configurations() {
    let test_cases = vec![
        ("small", 500_000),
        ("medium", 5_000_000),
        ("large", 50_000_000),
    ];

    for (name, node_count) in test_cases {
        let config = create_optimized_config(node_count);

        println!("Optimized configuration for {} ({} nodes):", name, node_count);
        println!("  Bloom filter bits: {}", config.bloom_bits);
        println!("  Scan batch: {}", config.scan_batch);
        println!("  Delete batch: {}", config.delete_batch);
        println!("  Interval: {} seconds", config.interval_s);
        println!("  Estimated memory: {:.2} MB", config.bloom_bits as f64 / 8.0 / 1024.0 / 1024.0);
        println!();
    }
}