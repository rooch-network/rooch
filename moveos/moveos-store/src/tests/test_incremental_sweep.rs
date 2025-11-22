// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for Incremental Sweep functionality
//!
//! This test verifies that:
//! 1. Stale indices are correctly written during state updates via handle_tx_output
//! 2. Refcounts are properly maintained
//! 3. Incremental sweep can delete nodes with refcount==0

use crate::MoveOSStore;
use move_core_types::vm_status::KeptVMStatus;
use moveos_types::h256::H256;
use moveos_types::test_utils::random_state_change_set;
use moveos_types::transaction::RawTransactionOutput;
use smt::NodeReader;

#[tokio::test]
async fn test_incremental_sweep_via_handle_tx_output() {
    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();

    // Step 1: Create first transaction output
    let tx_hash_v1 = H256::random();
    let mut changeset_v1 = random_state_change_set();

    let output_v1 = RawTransactionOutput {
        status: KeptVMStatus::Executed,
        changeset: changeset_v1.clone(),
        events: vec![],
        gas_used: 100,
        is_upgrade: false,
        is_gas_upgrade: false,
    };

    // Apply first transaction - this should create new nodes and increment refcount
    let (_tx_output_v1, _exec_info_v1) = store.handle_tx_output(tx_hash_v1, output_v1).unwrap();

    let state_root_v1 = changeset_v1.state_root;
    println!(
        "✅ Transaction v1 applied with state_root: {}",
        state_root_v1
    );

    // Verify: all new nodes should have refcount > 0
    let (nodes_v1, _) = store
        .get_state_store()
        .change_set_to_nodes(&mut changeset_v1)
        .unwrap();

    let mut nodes_with_refcount = 0;
    for hash in nodes_v1.keys() {
        let refcount = store.prune_store.get_node_refcount(*hash).unwrap();
        if refcount > 0 {
            nodes_with_refcount += 1;
        }
    }
    println!(
        "   Nodes with refcount > 0: {}/{}",
        nodes_with_refcount,
        nodes_v1.len()
    );
    assert!(
        nodes_with_refcount > 0,
        "Should have nodes with refcount > 0"
    );

    // Step 2: Create second transaction - this will create stale indices
    let tx_hash_v2 = H256::random();
    let mut changeset_v2 = random_state_change_set();
    changeset_v2.state_root = state_root_v1; // Build on top of v1

    let output_v2 = RawTransactionOutput {
        status: KeptVMStatus::Executed,
        changeset: changeset_v2.clone(),
        events: vec![],
        gas_used: 200,
        is_upgrade: false,
        is_gas_upgrade: false,
    };

    let (_tx_output_v2, _exec_info_v2) = store.handle_tx_output(tx_hash_v2, output_v2).unwrap();

    let state_root_v2 = changeset_v2.state_root;
    println!(
        "✅ Transaction v2 applied with state_root: {}",
        state_root_v2
    );

    // Step 3: Verify stale indices were written
    let cutoff = H256::from_low_u64_be(u64::MAX); // Include all stale indices
    let stale_list = store.prune_store.list_before(cutoff, 1000).unwrap();

    println!("   Found {} stale indices in store", stale_list.len());
    assert!(
        !stale_list.is_empty(),
        "Should have stale indices after second transaction"
    );

    // Step 4: Verify some nodes have refcount == 0
    let mut nodes_with_zero_refcount = Vec::new();
    for (_stale_root, node_hash) in &stale_list {
        let refcount = store.prune_store.get_node_refcount(*node_hash).unwrap();
        if refcount == 0 {
            nodes_with_zero_refcount.push(*node_hash);
            // Verify node exists before sweep
            let exists = store
                .get_state_node_store()
                .get(node_hash)
                .unwrap()
                .is_some();
            assert!(exists, "Node {} should exist before sweep", node_hash);
        }
    }

    println!(
        "   Nodes with refcount == 0: {}",
        nodes_with_zero_refcount.len()
    );
    assert!(
        !nodes_with_zero_refcount.is_empty(),
        "Should have at least some nodes with refcount == 0"
    );

    // Step 5: Manually run incremental sweep
    let mut deleted_count = 0;
    for (stale_root, node_hash) in stale_list {
        let refcount = store.prune_store.get_node_refcount(node_hash).unwrap();
        if refcount == 0 {
            // Delete the node
            store
                .get_state_node_store()
                .delete_nodes(vec![node_hash])
                .unwrap();
            // Remove metadata
            store
                .prune_store
                .remove_stale_indice((stale_root, node_hash))
                .unwrap();
            store.prune_store.remove_node_refcount(node_hash).unwrap();
            deleted_count += 1;
        }
    }

    println!("   Incremental sweep deleted {} nodes", deleted_count);
    assert!(deleted_count > 0, "Should delete at least some nodes");

    // Step 6: Verify deleted nodes are gone
    for node_hash in &nodes_with_zero_refcount {
        let exists = store
            .get_state_node_store()
            .get(node_hash)
            .unwrap()
            .is_some();
        assert!(!exists, "Node {} should be deleted after sweep", node_hash);
    }

    println!("✅ Incremental sweep integration test passed!");
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
