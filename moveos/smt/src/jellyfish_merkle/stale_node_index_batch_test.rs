// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use super::{mock_tree_store::MockTestStore, *};
use crate::jellyfish_merkle::mock_tree_store::{TestKey, TestValue};
use crate::jellyfish_merkle::node_type::Node;

fn reachable_nodes(store: &MockTestStore, root: SMTNodeHash) -> HashSet<NodeKey> {
    let mut seen = HashSet::new();
    let mut stack = vec![root];
    while let Some(node_key) = stack.pop() {
        if !seen.insert(node_key) {
            continue;
        }
        if let Ok(Some(node)) = store.get_node_option(&node_key) {
            match node {
                Node::Internal(internal) => {
                    for child_hash in internal.all_child() {
                        stack.push(child_hash);
                    }
                }
                Node::Leaf(_leaf) => {
                    // no children
                }
                Node::Null => {}
            }
        }
    }
    seen
}

#[test]
fn test_stale_node_index_batch_basic_stale_verification() {
    // Create initial tree state with multiple key-value pairs
    let store = MockTestStore::new_test();
    let tree = JellyfishMerkleTree::new(&store);

    // Insert initial key-value pairs
    let key1 = TestKey::random();
    let key2 = TestKey::random();

    let value1 = TestValue::from(vec![1u8; 32]);
    let value2 = TestValue::from(vec![2u8; 32]);

    let (root1, batch1) = tree.put_blob_set(None, vec![(key1, value1.clone().into())]).unwrap();
    assert!(batch1.stale_node_index_batch.is_empty());
    assert_eq!(batch1.num_stale_leaves, 0);
    store.write_tree_update_batch(batch1).unwrap();

    let (root2, batch2) = tree.put_blob_set(Some(root1), vec![(key2, value2.clone().into())]).unwrap();
    assert!(batch2.stale_node_index_batch.is_empty());
    assert_eq!(batch2.num_stale_leaves, 0);
    store.write_tree_update_batch(batch2).unwrap();

    // Now update existing key1 with a new value
    let new_value1 = TestValue::from(vec![10u8; 32]);
    let (new_root1, update_batch) = tree.put_blob_set(Some(root2), vec![(key1, new_value1.clone().into())]).unwrap();

    // Verify stale_node_index_batch content
    assert!(!update_batch.stale_node_index_batch.is_empty());
    assert_eq!(update_batch.num_stale_leaves, 1);

    // Key verification 1: stale nodes should not overlap with new nodes
    let stale_nodes: HashSet<NodeKey> = update_batch.stale_node_index_batch.iter()
        .map(|sni| sni.node_key)
        .collect();
    let new_nodes: HashSet<NodeKey> = update_batch.node_batch.keys().cloned().collect();
    assert!(stale_nodes.is_disjoint(&new_nodes), "Stale nodes should not overlap with new nodes");

    // Key verification 2: stale_since_version should be the new root hash
    for sni in &update_batch.stale_node_index_batch {
        assert_eq!(sni.stale_since_version, new_root1, "stale_since_version should be the new root hash");
    }

    // Commit the batch and verify we can still get the updated value, and stale nodes are unreachable
    let stale_nodes_vec: Vec<NodeKey> = stale_nodes.iter().cloned().collect();
    store.write_tree_update_batch(update_batch).unwrap();
    let reachable = reachable_nodes(&store, new_root1);
    for stale in &stale_nodes_vec {
        assert!(
            !reachable.contains(stale),
            "Stale node should not be reachable in new root"
        );
    }
    let (result, _proof) = tree.get_with_proof(new_root1, key1).unwrap();
    assert_eq!(result, Some(new_value1.into()));
}

