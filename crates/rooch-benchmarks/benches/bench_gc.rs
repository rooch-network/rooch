// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//! GC Mark Phase Benchmarks
//!
//! This benchmark suite tests the performance of the GC Mark Phase,
//! focusing on:
//! - Parallel vs single-threaded traversal
//! - Different worker counts and batch sizes
//! - AtomicBloomFilter vs Mutex-based BloomFilter

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rooch_benchmarks::gc::TreeBuilder;
use std::sync::Arc;
use std::time::Duration;

// External crate imports for benchmarks
extern crate moveos_common;
extern crate moveos_store;
extern crate parking_lot;
extern crate rooch_pruner;

use moveos_common::bloom_filter::BloomFilter;
use moveos_store::MoveOSStore;
use parking_lot::Mutex;
use rooch_pruner::marker::{AtomicBloomFilterMarker, BloomFilterMarker};
use rooch_pruner::reachability::ReachableBuilder;

/// Benchmark different data scales with single-thread vs parallel
fn bench_mark_phase_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("mark_phase_scaling");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    // Test different scales: 10K, 50K, 100K nodes
    let scales = vec![("10K", 10_000), ("50K", 50_000), ("100K", 100_000)];

    for (name, node_count) in scales {
        // Setup test data
        let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
        let store = Arc::new(store);
        let builder = TreeBuilder::new(store.clone());
        let (root_hash, _all_hashes) = builder.create_tree(node_count).unwrap();

        // Benchmark single-threaded
        group.bench_with_input(
            BenchmarkId::new("single_thread", name),
            &(store.clone(), root_hash),
            |b, (store, root)| {
                b.iter(|| {
                    let bloom = Arc::new(Mutex::new(BloomFilter::new(1 << 20, 4)));
                    let reachable_builder = ReachableBuilder::new((**store).clone(), bloom);
                    let marker = AtomicBloomFilterMarker::new(1 << 20, 4);
                    let count = reachable_builder
                        .build_with_marker(vec![*root], &marker, 1000)
                        .unwrap();
                    black_box(count);
                });
            },
        );

        // Benchmark parallel with 4 workers
        group.bench_with_input(
            BenchmarkId::new("parallel_4_workers", name),
            &(store.clone(), root_hash),
            |b, (store, root)| {
                b.iter(|| {
                    let bloom = Arc::new(Mutex::new(BloomFilter::new(1 << 20, 4)));
                    let reachable_builder = ReachableBuilder::new((**store).clone(), bloom);
                    let marker = AtomicBloomFilterMarker::new(1 << 20, 4);
                    let count = reachable_builder
                        .build_with_marker_parallel(vec![*root], 4, &marker, 1000)
                        .unwrap();
                    black_box(count);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark different worker counts
fn bench_mark_phase_workers(c: &mut Criterion) {
    let mut group = c.benchmark_group("mark_phase_workers");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    // Setup test data - 100K nodes
    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let store = Arc::new(store);
    let builder = TreeBuilder::new(store.clone());
    let (root_hash, _all_hashes) = builder.create_tree(100_000).unwrap();

    // Test different worker counts
    let worker_counts = vec![1, 2, 4, 8];

    for workers in worker_counts {
        group.bench_with_input(
            BenchmarkId::from_parameter(workers),
            &workers,
            |b, &workers| {
                b.iter(|| {
                    let bloom = Arc::new(Mutex::new(BloomFilter::new(1 << 20, 4)));
                    let reachable_builder = ReachableBuilder::new((*store).clone(), bloom);
                    let marker = AtomicBloomFilterMarker::new(1 << 20, 4);
                    let count = if workers == 1 {
                        reachable_builder
                            .build_with_marker(vec![root_hash], &marker, 1000)
                            .unwrap()
                    } else {
                        reachable_builder
                            .build_with_marker_parallel(vec![root_hash], workers, &marker, 1000)
                            .unwrap()
                    };
                    black_box(count);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark different batch sizes
fn bench_mark_phase_batch_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("mark_phase_batch_size");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    // Setup test data - 100K nodes
    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let store = Arc::new(store);
    let builder = TreeBuilder::new(store.clone());
    let (root_hash, _all_hashes) = builder.create_tree(100_000).unwrap();

    // Test different batch sizes
    let batch_sizes = vec![100, 500, 1000, 5000, 10000];

    for batch_size in batch_sizes {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    let bloom = Arc::new(Mutex::new(BloomFilter::new(1 << 20, 4)));
                    let reachable_builder = ReachableBuilder::new((*store).clone(), bloom);
                    let marker = AtomicBloomFilterMarker::new(1 << 20, 4);
                    let count = reachable_builder
                        .build_with_marker_parallel(vec![root_hash], 4, &marker, batch_size)
                        .unwrap();
                    black_box(count);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark AtomicBloomFilter vs Mutex BloomFilter
fn bench_atomic_bloom_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("atomic_bloom_comparison");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    // Setup test data - 50K nodes for faster comparison
    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let store = Arc::new(store);
    let builder = TreeBuilder::new(store.clone());
    let (root_hash, _all_hashes) = builder.create_tree(50_000).unwrap();

    // Test AtomicBloomFilterMarker
    group.bench_function("atomic_bloom_marker", |b| {
        b.iter(|| {
            let bloom = Arc::new(Mutex::new(BloomFilter::new(1 << 20, 4)));
            let reachable_builder = ReachableBuilder::new((*store).clone(), bloom);
            let marker = AtomicBloomFilterMarker::new(1 << 20, 4);
            let count = reachable_builder
                .build_with_marker_parallel(vec![root_hash], 4, &marker, 1000)
                .unwrap();
            black_box(count);
        });
    });

    // Test BloomFilterMarker (with Mutex)
    group.bench_function("mutex_bloom_marker", |b| {
        b.iter(|| {
            let bloom = Arc::new(Mutex::new(BloomFilter::new(1 << 20, 4)));
            let reachable_builder = ReachableBuilder::new((*store).clone(), bloom);
            let marker = BloomFilterMarker::with_estimated_nodes(50_000, 0.01);
            let count = reachable_builder
                .build_with_marker_parallel(vec![root_hash], 4, &marker, 1000)
                .unwrap();
            black_box(count);
        });
    });

    group.finish();
}

/// Benchmark multi-root scenarios
fn bench_mark_phase_multi_root(c: &mut Criterion) {
    let mut group = c.benchmark_group("mark_phase_multi_root");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    // Setup test data - create 4 separate trees with 10K nodes each
    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let store = Arc::new(store);
    let builder = TreeBuilder::new(store.clone());
    let trees = builder.create_multiple_trees(4, 10_000).unwrap();
    let roots: Vec<_> = trees.iter().map(|(root, _)| *root).collect();

    // Benchmark single-threaded
    group.bench_function("single_thread_4_roots", |b| {
        b.iter(|| {
            let bloom = Arc::new(Mutex::new(BloomFilter::new(1 << 20, 4)));
            let reachable_builder = ReachableBuilder::new((*store).clone(), bloom);
            let marker = AtomicBloomFilterMarker::new(1 << 20, 4);
            let count = reachable_builder
                .build_with_marker(roots.clone(), &marker, 1000)
                .unwrap();
            black_box(count);
        });
    });

    // Benchmark parallel with 4 workers (one per root)
    group.bench_function("parallel_4_workers_4_roots", |b| {
        b.iter(|| {
            let bloom = Arc::new(Mutex::new(BloomFilter::new(1 << 20, 4)));
            let reachable_builder = ReachableBuilder::new((*store).clone(), bloom);
            let marker = AtomicBloomFilterMarker::new(1 << 20, 4);
            let count = reachable_builder
                .build_with_marker_parallel(roots.clone(), 4, &marker, 1000)
                .unwrap();
            black_box(count);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_mark_phase_scaling,
    bench_mark_phase_workers,
    bench_mark_phase_batch_size,
    bench_atomic_bloom_comparison,
    bench_mark_phase_multi_root
);

criterion_main!(benches);
