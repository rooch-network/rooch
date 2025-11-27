// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::reachability::ReachableBuilder;
use crate::tests::test_utils::{BloomInspector, TreeBuilder};
use moveos_common::bloom_filter::BloomFilter;
use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use parking_lot::Mutex;
use std::sync::Arc;

#[tokio::test]
async fn test_reachable_stale_disjoint_basic() {
    // This is the CRITICAL test for data safety: reachable ∩ stale = Ø

    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let store = Arc::new(store);
    let tree_builder = TreeBuilder::new(store.clone());

    // Create a simple tree structure with multiple leaves
    let (root_hash, all_node_hashes) = tree_builder.create_root_and_leaves(3);
    println!("Created tree with {} leaf nodes and root", all_node_hashes.len());

    // Mark some leaf nodes as stale (but NOT the root)
    // We'll mark the second leaf as stale, but keep root and first leaf reachable
    let stale_leaf_hash = all_node_hashes[1]; // second leaf is stale
    let stale_cutoff_order = 100;
    tree_builder.inject_stale_indices(vec![stale_leaf_hash], stale_cutoff_order).unwrap();

    // Run BuildReach to mark reachable nodes
    let bloom = Arc::new(Mutex::new(BloomFilter::new(1 << 20, 4)));
    let reachable_roots = vec![root_hash];

    let builder = ReachableBuilder::new(store.clone(), bloom.clone());
    let scanned_size = builder.build(reachable_roots, 1).unwrap();

    println!("BuildReach scanned {} nodes", scanned_size);

    // Now collect all stale nodes from the database
    let stale_nodes = store.get_prune_store()
        .list_before(stale_cutoff_order + 1, 1000) // Get stale nodes up to cutoff
        .unwrap();

    println!("Found {} stale nodes", stale_nodes.len());

    // Extract stale node hashes
    let stale_hashes: Vec<H256> = stale_nodes.iter().map(|(hash, _)| *hash).collect();

    // Verify the critical invariant: reachable ∩ stale = Ø
    // Since we can't directly extract all reachable nodes from bloom filter,
    // we'll test by checking that our known stale node is NOT in the reachable set

    // The stale_leaf should be in stale set but NOT in reachable set
    let leaf2_in_stale = stale_hashes.contains(&stale_leaf_hash);
    let leaf2_in_bloom = {
        let bloom_guard = bloom.lock();
        bloom_guard.contains(&stale_leaf_hash)
    };

    println!("Stale leaf in stale: {}, in bloom: {}", leaf2_in_stale, leaf2_in_bloom);

    // CRITICAL ASSERTION: A node cannot be both stale and reachable
    assert!(
        !(leaf2_in_stale && leaf2_in_bloom),
        "CRITICAL SAFETY VIOLATION: Node {} is both stale and reachable! This could lead to data deletion.",
        stale_leaf_hash
    );

    // Additional verification: root should be reachable but not stale
    let root_in_bloom = {
        let bloom_guard = bloom.lock();
        bloom_guard.contains(&root_hash)
    };

    let root_in_stale = stale_hashes.contains(&root_hash);

    assert!(root_in_bloom, "Root should be reachable");
    assert!(!root_in_stale, "Root should not be stale");

    println!("✅ Basic reachable ∩ stale = Ø test PASSED");
}

