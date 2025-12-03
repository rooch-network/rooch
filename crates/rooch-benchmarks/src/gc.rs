// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//! GC Mark Phase Benchmark utilities
//!
//! This module provides utilities for benchmarking the GC Mark Phase,
//! focusing on parallel traversal, work-stealing, and AtomicBloomFilter performance.

use anyhow::Result;
use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use smt::{SMTree, SPARSE_MERKLE_PLACEHOLDER_HASH};
use std::sync::Arc;

/// Test tree builder for GC benchmarks
/// Uses SMTree API to create proper JellyfishMerkleTree structures
/// with Internal and Leaf nodes.
pub struct TreeBuilder {
    store: Arc<MoveOSStore>,
}

impl TreeBuilder {
    pub fn new(store: Arc<MoveOSStore>) -> Self {
        Self { store }
    }

    /// Create a proper tree structure with the specified number of leaf nodes
    /// Returns (root_hash, total_node_count)
    ///
    /// This creates a real JellyfishMerkleTree structure with:
    /// - Internal nodes connecting child nodes
    /// - Leaf nodes storing key-value pairs
    /// - Proper tree traversal from root to leaves
    pub fn create_tree(&self, leaf_count: usize) -> Result<(H256, Vec<H256>)> {
        let registry = prometheus::Registry::new();
        let node_store = self.store.get_state_node_store();
        let smt: SMTree<H256, Vec<u8>, _> = SMTree::new(node_store.clone(), &registry);

        // Start from placeholder root
        let mut current_root = *SPARSE_MERKLE_PLACEHOLDER_HASH;

        // Batch insert to create proper tree structure
        // Use batch size to balance between tree depth and batch overhead
        let batch_size = 1000.min(leaf_count);
        let mut all_node_hashes = Vec::new();

        for batch_start in (0..leaf_count).step_by(batch_size) {
            let batch_end = (batch_start + batch_size).min(leaf_count);
            let updates: Vec<(H256, Option<Vec<u8>>)> = (batch_start..batch_end)
                .map(|i| {
                    let key = H256::from_low_u64_be(i as u64);
                    let value = format!("leaf_data_{}", i).into_bytes();
                    (key, Some(value))
                })
                .collect();

            let changeset = smt.puts(current_root, updates)?;
            current_root = changeset.state_root;

            // Collect all node hashes from this batch
            all_node_hashes.extend(changeset.nodes.keys().cloned());

            // Write nodes to store
            node_store.write_nodes(changeset.nodes)?;
        }

        Ok((current_root, all_node_hashes))
    }

    /// Create multiple separate trees for testing multi-root scenarios
    /// Returns list of (root_hash, node_count) tuples
    pub fn create_multiple_trees(
        &self,
        tree_count: usize,
        nodes_per_tree: usize,
    ) -> Result<Vec<(H256, usize)>> {
        let registry = prometheus::Registry::new();
        let node_store = self.store.get_state_node_store();
        let smt: SMTree<H256, Vec<u8>, _> = SMTree::new(node_store.clone(), &registry);

        let mut trees = Vec::new();

        for tree_id in 0..tree_count {
            let base_offset = tree_id * nodes_per_tree * 2; // Ensure unique keys across trees

            // Start from placeholder root for each tree
            let mut current_root = *SPARSE_MERKLE_PLACEHOLDER_HASH;
            let mut total_nodes = 0;

            // Batch insert for this tree
            let batch_size = 1000.min(nodes_per_tree);
            for batch_start in (0..nodes_per_tree).step_by(batch_size) {
                let batch_end = (batch_start + batch_size).min(nodes_per_tree);
                let updates: Vec<(H256, Option<Vec<u8>>)> = (batch_start..batch_end)
                    .map(|i| {
                        let key = H256::from_low_u64_be((base_offset + i) as u64);
                        let value = format!("tree_{}_leaf_{}", tree_id, i).into_bytes();
                        (key, Some(value))
                    })
                    .collect();

                let changeset = smt.puts(current_root, updates)?;
                current_root = changeset.state_root;
                total_nodes += changeset.nodes.len();

                // Write nodes to store
                node_store.write_nodes(changeset.nodes)?;
            }

            trees.push((current_root, total_nodes));
        }

        Ok(trees)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smt::NodeReader;

    #[test]
    fn test_tree_builder_basic() -> Result<()> {
        let (store, _tmpdir) = MoveOSStore::mock_moveos_store()?;
        let store = Arc::new(store);
        let builder = TreeBuilder::new(store.clone());

        let (root_hash, all_hashes) = builder.create_tree(10)?;

        // Should have some nodes (internal + leaf nodes)
        assert!(!all_hashes.is_empty());

        // Root should be valid and retrievable
        let node_store = store.get_state_node_store();
        assert!(node_store.get(&root_hash)?.is_some());

        // Verify tree structure is traversable
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![root_hash];
        while let Some(hash) = stack.pop() {
            if visited.contains(&hash) || hash == *SPARSE_MERKLE_PLACEHOLDER_HASH {
                continue;
            }
            visited.insert(hash);

            if let Some(bytes) = node_store.get(&hash)? {
                use smt::jellyfish_merkle::node_type::Node;
                if let Ok(node) = Node::<H256, Vec<u8>>::decode(&bytes) {
                    // Add children to stack for traversal
                    match node {
                        Node::Internal(internal) => {
                            for child_hash in internal.all_child() {
                                stack.push(child_hash.into());
                            }
                        }
                        Node::Leaf(_) | Node::Null => {
                            // Leaf and Null nodes have no children
                        }
                    }
                }
            }
        }

        // Should have visited multiple nodes (internal + leaves)
        assert!(
            visited.len() >= 10,
            "Should have at least 10 nodes, got {}",
            visited.len()
        );

        Ok(())
    }

    #[test]
    fn test_tree_builder_multiple_trees() -> Result<()> {
        let (store, _tmpdir) = MoveOSStore::mock_moveos_store()?;
        let store = Arc::new(store);
        let builder = TreeBuilder::new(store.clone());

        let trees = builder.create_multiple_trees(3, 5)?;

        // Should have 3 trees
        assert_eq!(trees.len(), 3);

        // Each tree should have some nodes
        for (root, count) in &trees {
            assert!(*count > 0, "Tree should have nodes");
            // Verify root is retrievable
            let node_store = store.get_state_node_store();
            assert!(node_store.get(root)?.is_some());
        }

        Ok(())
    }
}
