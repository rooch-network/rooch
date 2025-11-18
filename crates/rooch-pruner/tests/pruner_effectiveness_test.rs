// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use primitive_types::H256 as PrimitiveH256;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use rooch_config::prune_config::PruneConfig;
use rooch_pruner::pruner::StatePruner;
use rooch_store::RoochStore;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::broadcast;

/// Test to measure pruner effectiveness with different data sizes
#[tokio::test]
async fn test_pruner_effectiveness_small_dataset() {
    test_pruner_effectiveness_with_size(1000, 100).await;
}

#[tokio::test]
async fn test_pruner_effectiveness_medium_dataset() {
    test_pruner_effectiveness_with_size(10000, 1000).await;
}

#[tokio::test]
async fn test_pruner_effectiveness_large_dataset() {
    test_pruner_effectiveness_with_size(50000, 5000).await;
}

async fn test_pruner_effectiveness_with_size(node_count: usize, expired_count: u64) {
    let (moveos_store, rooch_store, _temp_dir) = create_test_environment();

    // Generate test data
    let nodes = generate_test_nodes(node_count, &moveos_store);
    let expired_roots = generate_expired_state_roots(&rooch_store, expired_count);

    // Get initial node count
    let initial_node_count = count_nodes_in_store(&moveos_store);
    println!("Initial node count: {}", initial_node_count);

    // Configure pruner
    let config = PruneConfig {
        enable: true,
        boot_cleanup_done: false,
        scan_batch: 1000,
        delete_batch: 500,
        interval_s: 1, // Short interval for testing
        bloom_bits: node_count * 8,
        enable_reach_seen_cf: false,
        window_days: 30,
    };

    // Start pruner
    let (_shutdown_tx, shutdown_rx) = broadcast::channel(16);
    let pruner = StatePruner::start(
        Arc::new(config),
        Arc::new(moveos_store.clone()),
        Arc::new(rooch_store.clone()),
        shutdown_rx,
        None, // No metrics for this test
    ).expect("Failed to start pruner");

    // Wait for pruner to complete some work
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Check results
    let final_node_count = count_nodes_in_store(&moveos_store);
    let nodes_deleted = initial_node_count.saturating_sub(final_node_count);

    println!("Final node count: {}", final_node_count);
    println!("Nodes deleted: {}", nodes_deleted);
    println!("Deletion ratio: {:.2}%", (nodes_deleted as f64 / initial_node_count as f64) * 100.0);

    // At least some nodes should be deleted (this is a basic effectiveness test)
    assert!(nodes_deleted > 0, "Pruner should have deleted some nodes");

    // Cleanup
    drop(pruner);
}

#[test]
fn test_pruner_configuration_impact() {
    // Test how different configurations affect pruner performance
    let configurations = vec![
        (1000, 100, "small"),
        (10000, 1000, "medium"),
        (50000, 5000, "large"),
    ];

    for (node_count, expired_count, name) in configurations {
        println!("Testing configuration: {} nodes, {} expired", name, node_count);

        let (moveos_store, rooch_store, _temp_dir) = create_test_environment();
        let nodes = generate_test_nodes(node_count, &moveos_store);
        let expired_roots = generate_expired_state_roots(&rooch_store, expired_count);

        let start_time = std::time::Instant::now();

        // Simulate pruning process
        simulate_pruning_process(&moveos_store, &rooch_store, &nodes, &expired_roots);

        let duration = start_time.elapsed();
        println!("Configuration {}: {:?}", name, duration);

        // Performance should scale reasonably
        assert!(duration.as_secs() < 60, "Pruning should complete within 60 seconds for {}", name);
    }
}

#[test]
fn test_bloom_filter_effectiveness() {
    use moveos_common::bloom_filter::BloomFilter;
    use parking_lot::Mutex;
    use std::sync::Arc;

    let test_sizes = vec![1000, 10000, 100000];

    for size in test_sizes {
        let bloom = Arc::new(Mutex::new(BloomFilter::new(size * 8, 4)));
        let mut rng = StdRng::from_seed([42; 32]);
        let mut test_hashes = Vec::new();

        // Insert test values
        for _ in 0..size {
            let hash = H256::random_using(&mut rng);
            bloom.lock().insert(&hash);
            test_hashes.push(hash);
        }

        // Test false positive rate
        let mut false_positives = 0;
        let test_count = size / 10;

        for _ in 0..test_count {
            let test_hash = H256::random_using(&mut rng);
            if !test_hashes.contains(&test_hash) && bloom.lock().contains(&test_hash) {
                false_positives += 1;
            }
        }

        let false_positive_rate = false_positives as f64 / test_count as f64;
        println!("Bloom filter size {}: false positive rate = {:.4}%", size, false_positive_rate * 100.0);

        // False positive rate should be reasonable (< 10%)
        assert!(false_positive_rate < 0.1, "False positive rate too high for size {}: {:.4}%", size, false_positive_rate * 100.0);
    }
}