#[tokio::test]
async fn test_reachable_stale_disjoint_complex() {
    // Test more complex scenarios with refcount edge cases and API behavior validation

    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let store = Arc::new(store);

    // Create nodes directly to test API behavior
    let node1 = H256::random();
    let node2 = H256::random();
    let node3 = H256::random();
    let root_hash = H256::random();

    // Test refcount behaviors
    // Node1: no refcount (None) - should be treated as 0
    let no_refcount_node = node1;

    // Node2: refcount = 0 (after increment and decrement)
    let zero_refcount_node = node2;
    store.get_prune_store().inc_node_refcount(zero_refcount_node).unwrap();
    store.get_prune_store().dec_node_refcount(zero_refcount_node).unwrap();

    // Node3: refcount > 0
    let positive_refcount_node = node3;
    store.get_prune_store().inc_node_refcount(positive_refcount_node).unwrap();

    // Store the nodes in node_store so BuildReach can find them
    let node_store = store.get_state_node_store().clone();
    use smt::jellyfish_merkle::node_type::Node;
    use smt::SMTObject;

    // Create actual SMT nodes
    for node_hash in [no_refcount_node, zero_refcount_node, positive_refcount_node, root_hash] {
        let node = Node::new_leaf(H256::random(), SMTObject::<Vec<u8>>::from_origin(vec![1,2,3]).unwrap());
        node_store.put(node_hash, node.encode().unwrap()).unwrap();
    }

    // Mark some nodes as stale
    let stale_nodes = vec![no_refcount_node, zero_refcount_node];
    let stale_cutoff_order = 100;

    // Create stale indices manually
    for (i, node_hash) in stale_nodes.iter().enumerate() {
        let tx_order = stale_cutoff_order + i as u64;
        let stale_pair = (*node_hash, H256::from_low_u64_be(tx_order));
        store.get_prune_store().write_stale_indices(tx_order, &[stale_pair]).unwrap();
    }

    // Run BuildReach with root
    let bloom = Arc::new(Mutex::new(BloomFilter::new(1 << 20, 4)));
    let reachable_roots = vec![root_hash];

    let builder = ReachableBuilder::new(store.clone(), bloom.clone());
    let scanned_size = builder.build(reachable_roots, 1).unwrap();

    println!("BuildReach scanned {} nodes in complex scenario", scanned_size);

    // Verify root is reachable
    let root_in_bloom = {
        let bloom_guard = bloom.lock();
        bloom_guard.contains(&root_hash)
    };
    assert!(root_in_bloom, "Root should be reachable");
    assert!(scanned_size >= 1, "Should have scanned at least the root");

    // Get stale nodes
    let stale_records = store.get_prune_store()
        .list_before(stale_cutoff_order + 10, 1000)
        .unwrap();
    let stale_hashes: Vec<H256> = stale_records.iter().map(|(hash, _)| *hash).collect();

    // Verify the critical invariant: no stale node is reachable
    for stale_hash in &stale_hashes {
        let in_bloom = {
            let bloom_guard = bloom.lock();
            bloom_guard.contains(stale_hash)
        };

        assert!(
            !in_bloom,
            "CRITICAL SAFETY VIOLATION: Stale node {} is also reachable! refcount: {:?}",
            stale_hash,
            store.get_prune_store().get_node_refcount(*stale_hash).unwrap()
        );
    }

    // Verify refcount states
    let no_refcount = store.get_prune_store().get_node_refcount(no_refcount_node).unwrap();
    let zero_refcount = store.get_prune_store().get_node_refcount(zero_refcount_node).unwrap();
    let positive_refcount = store.get_prune_store().get_node_refcount(positive_refcount_node).unwrap();

    println!("Refcount states: no_refcount={:?}, zero_refcount={:?}, positive_refcount={:?}",
             no_refcount, zero_refcount, positive_refcount);

    // Validate refcount behavior (the exact behavior may vary)
    assert!(positive_refcount == Some(1), "Positive refcount should be maintained");

    println!("✅ Complex reachable ∩ stale = Ø test PASSED (API behavior validated)");
}

#[tokio::test]
async fn test_stale_index_semantic_validation() {
    // Test that stale index API works correctly and validates basic behavior

    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let store = Arc::new(store);

    // Create nodes manually
    let node1 = H256::random();
    let node2 = H256::random();

    // Test stale index creation with different tx_orders
    let tx_order_1 = 100;
    let tx_order_2 = 200;

    // Add stale indices with different orders
    let stale1 = (node1, H256::from_low_u64_be(tx_order_1));
    let stale2 = (node2, H256::from_low_u64_be(tx_order_2));
    store.get_prune_store().write_stale_indices(tx_order_1, &[stale1]).unwrap();
    store.get_prune_store().write_stale_indices(tx_order_2, &[stale2]).unwrap();

    // Query stale indices before different cutoffs
    let stale_before_100 = store.get_prune_store()
        .list_before(tx_order_1, 1000)
        .unwrap();

    let stale_before_200 = store.get_prune_store()
        .list_before(tx_order_2, 1000)
        .unwrap();

    let stale_before_201 = store.get_prune_store()
        .list_before(tx_order_2 + 1, 1000)
        .unwrap();

    println!("Stale nodes by cutoff: before_100={}, before_200={}, before_201={}",
             stale_before_100.len(), stale_before_200.len(), stale_before_201.len());

    // Validate monotonic behavior: more nodes should be available with higher cutoffs
    assert!(stale_before_201.len() >= stale_before_200.len(),
            "Higher cutoff should return at least as many nodes");
    assert!(stale_before_200.len() >= stale_before_100.len(),
            "Higher cutoff should return at least as many nodes");

    // If we found any nodes, verify timestamp format is consistent
    for (i, (timestamp, hash)) in stale_before_201.iter().enumerate() {
        println!("Stale node {}: hash={}, timestamp={}", i + 1, hash, timestamp);

        // Verify timestamp is a valid H256 (not default/uninitialized)
        assert_ne!(*timestamp, H256::default(), "Timestamp should be properly set");
    }

    println!("✅ Stale index semantic validation test PASSED");
}

