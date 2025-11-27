// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use smt::jellyfish_merkle::node_type::Node;
use smt::SMTObject;
use std::sync::Arc;

/// Test utilities for building predictable SMT structures and testing pruner scenarios
pub struct TreeBuilder {
    store: Arc<MoveOSStore>,
}

impl TreeBuilder {
    pub fn new(store: Arc<MoveOSStore>) -> Self {
        Self { store }
    }

    /// Create a simple tree structure with multiple leaf nodes
    /// Returns (root_hash, all_node_hashes)
    /// For simplicity, we'll create multiple leaf nodes and use one as the root
    pub fn create_simple_tree(&self, leaf_count: usize) -> (H256, Vec<H256>) {
        let mut node_hashes = Vec::new();

        // Create leaf nodes
        for i in 0..leaf_count {
            let key = H256::from_low_u64_be(i as u64);
            let value = SMTObject::<Vec<u8>>::from_origin(format!("leaf_data_{}", i).into_bytes()).unwrap();
            let leaf_node = Node::new_leaf(key, value);
            let leaf_hash: H256 = leaf_node.get_merkle_hash().into();
            self.store.get_state_node_store().put(leaf_hash, leaf_node.encode().unwrap()).unwrap();
            node_hashes.push(leaf_hash);
        }

        // Use the first leaf as root for simplicity
        let root_hash = node_hashes[0];

        (root_hash, node_hashes)
    }

    /// Create a more complex tree structure with different root and leaves
    /// Returns (root_hash, leaf_hashes)
    pub fn create_root_and_leaves(&self, leaf_count: usize) -> (H256, Vec<H256>) {
        let mut leaf_hashes = Vec::new();

        // Create leaf nodes
        for i in 0..leaf_count {
            let key = H256::from_low_u64_be(i as u64);
            let value = SMTObject::<Vec<u8>>::from_origin(format!("leaf_data_{}", i).into_bytes()).unwrap();
            let leaf_node = Node::new_leaf(key, value);
            let leaf_hash: H256 = leaf_node.get_merkle_hash().into();
            self.store.get_state_node_store().put(leaf_hash, leaf_node.encode().unwrap()).unwrap();
            leaf_hashes.push(leaf_hash);
        }

        // Create a separate root node
        let root_key = H256::from_low_u64_be(999999);
        let root_value = SMTObject::<Vec<u8>>::from_origin(b"root_data".to_vec()).unwrap();
        let root_node = Node::new_leaf(root_key, root_value);
        let root_hash: H256 = root_node.get_merkle_hash().into();
        self.store.get_state_node_store().put(root_hash, root_node.encode().unwrap()).unwrap();

        (root_hash, leaf_hashes)
    }

    /// Inject stale indices for specified nodes at given transaction order
    pub fn inject_stale_indices(&self, nodes: Vec<H256>, tx_order: u64) -> Result<(), anyhow::Error> {
        // Create root entries for stale indices
        let root_entries: Vec<(H256, H256)> = nodes.iter().map(|&node_hash| {
            let dummy_root = H256::random(); // Dummy root for testing
            (dummy_root, node_hash)
        }).collect();

        self.store.get_prune_store().write_stale_indices(tx_order, &root_entries)?;
        Ok(())
    }

    /// Set refcount for nodes (useful for testing refcount edge cases)
    pub fn set_node_refcounts(&self, refcounts: Vec<(H256, u64)>) -> Result<(), anyhow::Error> {
        for (node_hash, count) in refcounts {
            for _ in 0..count {
                self.store.get_prune_store().inc_node_refcount(node_hash)?;
            }
        }
        Ok(())
    }
}

/// Utility for inspecting BloomFilter contents in tests
pub struct BloomInspector;

impl BloomInspector {
    /// Since we can't directly extract all hashes from a bloom filter,
    /// we provide testing utilities to work with bloom filter behavior

    /// Count how many test hashes are contained in the bloom filter
    pub fn count_contained_hashes(bloom: &parking_lot::Mutex<moveos_common::bloom_filter::BloomFilter>, test_hashes: &[H256]) -> usize {
        let bloom_guard = bloom.lock();
        test_hashes.iter().filter(|hash| bloom_guard.contains(hash)).count()
    }

    /// Check if all expected hashes are in the bloom filter
    pub fn contains_all_hashes(bloom: &parking_lot::Mutex<moveos_common::bloom_filter::BloomFilter>, expected_hashes: &[H256]) -> bool {
        let bloom_guard = bloom.lock();
        expected_hashes.iter().all(|hash| bloom_guard.contains(hash))
    }

    /// Create a test bloom filter with known hashes
    pub fn create_test_bloom_with_hashes(hashes: &[H256]) -> parking_lot::Mutex<moveos_common::bloom_filter::BloomFilter> {
        let bloom = moveos_common::bloom_filter::BloomFilter::new(1 << 20, 4);
        let mutex_bloom = parking_lot::Mutex::new(bloom);

        {
            let mut bloom_guard = mutex_bloom.lock();
            for hash in hashes {
                bloom_guard.insert(hash);
            }
        }

        mutex_bloom
    }
}