#[test]
fn test_stale_node_index_batch_delete_stale_verification() {
    // Create tree with multiple key-value pairs
    let store = MockTestStore::new_test();
    let tree = JellyfishMerkleTree::new(&store);

    let key1 = TestKey::random();
    let key2 = TestKey::random();

    let value1 = TestValue::from(vec![1u8; 32]);
    let value2 = TestValue::from(vec![2u8; 32]);

    // Build initial tree
    let (root, batch) = tree.put_blob_set(None, vec![
        (key1, value1.clone().into()),
        (key2, value2.clone().into())
    ]).unwrap();
    store.write_tree_update_batch(batch).unwrap();

    // Delete key2 by setting to empty value
    let empty_value = TestValue::from(vec![]);
    let (new_root, delete_batch) = tree.put_blob_set(Some(root), vec![(key2, empty_value.into())]).unwrap();

    // Verify stale_node_index_batch for deletion
    assert!(!delete_batch.stale_node_index_batch.is_empty());
    assert_eq!(delete_batch.num_stale_leaves, 1);

    // Key verification 1: stale nodes should not overlap with new nodes
    let stale_nodes: HashSet<NodeKey> = delete_batch.stale_node_index_batch.iter()
        .map(|sni| sni.node_key)
        .collect();
    let new_nodes: HashSet<NodeKey> = delete_batch.node_batch.keys().cloned().collect();
    assert!(stale_nodes.is_disjoint(&new_nodes));

    // Key verification 2: stale_since_version should be the new root hash
    for sni in &delete_batch.stale_node_index_batch {
        assert_eq!(sni.stale_since_version, new_root);
    }

    // Commit the batch and verify deleted key returns empty value
    let stale_nodes_vec: Vec<NodeKey> = stale_nodes.iter().cloned().collect();
    store.write_tree_update_batch(delete_batch).unwrap();
    let reachable = reachable_nodes(&store, new_root);
    for stale in &stale_nodes_vec {
        assert!(
            !reachable.contains(stale),
            "Stale node should not be reachable in new root (delete)"
        );
    }
    let (result, _proof) = tree.get_with_proof(new_root, key2).unwrap();
    assert_eq!(result, Some(TestValue::from(vec![]).into()));

    // Verify other key still exists
    let (result1, _proof1) = tree.get_with_proof(new_root, key1).unwrap();
    assert_eq!(result1, Some(value1.into()));
}

#[test]
fn test_stale_node_index_batch_repeated_updates_stale_verification() {
    // Test multiple updates to the same key
    let store = MockTestStore::new_test();
    let tree = JellyfishMerkleTree::new(&store);

    let key = TestKey::random();
    let initial_value = TestValue::from(vec![1u8; 32]);

    // Insert initial value
    let (root1, batch1) = tree.put_blob_set(None, vec![(key, initial_value.clone().into())]).unwrap();
    assert!(batch1.stale_node_index_batch.is_empty());
    store.write_tree_update_batch(batch1).unwrap();

    // Track root hashes
    let mut roots = vec![root1];

    // Perform multiple updates to the same key
    for i in 1..=3 { // Reduced to 3 for simplicity
        let new_value = TestValue::from(vec![(1 + i) as u8; 32]);
        let (new_root, update_batch) = tree.put_blob_set(Some(roots[i-1]), vec![(key, new_value.clone().into())]).unwrap();

        roots.push(new_root);

        // Verify each update creates exactly one stale leaf
        assert_eq!(update_batch.num_stale_leaves, 1);
        assert!(!update_batch.stale_node_index_batch.is_empty());

        // Verify stale nodes don't overlap with new nodes
        let stale_nodes: HashSet<NodeKey> = update_batch.stale_node_index_batch.iter()
            .map(|sni| sni.node_key)
            .collect();
        let new_nodes: HashSet<NodeKey> = update_batch.node_batch.keys().cloned().collect();
        assert!(stale_nodes.is_disjoint(&new_nodes));

        // Verify stale_since_version is the new root
        for sni in &update_batch.stale_node_index_batch {
            assert_eq!(sni.stale_since_version, new_root);
        }

        // Commit the batch for next iteration and ensure stale nodes are unreachable
        let stale_nodes_vec: Vec<NodeKey> = stale_nodes.iter().cloned().collect();
        store.write_tree_update_batch(update_batch).unwrap();
        let reachable = reachable_nodes(&store, new_root);
        for stale in &stale_nodes_vec {
            assert!(
                !reachable.contains(stale),
                "Stale node should not be reachable in repeated updates"
            );
        }
    }

    // Verify we have a sequence of distinct root hashes
    let distinct_roots: HashSet<SMTNodeHash> = roots.into_iter().collect();
    assert_eq!(distinct_roots.len(), 4); // Initial + 3 updates
}

