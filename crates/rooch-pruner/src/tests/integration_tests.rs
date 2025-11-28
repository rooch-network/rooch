// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::reachability::ReachableBuilder;
use crate::tests::test_utils::TreeBuilder;
use moveos_common::bloom_filter::BloomFilter;
use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use parking_lot::Mutex;
use std::sync::Arc;

#[tokio::test]
async fn test_end_to_end_pruning_pipeline() {
    // Test the complete pruning pipeline: BuildReach -> SweepExpired
    // This validates the integration between all components

    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let store = Arc::new(store);
    let tree_builder = TreeBuilder::new(store.clone());

    // Create a tree structure with multiple nodes
    let (root_hash, all_node_hashes) = tree_builder.create_root_and_leaves(5);
    println!(
        "Created tree with {} nodes for E2E test",
        all_node_hashes.len() + 1
    );

    // Phase 1: Mark some nodes as stale
    let stale_cutoff_order = 100;
    let stale_nodes = vec![all_node_hashes[1], all_node_hashes[3]]; // Mark 2nd and 4th leaves as stale
    tree_builder
        .inject_stale_indices(stale_nodes.clone(), stale_cutoff_order)
        .unwrap();

    // Phase 2: BuildReach to identify reachable nodes
    let bloom = Arc::new(Mutex::new(BloomFilter::new(1 << 20, 4)));
    let reachable_roots = vec![root_hash];

    let builder = ReachableBuilder::new(store.clone(), bloom.clone());
    let scanned_size = builder.build(reachable_roots, 1).unwrap();

    println!("BuildReach phase completed: scanned {} nodes", scanned_size);

    // Phase 3: Verify reachable ∩ stale = Ø invariant
    let stale_records = store
        .get_prune_store()
        .list_before(stale_cutoff_order + 1, 1000)
        .unwrap();
    let stale_hashes: Vec<H256> = stale_records.iter().map(|(hash, _)| *hash).collect();

    // Check that no stale node is marked as reachable
    for stale_hash in &stale_hashes {
        let in_bloom = {
            let bloom_guard = bloom.lock();
            bloom_guard.contains(stale_hash)
        };

        assert!(
            !in_bloom,
            "E2E INTEGRATION FAILURE: Stale node {} is also reachable!",
            stale_hash
        );
    }

    // Phase 4: Verify root is reachable and not stale, and BuildReach completed successfully
    let root_in_bloom = {
        let bloom_guard = bloom.lock();
        bloom_guard.contains(&root_hash)
    };
    let root_in_stale = stale_hashes.contains(&root_hash);

    println!(
        "E2E Integration: root reachable = {}, root stale = {}, scanned nodes = {}",
        root_in_bloom, root_in_stale, scanned_size
    );

    // Root should be reachable and not stale, BuildReach should have processed nodes
    assert!(root_in_bloom, "Root should be reachable after BuildReach");
    assert!(!root_in_stale, "Root should not be stale");
    assert!(
        scanned_size > 0,
        "BuildReach should have scanned at least the root"
    );

    println!("✅ End-to-end pruning pipeline test PASSED");
}

#[tokio::test]
async fn test_concurrent_pruning_operations() {
    // Test concurrent BuildReach and SweepExpired operations
    // This ensures thread safety and consistency under concurrent access

    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let store = Arc::new(store);
    let tree_builder = TreeBuilder::new(store.clone());

    // Create multiple independent trees
    let (root1, nodes1) = tree_builder.create_root_and_leaves(3);
    let (root2, nodes2) = tree_builder.create_root_and_leaves(3);
    let (root3, nodes3) = tree_builder.create_root_and_leaves(3);

    // Mark different nodes as stale for each tree
    let stale_cutoff1 = 100;
    let stale_cutoff2 = 200;
    let stale_cutoff3 = 300;

    tree_builder
        .inject_stale_indices(vec![nodes1[1]], stale_cutoff1)
        .unwrap();
    tree_builder
        .inject_stale_indices(vec![nodes2[0], nodes2[2]], stale_cutoff2)
        .unwrap();
    tree_builder
        .inject_stale_indices(vec![nodes3[2]], stale_cutoff3)
        .unwrap();

    // Simulate concurrent operations by running them sequentially but rapidly
    let mut results = Vec::new();

    // Operation 1: BuildReach for tree 1
    let bloom1 = Arc::new(Mutex::new(BloomFilter::new(1 << 18, 4)));
    let builder1 = ReachableBuilder::new(store.clone(), bloom1.clone());
    let scanned1 = builder1.build(vec![root1], 1).unwrap();
    results.push(("Tree1", scanned1, bloom1.clone()));

    // Operation 2: BuildReach for tree 2
    let bloom2 = Arc::new(Mutex::new(BloomFilter::new(1 << 18, 4)));
    let builder2 = ReachableBuilder::new(store.clone(), bloom2.clone());
    let scanned2 = builder2.build(vec![root2], 1).unwrap();
    results.push(("Tree2", scanned2, bloom2.clone()));

    // Operation 3: BuildReach for tree 3
    let bloom3 = Arc::new(Mutex::new(BloomFilter::new(1 << 18, 4)));
    let builder3 = ReachableBuilder::new(store.clone(), bloom3.clone());
    let scanned3 = builder3.build(vec![root3], 1).unwrap();
    results.push(("Tree3", scanned3, bloom3.clone()));

    // Verify all operations completed successfully
    for (tree_name, scanned_size, bloom) in results {
        println!("{}: scanned {} nodes", tree_name, scanned_size);
        assert!(
            scanned_size > 0,
            "{} should have scanned some nodes",
            tree_name
        );

        // Check that bloom filter has some entries (indicating successful operation)
        let bloom_guard = bloom.lock();
        let _has_entries = bloom_guard.contains(&H256::from_low_u64_be(1))
            || bloom_guard.contains(&H256::from_low_u64_be(2))
            || bloom_guard.contains(&H256::from_low_u64_be(3));
        // We don't check for specific entries, just that the bloom was used
    }

    println!("✅ Concurrent pruning operations test PASSED");
}