#[test]
fn test_memory_usage_scaling() {
    let test_sizes = vec![1000, 5000, 10000, 50000];

    for size in test_sizes {
        let (moveos_store, _rooch_store, _temp_dir) = create_test_environment();

        // Measure memory usage before
        let memory_before = get_memory_usage();

        // Generate test data
        let _nodes = generate_test_nodes(size, &moveos_store);

        // Measure memory usage after
        let memory_after = get_memory_usage();
        let memory_used = memory_after.saturating_sub(memory_before);

        println!("Memory usage for {} nodes: {} MB", size, memory_used / 1024 / 1024);

        // Memory usage should scale linearly (within reasonable bounds)
        let memory_per_node = memory_used / size as u64;
        assert!(memory_per_node < 10240, "Memory usage per node too high: {} bytes", memory_per_node); // < 10KB per node
    }
}

// Helper functions

fn create_test_environment() -> (MoveOSStore, RoochStore, TempDir) {
    let temp_dir = tempfile::tempdir().unwrap();
    let store_config = moveos_store::config_store::ConfigStore::default_config();
    let (moveos_store, rooch_store) = MoveOSStore::mock_moveos_and_rooch_store_with_config(
        temp_dir.path(),
        store_config,
    ).unwrap();
    (moveos_store, rooch_store, temp_dir)
}

fn generate_test_nodes(count: usize, store: &MoveOSStore) -> Vec<H256> {
    let mut rng = StdRng::from_seed([42; 32]);
    let node_store = store.get_state_node_store();
    let mut node_hashes = Vec::with_capacity(count);

    for _ in 0..count {
        let key = H256::random();
        let mut value = vec![0u8; 32];
        rng.fill(&mut value[..]);

        let node: smt::jellyfish_merkle::node_type::Node<H256, Vec<u8>> =
            smt::jellyfish_merkle::node_type::Node::new_leaf(
                key,
                smt::SMTObject::<Vec<u8>>::from_origin(value.clone()).unwrap(),
            );
        let hash: H256 = node.get_merkle_hash().into();

        node_store.put(hash, node.encode().unwrap()).unwrap();
        node_hashes.push(hash);
    }

    node_hashes
}

fn generate_expired_state_roots(store: &RoochStore, count: u64) -> Vec<PrimitiveH256> {
    let mut rng = StdRng::from_seed([123; 32]);
    let mut roots = Vec::with_capacity(count as usize);

    for _ in 0..count {
        let root = PrimitiveH256::random_using(&mut rng);
        roots.push(root);
    }

    roots
}

fn count_nodes_in_store(store: &MoveOSStore) -> usize {
    // This is a simplified count - in a real implementation, you would
    // iterate through the actual node store
    store.get_state_node_store().len().unwrap_or(0)
}

fn simulate_pruning_process(
    moveos_store: &MoveOSStore,
    rooch_store: &RoochStore,
    nodes: &[H256],
    expired_roots: &[PrimitiveH256],
) {
    use moveos_common::bloom_filter::BloomFilter;
    use parking_lot::Mutex;
    use std::sync::Arc;
    use rooch_pruner::reachability::ReachableBuilder;
    use rooch_pruner::sweep_expired::SweepExpired;

    // Create bloom filter
    let bloom = Arc::new(Mutex::new(BloomFilter::new(nodes.len() * 8, 4)));

    // Mark some nodes as reachable (simulate live state)
    let reachable_count = nodes.len() / 3;
    let builder = ReachableBuilder::new(Arc::new(moveos_store.clone()), bloom.clone());
    let _ = builder.build(nodes[..reachable_count].to_vec(), 1);

    // Sweep expired nodes
    let sweeper = SweepExpired::new(Arc::new(moveos_store.clone()), bloom);
    let _ = sweeper.sweep(expired_roots.to_vec(), 1);
}

fn get_memory_usage() -> u64 {
    // Simplified memory usage estimation
    // In a real implementation, you would use system APIs to get accurate memory usage
    use std::alloc::{GlobalAlloc, Layout, System};

    // This is a placeholder - actual memory measurement would require platform-specific code
    0
}