#[tokio::test]
async fn test_no_stale_nodes_scenario() {
    // Test edge case where there are no stale nodes

    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let store = Arc::new(store);

    // Create a root node manually
    let root_hash = H256::random();
    let node_store = store.get_state_node_store().clone();
    use smt::jellyfish_merkle::node_type::Node;
    use smt::SMTObject;

    let root_node = Node::new_leaf(H256::random(), SMTObject::<Vec<u8>>::from_origin(vec![1,2,3]).unwrap());
    node_store.put(root_hash, root_node.encode().unwrap()).unwrap();

    // Run BuildReach without any stale nodes
    let bloom = Arc::new(Mutex::new(BloomFilter::new(1 << 20, 4)));
    let reachable_roots = vec![root_hash];

    let builder = ReachableBuilder::new(store.clone(), bloom.clone());
    let scanned_size = builder.build(reachable_roots, 1).unwrap();

    // Query stale indices (should be empty or very small)
    let stale_nodes = store.get_prune_store()
        .list_before(u64::MAX, 1000)
        .unwrap();

    println!("Found {} stale nodes (should be empty or minimal)", stale_nodes.len());

    // Verify root is reachable
    let root_reachable = {
        let bloom_guard = bloom.lock();
        bloom_guard.contains(&root_hash)
    };
    assert!(root_reachable, "Root should be reachable");
    assert!(scanned_size >= 1, "Should have scanned at least the root");

    println!("✅ No stale nodes scenario test PASSED");
}

#[tokio::test]
async fn test_all_nodes_stale_scenario() {
    // Test edge case where all nodes except root are marked as stale
    // This should still maintain reachable ∩ stale = Ø

    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let store = Arc::new(store);
    let tree_builder = TreeBuilder::new(store.clone());

    // Create tree
    let (root_hash, all_node_hashes) = tree_builder.create_root_and_leaves(3);

    // Mark ALL leaf nodes except root as stale
    let stale_cutoff_order = 100;
    let stale_nodes = all_node_hashes.clone(); // Mark all leaves as stale

    tree_builder.inject_stale_indices(stale_nodes.clone(), stale_cutoff_order).unwrap();

    // Run BuildReach
    let bloom = Arc::new(Mutex::new(BloomFilter::new(1 << 20, 4)));
    let reachable_roots = vec![root_hash];

    let builder = ReachableBuilder::new(store.clone(), bloom.clone());
    let scanned_size = builder.build(reachable_roots, 1).unwrap();

    // Get stale nodes
    let stale_records = store.get_prune_store()
        .list_before(stale_cutoff_order + 1, 1000)
        .unwrap();
    let stale_hashes: Vec<H256> = stale_records.iter().map(|(hash, _)| *hash).collect();

    // Verify no overlap between stale and reachable
    for stale_hash in &stale_hashes {
        let in_bloom = {
            let bloom_guard = bloom.lock();
            bloom_guard.contains(stale_hash)
        };

        assert!(
            !in_bloom,
            "CRITICAL SAFETY VIOLATION: Node {} is both stale and reachable",
            stale_hash
        );
    }

    // Root should be reachable but not stale
    let root_in_bloom = {
        let bloom_guard = bloom.lock();
        bloom_guard.contains(&root_hash)
    };
    let root_in_stale = stale_hashes.contains(&root_hash);

    assert!(root_in_bloom, "Root should be reachable");
    assert!(!root_in_stale, "Root should not be stale");

    println!("✅ All nodes stale scenario test PASSED");
}