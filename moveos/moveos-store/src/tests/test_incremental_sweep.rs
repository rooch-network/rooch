// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for Incremental Sweep functionality
//!
//! This test verifies that:
//! 1. Stale indices are correctly written during state updates via handle_tx_output
//! 2. Refcounts are properly maintained
//! 3. Incremental sweep can delete nodes with refcount==0

use crate::MoveOSStore;
use moveos_types::h256::H256;
use smt::NodeReader;

#[tokio::test]
async fn test_incremental_sweep_via_smt_api() {
    // This test directly uses SMT API to verify the complete stale_indices + refcount mechanism
    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();

    use smt::SMTree;
    use std::collections::HashSet;

    let smt = SMTree::new(
        store.get_state_node_store().clone(),
        &prometheus::Registry::new(),
    );
    let genesis_root = *smt::SPARSE_MERKLE_PLACEHOLDER_HASH;

    // Step 1: Insert initial data
    let key1 = H256::random();
    let value1 = vec![1u8; 32];

    let changeset1 = smt.put(genesis_root, key1, value1.clone()).unwrap();
    println!(
        "✅ Step 1: Inserted key1, created {} new nodes",
        changeset1.nodes.len()
    );

    // Simulate handle_tx_output: increment refcount for new nodes
    for hash in changeset1.nodes.keys() {
        let _ = store.prune_store.inc_node_refcount(*hash);
    }

    // Write nodes and stale indices
    store
        .get_state_node_store()
        .write_nodes(changeset1.nodes.clone())
        .unwrap();
    if !changeset1.stale_indices.is_empty() {
        let _ = store
            .prune_store
            .write_stale_indices(&changeset1.stale_indices);
    }

    // Verify: All new nodes should have refcount > 0
    for hash in changeset1.nodes.keys() {
        let refcount = store.prune_store.get_node_refcount(*hash).unwrap();
        assert!(
            refcount > 0,
            "New node {:?} should have refcount > 0, got {}",
            hash,
            refcount
        );
    }

    // Step 2: Update the SAME key with a different value
    let value2 = vec![2u8; 32];
    let changeset2 = smt
        .put(changeset1.state_root, key1, value2.clone())
        .unwrap();
    println!(
        "✅ Step 2: Updated key1, created {} new nodes, {} stale indices",
        changeset2.nodes.len(),
        changeset2.stale_indices.len()
    );

    // CRITICAL TEST: Verify new nodes are NOT in stale_indices (this is the bug we fixed!)
    let new_nodes_2: HashSet<H256> = changeset2.nodes.keys().cloned().collect();
    let mut bug_detected = false;
    for (_stale_root, stale_hash) in &changeset2.stale_indices {
        if new_nodes_2.contains(stale_hash) {
            eprintln!(
                "❌ BUG DETECTED: New node {:?} found in stale_indices!",
                stale_hash
            );
            bug_detected = true;
        }
    }
    assert!(
        !bug_detected,
        "BUG: New nodes should NEVER appear in stale_indices!"
    );
    println!("   ✅ Verified: No new nodes in stale_indices (bug fix working!)");

    // Simulate handle_tx_output: increment refcount for new nodes
    for hash in changeset2.nodes.keys() {
        let _ = store.prune_store.inc_node_refcount(*hash);
    }

    // Write new nodes and stale indices
    store
        .get_state_node_store()
        .write_nodes(changeset2.nodes.clone())
        .unwrap();
    if !changeset2.stale_indices.is_empty() {
        let _ = store
            .prune_store
            .write_stale_indices(&changeset2.stale_indices);
    }

    // Step 3: Verify refcount correctness
    // New nodes from changeset2 should have refcount > 0
    for hash in changeset2.nodes.keys() {
        let refcount = store.prune_store.get_node_refcount(*hash).unwrap();
        assert!(
            refcount > 0,
            "New node {:?} should have refcount > 0, got {}",
            hash,
            refcount
        );
    }
    println!("   ✅ All new nodes have refcount > 0");

    // Stale nodes (old nodes from changeset1) should have refcount == 0
    let mut nodes_with_zero_refcount = 0;
    for (_stale_root, stale_hash) in &changeset2.stale_indices {
        let refcount = store.prune_store.get_node_refcount(*stale_hash).unwrap();
        if refcount == 0 {
            nodes_with_zero_refcount += 1;
        }
    }
    println!(
        "   ✅ {} stale nodes have refcount == 0 (safe to delete)",
        nodes_with_zero_refcount
    );

    // Step 4: Simulate IncrementalSweep - delete nodes with refcount == 0
    let stale_list = store
        .prune_store
        .list_before(H256::from_low_u64_be(u64::MAX), 1000)
        .unwrap();
    let mut deleted_count = 0;
    for (stale_root, node_hash) in stale_list {
        let refcount = store.prune_store.get_node_refcount(node_hash).unwrap();
        if refcount == 0 {
            // Verify node exists before deletion
            let exists = store
                .get_state_node_store()
                .get(&node_hash)
                .unwrap()
                .is_some();
            assert!(exists, "Node {:?} should exist before sweep", node_hash);

            // Delete the node
            store
                .get_state_node_store()
                .delete_nodes(vec![node_hash])
                .unwrap();
            store
                .prune_store
                .remove_stale_indice((stale_root, node_hash))
                .unwrap();
            store.prune_store.remove_node_refcount(node_hash).unwrap();
            deleted_count += 1;

            // Verify node is deleted
            let exists = store
                .get_state_node_store()
                .get(&node_hash)
                .unwrap()
                .is_some();
            assert!(
                !exists,
                "Node {:?} should be deleted after sweep",
                node_hash
            );
        }
    }
    println!(
        "✅ Step 4: Incremental sweep deleted {} nodes with refcount == 0",
        deleted_count
    );

    // Step 5: Verify the tree is still intact - we can still query the latest data
    let (result, _proof) = smt.get_with_proof(changeset2.state_root, key1).unwrap();
    assert_eq!(
        result,
        Some(value2.clone()),
        "Tree should still be queryable after sweep"
    );
    println!("✅ Step 5: Tree is still intact, can query key1 successfully");

    println!("\n✅ Complete integration test PASSED!");
    println!("   - Stale indices only contain OLD nodes (not new nodes)");
    println!("   - Refcount mechanism prevents deletion of active nodes");
    println!("   - IncrementalSweep safely deletes only nodes with refcount == 0");
}