#[test]
fn test_stale_node_index_batch_multiple_keys_updates_stale_verification() {
    // Test updates to multiple keys in sequence
    let store = MockTestStore::new_test();
    let tree = JellyfishMerkleTree::new(&store);

    let key1 = TestKey::random();
    let key2 = TestKey::random();

    let value1 = TestValue::from(vec![1u8; 32]);
    let value2 = TestValue::from(vec![2u8; 32]);

    // Build initial tree
    let (root, batch) = tree.put_blob_set(None, vec![
        (key1, value1.clone().into()),
        (key2, value2.clone().into())
    ]).unwrap();
    store.write_tree_update_batch(batch).unwrap();

    // Update key1
    let new_value1 = TestValue::from(vec![11u8; 32]);
    let (root2, update1_batch) = tree.put_blob_set(Some(root), vec![(key1, new_value1.clone().into())]).unwrap();

    // Verify first update
    assert_eq!(update1_batch.num_stale_leaves, 1);
    let stale_nodes1: HashSet<NodeKey> = update1_batch.stale_node_index_batch.iter()
        .map(|sni| sni.node_key)
        .collect();
    let new_nodes1: HashSet<NodeKey> = update1_batch.node_batch.keys().cloned().collect();
    assert!(stale_nodes1.is_disjoint(&new_nodes1));
    let stale_nodes1_vec: Vec<NodeKey> = stale_nodes1.iter().cloned().collect();
    store.write_tree_update_batch(update1_batch).unwrap();
    let reachable1 = reachable_nodes(&store, root2);
    for stale in &stale_nodes1_vec {
        assert!(
            !reachable1.contains(stale),
            "Stale node should not be reachable after update1"
        );
    }

    // Update key2
    let new_value2 = TestValue::from(vec![22u8; 32]);
    let (root3, update2_batch) = tree.put_blob_set(Some(root2), vec![(key2, new_value2.clone().into())]).unwrap();

    // Verify second update
    assert_eq!(update2_batch.num_stale_leaves, 1);
    let stale_nodes2: HashSet<NodeKey> = update2_batch.stale_node_index_batch.iter()
        .map(|sni| sni.node_key)
        .collect();
    let new_nodes2: HashSet<NodeKey> = update2_batch.node_batch.keys().cloned().collect();
    assert!(stale_nodes2.is_disjoint(&new_nodes2));
    let stale_nodes2_vec: Vec<NodeKey> = stale_nodes2.iter().cloned().collect();
    store.write_tree_update_batch(update2_batch).unwrap();
    let reachable2 = reachable_nodes(&store, root3);
    for stale in &stale_nodes2_vec {
        assert!(
            !reachable2.contains(stale),
            "Stale node should not be reachable after update2"
        );
    }

    // Verify all updates are accessible
    let (result1, _proof1) = tree.get_with_proof(root3, key1).unwrap();
    let (result2, _proof2) = tree.get_with_proof(root3, key2).unwrap();
    assert_eq!(result1, Some(new_value1.into()));
    assert_eq!(result2, Some(new_value2.into()));
}

#[test]
fn test_stale_node_index_batch_batch_updates_stale_verification() {
    // Test batch operations
    let store = MockTestStore::new_test();
    let tree = JellyfishMerkleTree::new(&store);

    // Prepare test data
    let mut key_value_pairs = Vec::new();
    for i in 1..=4 {
        let key = TestKey::random();
        let value = TestValue::from(vec![(i * 10) as u8; 32]);
        key_value_pairs.push((key, value));
    }

    // Build initial tree with all keys
    let (batch_root, batch_result) = tree.put_blob_set(
        None,
        key_value_pairs.iter().map(|(k, v)| (*k, v.clone().into())).collect()
    ).unwrap();
    store.write_tree_update_batch(batch_result).unwrap();

    // Now update multiple keys in a batch
    let updated_pairs: Vec<_> = key_value_pairs.iter()
        .take(2)
        .map(|(key, _value)| {
            let new_value = TestValue::from(vec![255u8; 32]);
            (*key, new_value)
        })
        .collect();

    // Batch updates
    let (batch_updated_root, batch_updated_result) = tree.put_blob_set(
        Some(batch_root),
        updated_pairs.iter().map(|(k, v)| (*k, v.clone().into())).collect()
    ).unwrap();

    // Verify batch update stale nodes
    let batch_updated_stale: HashSet<NodeKey> = batch_updated_result.stale_node_index_batch.iter()
        .map(|sni| sni.node_key)
        .collect();

    let batch_updated_new_nodes: HashSet<NodeKey> = batch_updated_result.node_batch.keys().cloned().collect();
    assert!(batch_updated_stale.is_disjoint(&batch_updated_new_nodes));

    // Verify stale_since_version consistency for batch update
    for sni in &batch_updated_result.stale_node_index_batch {
        assert_eq!(sni.stale_since_version, batch_updated_root);
    }

    // Should have exactly 2 stale leaves (one for each updated key)
    assert_eq!(batch_updated_result.num_stale_leaves, 2);

    // Commit and verify updated values are accessible, stale unreachable
    let stale_nodes_vec: Vec<NodeKey> = batch_updated_stale.iter().cloned().collect();
    store.write_tree_update_batch(batch_updated_result).unwrap();
    let reachable = reachable_nodes(&store, batch_updated_root);
    for stale in &stale_nodes_vec {
        assert!(
            !reachable.contains(stale),
            "Stale node should not be reachable in batch update"
        );
    }
    for (key, new_value) in &updated_pairs {
        let (batch_result_check, _proof2) = tree.get_with_proof(batch_updated_root, *key).unwrap();
        assert_eq!(batch_result_check, Some(new_value.clone().into()));
    }

    // Verify non-updated keys remain unchanged
    for (key, original_value) in key_value_pairs.iter().skip(2) {
        let (batch_result_check, _proof2) = tree.get_with_proof(batch_updated_root, *key).unwrap();
        assert_eq!(batch_result_check, Some(original_value.clone().into()));
    }
}
