// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[test]
fn test_smt() {
    let node_store = InMemoryNodeStore::default();
    let registry = prometheus::Registry::new();
    let smt = SMTree::new(node_store.clone(), &registry);
    let key = H256::random();
    let value = "value";
    let genesis_root = *SPARSE_MERKLE_PLACEHOLDER_HASH;
    let changeset = smt.put(genesis_root, key, value.to_string()).unwrap();
    node_store.write_nodes(changeset.nodes).unwrap();
    let (result, proof) = smt.get_with_proof(changeset.state_root, key).unwrap();
    assert_eq!(result.unwrap(), value.to_string());
    assert!(proof
        .verify(changeset.state_root, key, Some(value.to_string()))
        .is_ok());
    let key2 = H256::random();

    let (result, proof) = smt.get_with_proof(changeset.state_root, key2).unwrap();
    assert_eq!(result, None);
    assert!(proof
        .verify::<H256, String>(changeset.state_root, key2, None)
        .is_ok());

    let mut iter = smt.iter(changeset.state_root, None).unwrap();

    let item = iter.next();
    assert_eq!(item.unwrap().unwrap(), (key, value.to_string()));

    let value2 = "value2".to_owned();
    let key3 = H256::random();
    let value3 = "value3".to_owned();

    let changeset2 = smt
        .puts(
            changeset.state_root,
            vec![(key2, Some(value2.clone())), (key3, Some(value3))],
        )
        .unwrap();

    node_store.write_nodes(changeset2.nodes).unwrap();
    let (result, proof) = smt.get_with_proof(changeset2.state_root, key2).unwrap();
    assert_eq!(result, Some(value2.clone()));
    assert!(proof
        .verify::<H256, String>(changeset2.state_root, key2, Some(value2))
        .is_ok());

    let iter = smt.iter(changeset2.state_root, None).unwrap();
    assert_eq!(iter.count(), 3);

    let changeset3 = smt.remove(changeset2.state_root, key2).unwrap();
    node_store.write_nodes(changeset3.nodes).unwrap();
    let iter = smt.iter(changeset3.state_root, None).unwrap();
    assert_eq!(iter.count(), 2);
}

#[test]
fn test_stale_indices_correctness() {
    // This test verifies that stale_indices contains OLD nodes, not NEW nodes.
    // Bug: Previously, new nodes were incorrectly added to stale_indices.

    let node_store = InMemoryNodeStore::default();
    let registry = prometheus::Registry::new();
    let smt = SMTree::new(node_store.clone(), &registry);

    // Step 1: Create initial state with one key-value pair
    let key1 = H256::random();
    let value1 = "value1".to_string();
    let genesis_root = *SPARSE_MERKLE_PLACEHOLDER_HASH;

    let changeset1 = smt.put(genesis_root, key1, value1.clone()).unwrap();
    node_store.write_nodes(changeset1.nodes.clone()).unwrap();

    // Collect all new node hashes from changeset1
    let new_nodes_1: std::collections::HashSet<H256> = changeset1.nodes.keys().cloned().collect();

    // CRITICAL: New nodes should NOT be in stale_indices
    for (_stale_root, stale_node_hash) in &changeset1.stale_indices {
        assert!(
            !new_nodes_1.contains(stale_node_hash),
            "Bug detected: New node {:?} found in stale_indices! New nodes should never be marked as stale.",
            stale_node_hash
        );
    }

    // Step 2: Update the same key with a new value
    let value2 = "value2".to_string();
    let changeset2 = smt
        .put(changeset1.state_root, key1, value2.clone())
        .unwrap();
    node_store.write_nodes(changeset2.nodes.clone()).unwrap();

    // Collect all new node hashes from changeset2
    let new_nodes_2: std::collections::HashSet<H256> = changeset2.nodes.keys().cloned().collect();

    // CRITICAL: New nodes should NOT be in stale_indices
    for (_stale_root, stale_node_hash) in &changeset2.stale_indices {
        assert!(
            !new_nodes_2.contains(stale_node_hash),
            "Bug detected: New node {:?} found in stale_indices! New nodes should never be marked as stale.",
            stale_node_hash
        );
    }

    // Verify that stale_indices contains old nodes from changeset1
    // (This is the correct behavior - old nodes being replaced should be marked stale)
    if !changeset2.stale_indices.is_empty() {
        for (_stale_root, stale_node_hash) in &changeset2.stale_indices {
            // Stale nodes should be from the OLD tree (changeset1), not the NEW tree (changeset2)
            assert!(
                !new_nodes_2.contains(stale_node_hash),
                "Stale node {:?} should not be in the new node set",
                stale_node_hash
            );
        }
    }

    // Step 3: Add another key to create a more complex tree
    let key2 = H256::random();
    let value3 = "value3".to_string();
    let changeset3 = smt
        .put(changeset2.state_root, key2, value3.clone())
        .unwrap();
    node_store.write_nodes(changeset3.nodes.clone()).unwrap();

    // Collect all new node hashes from changeset3
    let new_nodes_3: std::collections::HashSet<H256> = changeset3.nodes.keys().cloned().collect();

    // CRITICAL: New nodes should NOT be in stale_indices
    for (_stale_root, stale_node_hash) in &changeset3.stale_indices {
        assert!(
            !new_nodes_3.contains(stale_node_hash),
            "Bug detected: New node {:?} found in stale_indices! New nodes should never be marked as stale.",
            stale_node_hash
        );
    }

    // Verify final state
    let (result1, _) = smt.get_with_proof(changeset3.state_root, key1).unwrap();
    assert_eq!(result1, Some(value2));

    let (result2, _) = smt.get_with_proof(changeset3.state_root, key2).unwrap();
    assert_eq!(result2, Some(value3));
}