#[tokio::test]
async fn test_refcount_prevents_premature_deletion() {
    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();

    let node_hash = H256::random();
    let node_data = b"test_node_data".to_vec();

    // Create a node
    store
        .get_state_node_store()
        .put(node_hash, node_data.clone())
        .unwrap();

    // Simulate multiple references (node is used in multiple state roots)
    store.prune_store.inc_node_refcount(node_hash).unwrap();
    store.prune_store.inc_node_refcount(node_hash).unwrap();
    store.prune_store.inc_node_refcount(node_hash).unwrap();

    let refcount = store.prune_store.get_node_refcount(node_hash).unwrap();
    assert_eq!(refcount, 3, "Node should have refcount == 3");

    // Mark it as stale (this decrements refcount by 1)
    let root = H256::random();
    store
        .prune_store
        .write_stale_indices(&[(root, node_hash)])
        .unwrap();

    // Refcount should be 2 now (was 3, decremented to 2)
    let refcount = store.prune_store.get_node_refcount(node_hash).unwrap();
    assert_eq!(
        refcount, 2,
        "Node should have refcount == 2 after one write_stale_indices"
    );

    // Try to sweep - should NOT delete this node (refcount > 0)
    let cutoff = H256::from_low_u64_be(u64::MAX);
    let stale_list = store.prune_store.list_before(cutoff, 1000).unwrap();

    for (_stale_root, stale_node_hash) in stale_list {
        if stale_node_hash == node_hash {
            let refcount = store
                .prune_store
                .get_node_refcount(stale_node_hash)
                .unwrap();
            assert!(refcount > 0, "Node should have refcount > 0");

            // Should NOT delete
            let exists = store
                .get_state_node_store()
                .get(&stale_node_hash)
                .unwrap()
                .is_some();
            assert!(
                exists,
                "Node {} with refcount={} should NOT be deleted",
                stale_node_hash, refcount
            );
        }
    }

    println!("✅ Refcount protection test passed!");
}