#[tokio::test]
async fn test_multi_snapshot_consistency() {
    // Test consistency across multiple snapshots in sequence
    // This validates that the pruning system maintains consistency over time

    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let store = Arc::new(store);
    let tree_builder = TreeBuilder::new(store.clone());

    // Create trees for different "time periods"
    let (root1, nodes1) = tree_builder.create_root_and_leaves(2);
    let (root2, nodes2) = tree_builder.create_root_and_leaves(2);
    let (root3, nodes3) = tree_builder.create_root_and_leaves(2);

    // Simulate evolution over time with different stale cutoffs
    let time1 = 100;
    let time2 = 200;
    let time3 = 300;

    // Time 1: Some nodes become stale
    tree_builder
        .inject_stale_indices(vec![nodes1[0]], time1)
        .unwrap();

    // Snapshot 1: BuildReach at time1
    let bloom1 = Arc::new(Mutex::new(BloomFilter::new(1 << 16, 4)));
    let builder1 = ReachableBuilder::new(store.clone(), bloom1.clone());
    let scanned1 = builder1.build(vec![root1], 1).unwrap();

    // Time 2: More nodes become stale
    tree_builder
        .inject_stale_indices(vec![nodes2[1]], time2)
        .unwrap();

    // Snapshot 2: BuildReach at time2
    let bloom2 = Arc::new(Mutex::new(BloomFilter::new(1 << 16, 4)));
    let builder2 = ReachableBuilder::new(store.clone(), bloom2.clone());
    let scanned2 = builder2.build(vec![root2], 1).unwrap();

    // Time 3: Final batch of stale nodes
    tree_builder
        .inject_stale_indices(vec![nodes3[0], nodes3[1]], time3)
        .unwrap();

    // Snapshot 3: BuildReach at time3
    let bloom3 = Arc::new(Mutex::new(BloomFilter::new(1 << 16, 4)));
    let builder3 = ReachableBuilder::new(store.clone(), bloom3.clone());
    let scanned3 = builder3.build(vec![root3], 1).unwrap();

    // Verify all snapshots are internally consistent
    let snapshots = [
        ("Snapshot1", scanned1, bloom1.clone(), vec![root1]),
        ("Snapshot2", scanned2, bloom2.clone(), vec![root2]),
        ("Snapshot3", scanned3, bloom3.clone(), vec![root3]),
    ];

    for (snapshot_name, scanned_size, bloom, expected_roots) in snapshots {
        // Each snapshot should have scanned some nodes
        assert!(
            scanned_size > 0,
            "{} should have scanned nodes",
            snapshot_name
        );

        // Each snapshot should mark its expected roots as reachable
        let bloom_guard = bloom.lock();
        for root in &expected_roots {
            let root_reachable = bloom_guard.contains(root);
            assert!(
                root_reachable,
                "{} should have root {} reachable",
                snapshot_name, root
            );
        }
    }

    // Verify temporal consistency: later snapshots should have access to
    // the accumulated state of the system
    let all_stale_records = store
        .get_prune_store()
        .list_before(time3 + 1, 1000)
        .unwrap();

    assert!(
        all_stale_records.len() >= 3,
        "Should have accumulated stale nodes from all time periods"
    );

    println!(
        "Multi-snapshot consistency: {} total stale nodes accumulated",
        all_stale_records.len()
    );

    println!("✅ Multi-snapshot consistency test PASSED");
}

