// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::tests::test_utils::TreeBuilder;
use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use std::sync::Arc;

#[tokio::test]
async fn test_refcount_default_value_handling() {
    // Test that None refcount is correctly treated as 0
    // This addresses the bug where None refcounts weren't properly handled

    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let store = Arc::new(store);
    let tree_builder = TreeBuilder::new(store.clone());

    // Create a simple tree structure
    let (_root_hash, all_node_hashes) = tree_builder.create_root_and_leaves(3);

    // Test node with no refcount (should default to 0)
    let test_node = all_node_hashes[0];

    // Verify initial refcount is None (treated as 0)
    let initial_refcount = store
        .get_prune_store()
        .get_node_refcount(test_node)
        .unwrap();
    assert_eq!(
        initial_refcount, None,
        "New node should have None refcount (treated as 0)"
    );

    // Increment refcount to 1
    store
        .get_prune_store()
        .inc_node_refcount(test_node)
        .unwrap();
    let refcount_after_inc = store
        .get_prune_store()
        .get_node_refcount(test_node)
        .unwrap();
    assert_eq!(
        refcount_after_inc,
        Some(1),
        "Refcount should be 1 after increment"
    );

    // Decrement refcount back to 0 (becomes None)
    store
        .get_prune_store()
        .dec_node_refcount(test_node)
        .unwrap();
    let refcount_after_dec = store
        .get_prune_store()
        .get_node_refcount(test_node)
        .unwrap();
    assert_eq!(
        refcount_after_dec, None,
        "Refcount should be None after decrement to 0"
    );

    // Decrement again - should handle gracefully (not go negative)
    // This tests the fix for proper refcount boundary handling
    let result = store.get_prune_store().dec_node_refcount(test_node);
    assert!(
        result.is_ok(),
        "Decrementing None refcount should not error"
    );

    let final_refcount = store
        .get_prune_store()
        .get_node_refcount(test_node)
        .unwrap();
    assert_eq!(
        final_refcount, None,
        "Refcount should remain None, not go negative"
    );

    println!("✅ Refcount default value handling test PASSED");
}

#[tokio::test]
async fn test_stale_index_timestamp_semantics() {
    // Test stale index timestamp semantics fix
    // This addresses the bug where timestamp comparison logic was incorrect

    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let store = Arc::new(store);
    let tree_builder = TreeBuilder::new(store.clone());

    // Create simple tree
    let (_root_hash, all_node_hashes) = tree_builder.create_root_and_leaves(2);

    // Test stale index with different tx_orders
    let tx_order_1 = 100;
    let tx_order_2 = 200;
    let _tx_order_3 = 300;

    // Add stale indices with known orders
    tree_builder
        .inject_stale_indices(vec![all_node_hashes[0]], tx_order_1)
        .unwrap();
    tree_builder
        .inject_stale_indices(vec![all_node_hashes[1]], tx_order_2)
        .unwrap();

    // Verify timestamp conversion and storage
    let stale_before_150 = store.get_prune_store().list_before(150, 1000).unwrap();

    let stale_before_250 = store.get_prune_store().list_before(250, 1000).unwrap();

    let stale_before_350 = store.get_prune_store().list_before(350, 1000).unwrap();

    // Verify correct number of stale nodes for each cutoff
    assert_eq!(
        stale_before_150.len(),
        1,
        "Should have 1 stale node before 150"
    );
    assert_eq!(
        stale_before_250.len(),
        2,
        "Should have 2 stale nodes before 250"
    );
    assert_eq!(
        stale_before_350.len(),
        2,
        "Should have 2 stale nodes before 350"
    );

    // Verify timestamp encoding: tx_order -> H256::from_low_u64_be(tx_order)
    for (timestamp, hash) in &stale_before_150 {
        let expected_timestamp = H256::from_low_u64_be(tx_order_1);
        assert_eq!(
            *timestamp, expected_timestamp,
            "Timestamp should be H256::from_low_u64_be(tx_order)"
        );
        assert_eq!(*hash, all_node_hashes[0], "Should be the first node");
    }

    // Verify timestamp ordering is preserved
    let timestamps: Vec<H256> = stale_before_350
        .iter()
        .map(|(timestamp, _)| *timestamp)
        .collect();

    // Timestamps should be sortable and correspond to tx_orders
    let timestamp_1 = H256::from_low_u64_be(tx_order_1);
    let timestamp_2 = H256::from_low_u64_be(tx_order_2);

    assert!(
        timestamps.contains(&timestamp_1),
        "Should contain timestamp for tx_order_1"
    );
    assert!(
        timestamps.contains(&timestamp_2),
        "Should contain timestamp for tx_order_2"
    );

    println!("✅ Stale index timestamp semantics test PASSED");
}

#[tokio::test]
async fn test_stale_index_query_edge_cases() {
    // Test edge cases in stale index queries

    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let store = Arc::new(store);
    let tree_builder = TreeBuilder::new(store.clone());

    // Create tree
    let (_root_hash, all_node_hashes) = tree_builder.create_root_and_leaves(2);

    // Test empty query (no stale nodes)
    let empty_query = store.get_prune_store().list_before(100, 1000).unwrap();
    assert_eq!(empty_query.len(), 0, "Empty query should return no results");

    // Add some stale nodes with known timestamps
    tree_builder
        .inject_stale_indices(vec![all_node_hashes[0]], 50)
        .unwrap();
    tree_builder
        .inject_stale_indices(vec![all_node_hashes[1]], 150)
        .unwrap();

    // Test boundary conditions
    let before_0 = store.get_prune_store().list_before(0, 1000).unwrap();
    assert_eq!(before_0.len(), 0, "Should have no nodes before tx_order 0");

    let before_50 = store.get_prune_store().list_before(50, 1000).unwrap();
    assert_eq!(
        before_50.len(),
        0,
        "Should have no nodes before tx_order 50 (strict inequality)"
    );

    let before_51 = store.get_prune_store().list_before(51, 1000).unwrap();
    assert_eq!(before_51.len(), 1, "Should have 1 node before tx_order 51");

    let before_200 = store.get_prune_store().list_before(200, 1000).unwrap();
    assert_eq!(
        before_200.len(),
        2,
        "Should have 2 nodes before tx_order 200"
    );

    // Verify timestamp ordering and node identification
    let expected_timestamp_1 = H256::from_low_u64_be(50);
    let expected_timestamp_2 = H256::from_low_u64_be(150);

    let timestamps: Vec<H256> = before_200.iter().map(|(timestamp, _)| *timestamp).collect();
    let nodes: Vec<H256> = before_200.iter().map(|(_, node)| *node).collect();

    assert!(
        timestamps.contains(&expected_timestamp_1),
        "Should contain timestamp 50"
    );
    assert!(
        timestamps.contains(&expected_timestamp_2),
        "Should contain timestamp 150"
    );
    assert!(
        nodes.contains(&all_node_hashes[0]),
        "Should contain first node"
    );
    assert!(
        nodes.contains(&all_node_hashes[1]),
        "Should contain second node"
    );

    println!("✅ Stale index query edge cases test PASSED");
}
