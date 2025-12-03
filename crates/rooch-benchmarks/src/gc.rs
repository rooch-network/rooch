// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//! GC Mark Phase Benchmark utilities
//!
//! This module provides utilities for benchmarking the GC Mark Phase,
//! focusing on parallel traversal, work-stealing, and AtomicBloomFilter performance.

use anyhow::Result;
use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use smt::jellyfish_merkle::node_type::Node;
use smt::SMTObject;
use std::sync::Arc;

/// Test tree builder for GC benchmarks
pub struct TreeBuilder {
    store: Arc<MoveOSStore>,
}

impl TreeBuilder {
    pub fn new(store: Arc<MoveOSStore>) -> Self {
        Self { store }
    }

    /// Create a tree structure with the specified number of leaf nodes
    /// Returns (root_hash, all_node_hashes)
    ///
    /// This creates a simple structure where:
    /// - Multiple leaf nodes are created with unique keys and values
    /// - A separate root node is created
    /// - All nodes are stored in the node store
    pub fn create_tree(&self, leaf_count: usize) -> Result<(H256, Vec<H256>)> {
        let mut all_hashes = Vec::new();

        // Create leaf nodes
        for i in 0..leaf_count {
            let key = H256::from_low_u64_be(i as u64);
            let value = SMTObject::<Vec<u8>>::from_origin(format!("leaf_data_{}", i).into_bytes())?;
            let leaf_node = Node::new_leaf(key, value);
            let leaf_hash: H256 = leaf_node.get_merkle_hash().into();
            self.store
                .get_state_node_store()
                .put(leaf_hash, leaf_node.encode()?)?;
            all_hashes.push(leaf_hash);
        }

        // Create a root node
        let root_key = H256::from_low_u64_be(999999);
        let root_value = SMTObject::<Vec<u8>>::from_origin(b"root_data".to_vec())?;
        let root_node = Node::new_leaf(root_key, root_value);
        let root_hash: H256 = root_node.get_merkle_hash().into();
        self.store
            .get_state_node_store()
            .put(root_hash, root_node.encode()?)?;

        // Add root to the list
        all_hashes.push(root_hash);

        Ok((root_hash, all_hashes))
    }

    /// Create multiple separate trees for testing multi-root scenarios
    /// Returns list of (root_hash, node_count) tuples
    pub fn create_multiple_trees(
        &self,
        tree_count: usize,
        nodes_per_tree: usize,
    ) -> Result<Vec<(H256, usize)>> {
        let mut trees = Vec::new();

        for tree_id in 0..tree_count {
            let base_offset = tree_id * nodes_per_tree * 2; // Ensure unique keys across trees
            let mut tree_hashes = Vec::new();

            // Create leaf nodes for this tree
            for i in 0..nodes_per_tree {
                let key = H256::from_low_u64_be((base_offset + i) as u64);
                let value = SMTObject::<Vec<u8>>::from_origin(
                    format!("tree_{}_leaf_{}", tree_id, i).into_bytes(),
                )?;
                let leaf_node = Node::new_leaf(key, value);
                let leaf_hash: H256 = leaf_node.get_merkle_hash().into();
                self.store
                    .get_state_node_store()
                    .put(leaf_hash, leaf_node.encode()?)?;
                tree_hashes.push(leaf_hash);
            }

            // Create root for this tree
            let root_key = H256::from_low_u64_be((base_offset + nodes_per_tree + 999999) as u64);
            let root_value =
                SMTObject::<Vec<u8>>::from_origin(format!("tree_{}_root", tree_id).into_bytes())?;
            let root_node = Node::new_leaf(root_key, root_value);
            let root_hash: H256 = root_node.get_merkle_hash().into();
            self.store
                .get_state_node_store()
                .put(root_hash, root_node.encode()?)?;

            trees.push((root_hash, tree_hashes.len() + 1)); // +1 for root
        }

        Ok(trees)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_builder_basic() -> Result<()> {
        let (store, _tmpdir) = MoveOSStore::mock_moveos_store()?;
        let store = Arc::new(store);
        let builder = TreeBuilder::new(store.clone());

        let (root_hash, all_hashes) = builder.create_tree(10)?;

        // Should have 10 leaves + 1 root = 11 nodes
        assert_eq!(all_hashes.len(), 11);
        assert_eq!(all_hashes.last().unwrap(), &root_hash);

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

        // Each tree should have 5 leaves + 1 root = 6 nodes
        for (_root, count) in &trees {
            assert_eq!(*count, 6);
        }

        Ok(())
    }
}
