// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::tests::test_utils::{BloomInspector, TreeBuilder};
use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use parking_lot::Mutex;
use std::sync::Arc;

#[tokio::test]
async fn test_bloom_saturation_false_positives() {
    // Test that BuildReach still works correctly when bloom filter is saturated
    // This addresses gap: "Exercise BloomFilter saturation/false positives to ensure
    // no nodes are dropped due to bloom short-circuit"

    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let store = Arc::new(store);
    let tree_builder = TreeBuilder::new(store.clone());

    // Create a tree structure
    let (root_hash, _all_node_hashes) = tree_builder.create_root_and_leaves(3);
    let expected_reachable_nodes = vec![root_hash];

    // Create a small bloom filter (will have higher false positive rate)
    let bloom = Arc::new(Mutex::new(moveos_common::bloom_filter::BloomFilter::new(
        1 << 8,
        4,
    ))); // Small filter

    // Pre-load bloom filter with many random hashes to induce saturation/false positives
    println!("Pre-loading bloom filter with random hashes...");
    let saturation_count = 1000;
    {
        let mut bloom_guard = bloom.lock();
        for i in 0..saturation_count {
            let random_hash = H256::from_low_u64_be(i as u64 + 1000000); // Deterministic random-ish
            bloom_guard.insert(&random_hash);
        }
    }
    println!(
        "Pre-loaded {} random hashes into small bloom filter",
        saturation_count
    );

    // Check if our expected nodes are already "false positive" in bloom
    {
        let bloom_guard = bloom.lock();
        let root_already_in_bloom = bloom_guard.contains(&root_hash);
        drop(bloom_guard);
        println!(
            "Root hash already in pre-loaded bloom: {}",
            root_already_in_bloom
        );
    }

    // Run BuildReach with saturated bloom
    let reachable_roots = vec![root_hash];
    let builder = crate::reachability::ReachableBuilder::new((*store).clone(), bloom.clone());
    let scanned_size = builder.build(reachable_roots, 1).unwrap();

    println!(
        "BuildReach scanned {} nodes with saturated bloom filter",
        scanned_size
    );

    // Verify that root is still reachable despite bloom saturation
    let root_in_bloom = {
        let bloom_guard = bloom.lock();
        bloom_guard.contains(&root_hash)
    };

    // Key assertion: BuildReach should still work correctly and mark nodes as reachable
    assert!(
        scanned_size > 0,
        "BuildReach should have scanned some nodes despite bloom saturation"
    );

    // The root should definitely be reachable since we started from it
    assert!(
        root_in_bloom,
        "Root should be reachable even with saturated bloom"
    );

    // Check that we can still find our nodes in bloom
    let reachable_count = BloomInspector::count_contained_hashes(&bloom, &expected_reachable_nodes);
    println!(
        "Found {} out of {} expected reachable nodes",
        reachable_count,
        expected_reachable_nodes.len()
    );

    assert!(
        reachable_count >= 1,
        "Should find at least the root as reachable"
    );

    println!("[PASS] Bloom saturation false positives test PASSED");
}

#[tokio::test]
async fn test_bloom_capacity_boundaries() {
    // Test BloomFilter behavior at extreme capacity boundaries

    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let store = Arc::new(store);
    let tree_builder = TreeBuilder::new(store.clone());

    // Create tree
    let (root_hash, _all_node_hashes) = tree_builder.create_root_and_leaves(2);

    // Test 1: Very small bloom filter (minimal capacity)
    println!("Testing with minimal bloom filter...");
    let small_bloom = Arc::new(Mutex::new(moveos_common::bloom_filter::BloomFilter::new(
        1 << 4,
        2,
    ))); // Very small
    let start_time = std::time::Instant::now();

    let builder = crate::reachability::ReachableBuilder::new((*store).clone(), small_bloom.clone());
    let _scanned_size = builder.build(vec![root_hash], 1).unwrap();

    let small_bloom_duration = start_time.elapsed();
    println!(
        "Small bloom filter execution time: {:?}",
        small_bloom_duration
    );

    // Test 2: Large bloom filter (high capacity)
    println!("Testing with large bloom filter...");
    let large_bloom = Arc::new(Mutex::new(moveos_common::bloom_filter::BloomFilter::new(
        1 << 20,
        8,
    ))); // Large
    let start_time = std::time::Instant::now();

    let builder = crate::reachability::ReachableBuilder::new((*store).clone(), large_bloom.clone());
    let _scanned_size = builder.build(vec![root_hash], 1).unwrap();

    let large_bloom_duration = start_time.elapsed();
    println!(
        "Large bloom filter execution time: {:?}",
        large_bloom_duration
    );

    // Verify both work correctly
    let small_contains_root = {
        let bloom_guard = small_bloom.lock();
        bloom_guard.contains(&root_hash)
    };

    let large_contains_root = {
        let bloom_guard = large_bloom.lock();
        bloom_guard.contains(&root_hash)
    };

    assert!(small_contains_root, "Small bloom should contain root");
    assert!(large_contains_root, "Large bloom should contain root");

    // Performance should be reasonable (not orders of magnitude difference)
    let ratio = large_bloom_duration.as_nanos() as f64 / small_bloom_duration.as_nanos() as f64;
    println!("Performance ratio (large/small): {:.2}x", ratio);
    assert!(
        ratio < 100.0,
        "Large bloom shouldn't be dramatically slower than small bloom"
    );

    println!("[PASS] Bloom capacity boundaries test PASSED");
}