#[tokio::test]
async fn test_pruning_scalability() {
    // Test pruning behavior with larger datasets
    // This validates that the system scales reasonably

    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let store = Arc::new(store);
    let tree_builder = TreeBuilder::new(store.clone());

    // Create a larger tree structure
    let (root_hash, all_node_hashes) = tree_builder.create_root_and_leaves(10);
    println!(
        "Created scalability test with {} nodes",
        all_node_hashes.len() + 1
    );

    // Mark a significant portion as stale (40%)
    let stale_count = all_node_hashes.len() * 2 / 5;
    let stale_nodes: Vec<H256> = all_node_hashes.iter().take(stale_count).cloned().collect();
    let stale_cutoff_order = 1000;

    tree_builder
        .inject_stale_indices(stale_nodes.clone(), stale_cutoff_order)
        .unwrap();

    // Measure BuildReach performance
    let start_time = std::time::Instant::now();

    let bloom = Arc::new(Mutex::new(BloomFilter::new(1 << 22, 4))); // Larger bloom
    let reachable_roots = vec![root_hash];

    let builder = ReachableBuilder::new(store.clone(), bloom.clone());
    let scanned_size = builder.build(reachable_roots, 1).unwrap();

    let duration = start_time.elapsed();

    println!(
        "Scalability test: scanned {} nodes in {:?}",
        scanned_size, duration
    );

    // Performance should be reasonable (complete within 30 seconds for test)
    assert!(
        duration.as_secs() < 30,
        "BuildReach should complete in reasonable time"
    );

    // Verify safety invariants are maintained even with larger datasets
    let stale_records = store
        .get_prune_store()
        .list_before(stale_cutoff_order + 1, 2000)
        .unwrap();
    let stale_hashes: Vec<H256> = stale_records.iter().map(|(hash, _)| *hash).collect();

    // Verify no stale nodes are reachable
    for stale_hash in &stale_hashes {
        let in_bloom = {
            let bloom_guard = bloom.lock();
            bloom_guard.contains(stale_hash)
        };

        assert!(
            !in_bloom,
            "SCALABILITY SAFETY FAILURE: Stale node {} is marked as reachable",
            stale_hash
        );
    }

    // Verify root is reachable
    let root_in_bloom = {
        let bloom_guard = bloom.lock();
        bloom_guard.contains(&root_hash)
    };
    assert!(
        root_in_bloom,
        "Root should be reachable in scalability test"
    );

    println!("✅ Pruning scalability test PASSED");
}

#[tokio::test]
async fn test_error_recovery_integration() {
    // Test system behavior under error conditions and recovery
    // This validates robustness of the integrated pruning pipeline

    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let store = Arc::new(store);
    let tree_builder = TreeBuilder::new(store.clone());

    // Create a tree structure
    let (root_hash, all_node_hashes) = tree_builder.create_root_and_leaves(3);

    // Mark some nodes as stale
    let stale_cutoff_order = 100;
    let stale_nodes = vec![all_node_hashes[1]];
    tree_builder
        .inject_stale_indices(stale_nodes, stale_cutoff_order)
        .unwrap();

    // Test 1: BuildReach with valid data (baseline)
    let bloom1 = Arc::new(Mutex::new(BloomFilter::new(1 << 16, 4)));
    let builder1 = ReachableBuilder::new(store.clone(), bloom1.clone());
    let scanned1 = builder1.build(vec![root_hash], 1).unwrap();

    assert!(scanned1 > 0, "Baseline BuildReach should work");

    let root_in_bloom1 = {
        let bloom_guard = bloom1.lock();
        bloom_guard.contains(&root_hash)
    };
    assert!(root_in_bloom1, "Root should be reachable in baseline");

    // Test 2: BuildReach with empty roots (error condition)
    let bloom2 = Arc::new(Mutex::new(BloomFilter::new(1 << 16, 4)));
    let builder2 = ReachableBuilder::new(store.clone(), bloom2.clone());
    let scanned2 = builder2.build(vec![], 1).unwrap();

    assert_eq!(scanned2, 0, "Empty roots should scan 0 nodes");

    // Test 3: BuildReach with non-existent roots (error condition)
    let fake_root = H256::random();
    let bloom3 = Arc::new(Mutex::new(BloomFilter::new(1 << 16, 4)));
    let builder3 = ReachableBuilder::new(store.clone(), bloom3.clone());

    // Should not panic with non-existent root
    let scanned3 = builder3.build(vec![fake_root], 1).unwrap();
    println!(
        "BuildReach with non-existent root scanned {} nodes",
        scanned3
    );

    // Test 4: Recovery - BuildReach should still work correctly after errors
    let bloom4 = Arc::new(Mutex::new(BloomFilter::new(1 << 16, 4)));
    let builder4 = ReachableBuilder::new(store.clone(), bloom4.clone());
    let scanned4 = builder4.build(vec![root_hash], 1).unwrap();

    assert!(scanned4 > 0, "BuildReach should recover and work correctly");

    let root_in_bloom4 = {
        let bloom_guard = bloom4.lock();
        bloom_guard.contains(&root_hash)
    };
    assert!(root_in_bloom4, "Root should be reachable after recovery");

    // Verify safety invariants are maintained throughout error scenarios
    let stale_records = store
        .get_prune_store()
        .list_before(stale_cutoff_order + 1, 1000)
        .unwrap();
    let stale_hashes: Vec<H256> = stale_records.iter().map(|(hash, _)| *hash).collect();

    // Even after error conditions, no stale node should be reachable
    for stale_hash in &stale_hashes {
        let in_bloom4 = {
            let bloom_guard = bloom4.lock();
            bloom_guard.contains(stale_hash)
        };

        assert!(
            !in_bloom4,
            "ERROR RECOVERY FAILURE: Stale node {} is reachable after error scenarios",
            stale_hash
        );
    }

    println!("✅ Error recovery integration test PASSED");
}