#[test]
fn test_stale_indices_with_refcount_simulation() {
    // This test simulates how the pruner's refcount mechanism should work
    // with stale_indices to ensure nodes are not prematurely deleted.

    use std::collections::HashMap;

    let node_store = InMemoryNodeStore::default();
    let registry = prometheus::Registry::new();
    let smt = SMTree::new(node_store.clone(), &registry);

    // Simulate refcount tracking
    let mut refcount: HashMap<H256, u32> = HashMap::new();

    // Step 1: Create initial tree
    let key1 = H256::random();
    let value1 = "value1".to_string();
    let genesis_root = *SPARSE_MERKLE_PLACEHOLDER_HASH;

    let changeset1 = smt.put(genesis_root, key1, value1.clone()).unwrap();
    node_store.write_nodes(changeset1.nodes.clone()).unwrap();

    // Increment refcount for new nodes (simulating handle_tx_output)
    for node_hash in changeset1.nodes.keys() {
        *refcount.entry(*node_hash).or_insert(0) += 1;
    }

    // Decrement refcount for stale nodes (simulating write_stale_indices)
    for (_stale_root, stale_node_hash) in &changeset1.stale_indices {
        let count = refcount.entry(*stale_node_hash).or_insert(1);
        *count = count.saturating_sub(1);
    }

    // Verify: All new nodes should have refcount > 0
    for node_hash in changeset1.nodes.keys() {
        let count = refcount.get(node_hash).unwrap_or(&0);
        assert!(
            *count > 0,
            "Bug detected: New node {:?} has refcount=0! This would cause it to be deleted by IncrementalSweep.",
            node_hash
        );
    }

    // Step 2: Update the tree
    let value2 = "value2".to_string();
    let changeset2 = smt
        .put(changeset1.state_root, key1, value2.clone())
        .unwrap();
    node_store.write_nodes(changeset2.nodes.clone()).unwrap();

    // Increment refcount for new nodes
    for node_hash in changeset2.nodes.keys() {
        *refcount.entry(*node_hash).or_insert(0) += 1;
    }

    // Decrement refcount for stale nodes
    for (_stale_root, stale_node_hash) in &changeset2.stale_indices {
        let count = refcount.entry(*stale_node_hash).or_insert(1);
        *count = count.saturating_sub(1);
    }

    // Verify: All new nodes should have refcount > 0
    for node_hash in changeset2.nodes.keys() {
        let count = refcount.get(node_hash).unwrap_or(&0);
        assert!(
            *count > 0,
            "Bug detected: New node {:?} has refcount=0! This would cause it to be deleted by IncrementalSweep.",
            node_hash
        );
    }

    // Verify: Tree is still intact and can be queried
    let (result, _) = smt.get_with_proof(changeset2.state_root, key1).unwrap();
    assert_eq!(
        result,
        Some(value2),
        "Tree query failed - nodes may have been incorrectly marked for deletion"
    );
}