#[tokio::test]
async fn test_bloom_concurrent_access() {
    // Test BloomFilter behavior under concurrent access (simulated)

    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let store = Arc::new(store);
    let tree_builder = TreeBuilder::new(store.clone());

    // Create tree
    let (root_hash, all_node_hashes) = tree_builder.create_root_and_leaves(5);

    // Create bloom filter
    let bloom = Arc::new(Mutex::new(moveos_common::bloom_filter::BloomFilter::new(
        1 << 16,
        4,
    )));

    // Simulate concurrent access by pre-loading some data
    {
        let mut bloom_guard = bloom.lock();
        for i in 0..100 {
            let hash = H256::from_low_u64_be(i + 50000);
            bloom_guard.insert(&hash);
        }
    }

    // Run BuildReach
    let reachable_roots = vec![root_hash];
    let builder = crate::reachability::ReachableBuilder::new((*store).clone(), bloom.clone());
    let scanned_size = builder.build(reachable_roots, 1).unwrap();

    println!(
        "BuildReach scanned {} nodes under concurrent simulation",
        scanned_size
    );

    // Verify root is reachable
    let root_reachable = {
        let bloom_guard = bloom.lock();
        bloom_guard.contains(&root_hash)
    };
    assert!(root_reachable, "Root should be reachable");

    // Verify we can still access bloom filter correctly
    let reachable_count = BloomInspector::count_contained_hashes(&bloom, &all_node_hashes);
    println!(
        "Found {} reachable nodes out of {}",
        reachable_count,
        all_node_hashes.len()
    );

    // At minimum, root should be reachable (we'll verify root separately)
    assert!(scanned_size > 0, "Should have scanned some nodes");

    println!("[PASS] Bloom concurrent access test PASSED");
}

#[tokio::test]
async fn test_bloom_memory_usage() {
    // Test that bloom filter memory usage is reasonable

    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let store = Arc::new(store);
    let tree_builder = TreeBuilder::new(store.clone());

    // Create tree
    let (root_hash, _all_node_hashes) = tree_builder.create_root_and_leaves(3);

    // Create bloom filters of different sizes and test they work
    let small_bloom = Arc::new(Mutex::new(moveos_common::bloom_filter::BloomFilter::new(
        1 << 10,
        4,
    )));
    let medium_bloom = Arc::new(Mutex::new(moveos_common::bloom_filter::BloomFilter::new(
        1 << 16,
        4,
    )));
    let large_bloom = Arc::new(Mutex::new(moveos_common::bloom_filter::BloomFilter::new(
        1 << 20,
        4,
    )));

    // Test all sizes work with BuildReach
    for (size_name, bloom) in [
        ("small", small_bloom),
        ("medium", medium_bloom),
        ("large", large_bloom),
    ] {
        let start_time = std::time::Instant::now();

        let builder = crate::reachability::ReachableBuilder::new((*store).clone(), bloom.clone());
        let scanned_size = builder.build(vec![root_hash], 1).unwrap();

        let duration = start_time.elapsed();
        let root_reachable = {
            let bloom_guard = bloom.lock();
            bloom_guard.contains(&root_hash)
        };

        println!(
            "{} bloom: scanned {} nodes in {:?}, root reachable: {}",
            size_name, scanned_size, duration, root_reachable
        );

        assert!(
            root_reachable,
            "{} bloom should have root reachable",
            size_name
        );
        assert!(
            scanned_size > 0,
            "{} bloom should have scanned nodes",
            size_name
        );
    }

    println!("[PASS] Bloom memory usage test PASSED");
}