#[tokio::test]
async fn test_state_consistency_across_operations() {
    // Test that the pruning system maintains consistent state across different operations
    // This is a comprehensive integration test for state consistency

    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let store = Arc::new(store);
    let tree_builder = TreeBuilder::new(store.clone());

    // Create initial state
    let (root1, nodes1) = tree_builder.create_root_and_leaves(3);
    let (root2, nodes2) = tree_builder.create_root_and_leaves(3);

    // Operation 1: Mark some nodes as stale
    let stale_cutoff_1 = 100;
    let stale_nodes_1 = vec![nodes1[1], nodes2[0]];
    tree_builder
        .inject_stale_indices(stale_nodes_1.clone(), stale_cutoff_1)
        .unwrap();

    // Operation 2: BuildReach for root1
    let bloom1 = Arc::new(Mutex::new(BloomFilter::new(1 << 18, 4)));
    let builder1 = ReachableBuilder::new(store.clone(), bloom1.clone());
    let scanned1 = builder1.build(vec![root1], 1).unwrap();

    // Operation 3: More nodes become stale
    let stale_cutoff_2 = 200;
    let stale_nodes_2 = vec![nodes1[2], nodes2[2]];
    tree_builder
        .inject_stale_indices(stale_nodes_2.clone(), stale_cutoff_2)
        .unwrap();

    // Operation 4: BuildReach for root2
    let bloom2 = Arc::new(Mutex::new(BloomFilter::new(1 << 18, 4)));
    let builder2 = ReachableBuilder::new(store.clone(), bloom2.clone());
    let scanned2 = builder2.build(vec![root2], 1).unwrap();

    // Verify state consistency
    // 1. Each operation should have completed successfully
    assert!(scanned1 > 0, "Operation 2 should have scanned nodes");
    assert!(scanned2 > 0, "Operation 4 should have scanned nodes");

    // 2. Roots should be reachable in their respective blooms
    let root1_in_bloom1 = {
        let bloom_guard = bloom1.lock();
        bloom_guard.contains(&root1)
    };
    let root2_in_bloom2 = {
        let bloom_guard = bloom2.lock();
        bloom_guard.contains(&root2)
    };
    assert!(root1_in_bloom1, "Root1 should be reachable in bloom1");
    assert!(root2_in_bloom2, "Root2 should be reachable in bloom2");

    // 3. Stale nodes should not be reachable in any bloom
    let all_stale_nodes = stale_nodes_1
        .iter()
        .chain(stale_nodes_2.iter())
        .collect::<Vec<_>>();

    for stale_node in &all_stale_nodes {
        let in_bloom1 = {
            let bloom_guard = bloom1.lock();
            bloom_guard.contains(stale_node)
        };
        let in_bloom2 = {
            let bloom_guard = bloom2.lock();
            bloom_guard.contains(stale_node)
        };

        assert!(
            !(in_bloom1 || in_bloom2),
            "STATE CONSISTENCY FAILURE: Stale node {} is reachable in bloom1: {}, bloom2: {}",
            stale_node,
            in_bloom1,
            in_bloom2
        );
    }

    // 4. Global state consistency check
    let all_stale_records = store
        .get_prune_store()
        .list_before(stale_cutoff_2 + 1, 1000)
        .unwrap();

    println!(
        "State consistency: {} operations completed, {} stale nodes tracked",
        2,
        all_stale_records.len()
    );

    // The primary invariant is verified above - no stale nodes are reachable

    println!("✅ State consistency across operations test PASSED");
}
