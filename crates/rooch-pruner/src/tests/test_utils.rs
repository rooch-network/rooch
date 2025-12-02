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

    /// Create a more complex tree structure with different root and leaves
    /// Returns (root_hash, leaf_hashes)
    pub fn create_root_and_leaves(&self, leaf_count: usize) -> (H256, Vec<H256>) {
        let mut leaf_hashes = Vec::new();

        // Create leaf nodes
        for i in 0..leaf_count {
            let key = H256::from_low_u64_be(i as u64);
            let value =
                SMTObject::<Vec<u8>>::from_origin(format!("leaf_data_{}", i).into_bytes()).unwrap();
            let leaf_node = Node::new_leaf(key, value);
            let leaf_hash: H256 = leaf_node.get_merkle_hash().into();
            self.store
                .get_state_node_store()
                .put(leaf_hash, leaf_node.encode().unwrap())
                .unwrap();
            leaf_hashes.push(leaf_hash);
        }

        // Create a separate root node
        let root_key = H256::from_low_u64_be(999999);
        let root_value = SMTObject::<Vec<u8>>::from_origin(b"root_data".to_vec()).unwrap();
        let root_node = Node::new_leaf(root_key, root_value);
        let root_hash: H256 = root_node.get_merkle_hash().into();
        self.store
            .get_state_node_store()
            .put(root_hash, root_node.encode().unwrap())
            .unwrap();

        (root_hash, leaf_hashes)
    }
}

/// Utility for inspecting BloomFilter contents in tests
pub struct BloomInspector;

impl BloomInspector {
    /// Since we can't directly extract all hashes from a bloom filter,
    /// we provide testing utilities to work with bloom filter behavior

    /// Count how many test hashes are contained in the bloom filter
    pub fn count_contained_hashes(
        bloom: &parking_lot::Mutex<moveos_common::bloom_filter::BloomFilter>,
        test_hashes: &[H256],
    ) -> usize {
        let bloom_guard = bloom.lock();
        test_hashes
            .iter()
            .filter(|hash| bloom_guard.contains(hash))
            .count()
    }
}
