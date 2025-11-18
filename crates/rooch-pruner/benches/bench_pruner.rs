// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use moveos_common::bloom_filter::BloomFilter;
use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use parking_lot::Mutex;
use primitive_types::H256 as PrimitiveH256;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use rooch_config::prune_config::PruneConfig;
use rooch_pruner::pruner::StatePruner;
use rooch_pruner::reachability::ReachableBuilder;
use rooch_pruner::sweep_expired::SweepExpired;
use rooch_store::RoochStore;
use smt::jellyfish_merkle::node_type::Node;
use smt::SMTObject;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::broadcast;

fn create_test_node_store() -> (MoveOSStore, RoochStore, TempDir) {
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

    for i in 0..count {
        let key = H256::random();
        let mut value = vec![0u8; 32];
        rng.fill(&mut value[..]);

        let node: Node<H256, Vec<u8>> = Node::new_leaf(
            key,
            SMTObject::<Vec<u8>>::from_origin(value.clone()).unwrap(),
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

    for i in 0..count {
        let root = PrimitiveH256::random_using(&mut rng);

        // Create a mock state change set
        let mut state_change_set = rooch_types::test_utils::random_state_change_set();
        state_change_set.state_root = root;

        // This would normally be stored during transaction processing
        // For benchmarking, we'll simulate this
        roots.push(root);
    }

    roots
}

fn bench_bloom_filter_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("bloom_filter");

    for size in [1000, 10000, 100000, 1000000].iter() {
        group.bench_with_input(
            BenchmarkId::new("insert_and_contains", size),
            size,
            |b, &size| {
                let mut rng = StdRng::from_seed([42; 32]);
                let bloom = Arc::new(Mutex::new(BloomFilter::new(size * 8, 4)));

                b.iter(|| {
                    // Insert
                    for _ in 0..size {
                        let hash = H256::random_using(&mut rng);
                        bloom.lock().insert(&hash);
                    }

                    // Check contains
                    for _ in 0..size/10 {
                        let hash = H256::random_using(&mut rng);
                        black_box(bloom.lock().contains(&hash));
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_reachable_builder(c: &mut Criterion) {
    let mut group = c.benchmark_group("reachable_builder");

    for node_count in [1000, 5000, 10000, 50000].iter() {
        group.bench_with_input(
            BenchmarkId::new("build_reachable_set", node_count),
            node_count,
            |b, &node_count| {
                let (moveos_store, _rooch_store, _temp_dir) = create_test_node_store();
                let nodes = generate_test_nodes(node_count, &moveos_store);
                let bloom = Arc::new(Mutex::new(BloomFilter::new(node_count * 8, 4)));

                b.iter(|| {
                    let builder = ReachableBuilder::new(Arc::new(moveos_store.clone()), bloom.clone());
                    let result = builder.build(nodes.clone(), num_cpus::get());
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

fn bench_sweep_expired(c: &mut Criterion) {
    let mut group = c.benchmark_group("sweep_expired");

    for batch_size in [100, 500, 1000, 2000].iter() {
        group.bench_with_input(
            BenchmarkId::new("sweep_batch", batch_size),
            batch_size,
            |b, &batch_size| {
                let (moveos_store, rooch_store, _temp_dir) = create_test_node_store();
                let nodes = generate_test_nodes(batch_size * 2, &moveos_store);
                let expired_roots = generate_expired_state_roots(&rooch_store, batch_size as u64);
                let bloom = Arc::new(Mutex::new(BloomFilter::new(batch_size * 16, 4)));

                // Mark half the nodes as reachable
                let builder = ReachableBuilder::new(Arc::new(moveos_store.clone()), bloom.clone());
                let _ = builder.build(nodes[..batch_size].to_vec(), 1);

                b.iter(|| {
                    let sweeper = SweepExpired::new(Arc::new(moveos_store.clone()), bloom.clone());
                    let result = sweeper.sweep(expired_roots.clone(), 1);
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

fn bench_pruner_config_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("pruner_config_scenarios");

    // Test different bloom filter sizes
    for bloom_bits in [1_000_000, 10_000_000, 100_000_000].iter() {
        group.bench_with_input(
            BenchmarkId::new("bloom_filter_size", bloom_bits),
            bloom_bits,
            |b, &bloom_bits| {
                let (moveos_store, _rooch_store, _temp_dir) = create_test_node_store();
                let config = PruneConfig {
                    enable: true,
                    boot_cleanup_done: false,
                    scan_batch: 10000,
                    delete_batch: 5000,
                    interval_s: 60,
                    bloom_bits,
                    enable_reach_seen_cf: false,
                    window_days: 30,
                };

                b.iter(|| {
                    let bloom = BloomFilter::new(config.bloom_bits, 4);
                    black_box(bloom)
                });
            },
        );
    }

    // Test different batch sizes
    for batch_size in [1000, 5000, 10000, 20000].iter() {
        group.bench_with_input(
            BenchmarkId::new("scan_batch_size", batch_size),
            batch_size,
            |b, &batch_size| {
                let (moveos_store, _rooch_store, _temp_dir) = create_test_node_store();
                let nodes = generate_test_nodes(batch_size, &moveos_store);
                let bloom = Arc::new(Mutex::new(BloomFilter::new(batch_size * 8, 4)));

                b.iter(|| {
                    let builder = ReachableBuilder::new(Arc::new(moveos_store.clone()), bloom.clone());
                    let result = builder.build(nodes.clone(), 1);
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    for node_count in [10000, 50000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::new("memory_footprint", node_count),
            node_count,
            |b, &node_count| {
                b.iter(|| {
                    let (moveos_store, _rooch_store, _temp_dir) = create_test_node_store();
                    let _nodes = generate_test_nodes(node_count, &moveos_store);
                    let bloom = Arc::new(Mutex::new(BloomFilter::new(node_count * 8, 4)));

                    // Simulate memory usage by creating data structures
                    let builder = ReachableBuilder::new(Arc::new(moveos_store), bloom);
                    black_box(builder)
                });
            },
        );
    }

    group.finish();
}

// Benchmark to simulate real-world pruning scenarios
fn bench_real_world_scenario(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world_scenario");

    // Simulate a scenario with 30 days of data, pruning old data
    group.bench_function("30_day_pruning", |b| {
        b.iter(|| {
            let (moveos_store, rooch_store, _temp_dir) = create_test_node_store();

            // Generate nodes representing 30 days of state changes
            let total_nodes = 100000; // Approximate nodes over 30 days
            let _nodes = generate_test_nodes(total_nodes, &moveos_store);

            // Generate expired roots (older than 30 days)
            let expired_root_count = 30000; // Roots to be pruned
            let expired_roots = generate_expired_state_roots(&rooch_store, expired_root_count);

            // Create bloom filter for the scenario
            let bloom = Arc::new(Mutex::new(BloomFilter::new(total_nodes * 8, 4)));

            // Mark recent nodes as reachable (last 30%)
            let recent_nodes_count = total_nodes * 3 / 10;
            let recent_nodes = generate_test_nodes(recent_nodes_count, &moveos_store);
            let builder = ReachableBuilder::new(Arc::new(moveos_store.clone()), bloom.clone());
            let _ = builder.build(recent_nodes, num_cpus::get());

            // Sweep expired nodes
            let sweeper = SweepExpired::new(Arc::new(moveos_store), bloom);
            let result = sweeper.sweep(expired_roots, num_cpus::get());

            black_box(result)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_bloom_filter_performance,
    bench_reachable_builder,
    bench_sweep_expired,
    bench_pruner_config_scenarios,
    bench_memory_usage,
    bench_real_world_scenario
);

criterion_main!(benches);