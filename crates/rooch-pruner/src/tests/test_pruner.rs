// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::incremental_sweep::IncrementalSweep;
use crate::reachability::ReachableBuilder;
use crate::sweep_expired::SweepExpired;
use moveos_common::bloom_filter::BloomFilter;
use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use parking_lot::Mutex;
use smt::jellyfish_merkle::node_type::Node;
use smt::NodeReader;
use smt::SMTObject;
use std::sync::Arc;

#[test]
fn test_bloom_filter_basic() {
    let mut bloom = BloomFilter::new(1 << 20, 4);
    let hash = H256::random();
    assert!(!bloom.contains(&hash));
    bloom.insert(&hash);
    assert!(bloom.contains(&hash));
}

#[tokio::test]
async fn test_refcount_inc_dec() {
    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let key = H256::random();
    // initial ref == 0
    assert_eq!(store.get_prune_store().get_node_refcount(key).unwrap(), 0);
    // inc -> 1
    store.get_prune_store().inc_node_refcount(key).unwrap();
    assert_eq!(store.get_prune_store().get_node_refcount(key).unwrap(), 1);
    // dec -> 0 and removed
    store.get_prune_store().dec_node_refcount(key).unwrap();
}

#[tokio::test]
async fn test_write_stale_indices_and_refcount() {
    // Use MoveOSStore helper to get fully configured stores
    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();

    let root = H256::random();
    let node_hash = H256::random();

    // preset refcount to 1
    store
        .get_prune_store()
        .inc_node_refcount(node_hash)
        .unwrap();
    assert_eq!(
        store
            .get_prune_store()
            .get_node_refcount(node_hash)
            .unwrap(),
        1
    );

    // write stale index which should dec ref to 0 and create stale entry
    store
        .get_prune_store()
        .write_stale_indices(&[(root, node_hash)])
        .unwrap();

    // refcount removed -> 0
    assert_eq!(
        store
            .get_prune_store()
            .get_node_refcount(node_hash)
            .unwrap(),
        0
    );
    // stale index present (timestamp key internally generated)
    let cutoff = H256::from_low_u64_be(u64::MAX);
    let indices = store.get_prune_store().list_before(cutoff, 100).unwrap();
    assert!(indices.iter().any(|(_, nh)| *nh == node_hash));
}

#[tokio::test]
async fn test_reachable_and_sweep() {
    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let node_store = store.get_state_node_store().clone();

    // create reachable leaf
    let key1 = H256::random();
    let value1: Vec<u8> = b"val1".to_vec();
    // let test_value1 = TestValue::from(value1.clone());
    let node1: Node<H256, Vec<u8>> = Node::new_leaf(
        key1,
        SMTObject::<Vec<u8>>::from_origin(value1.clone()).unwrap(),
    );
    let hash1: H256 = node1.get_merkle_hash().into();
    node_store.put(hash1, node1.encode().unwrap()).unwrap();

    // create unreachable leaf
    let key2 = H256::random();
    let value2: Vec<u8> = b"val2".to_vec();
    let node2: Node<H256, Vec<u8>> = Node::new_leaf(
        key2,
        SMTObject::<Vec<u8>>::from_origin(value2.clone()).unwrap(),
    );
    let hash2: H256 = node2.get_merkle_hash().into();
    node_store.put(hash2, node2.encode().unwrap()).unwrap();

    let bloom = Arc::new(Mutex::new(BloomFilter::new(1 << 16, 4)));

    // Build reachable set with hash1
    let builder = ReachableBuilder::new(Arc::new(store.clone()), bloom.clone());
    let scanned = builder.build(vec![hash1], 1).unwrap();
    assert_eq!(scanned, 1);

    // Sweep expired roots containing both hashes
    let sweeper = SweepExpired::new(Arc::new(store.clone()), bloom);
    let deleted = sweeper.sweep(vec![hash1, hash2], 1).unwrap();
    assert_eq!(deleted, 1);

    // Validate storage
    assert!(store.node_store.get(&hash1).unwrap().is_some());
    assert!(node_store.get(&hash2).unwrap().is_none());
}

#[tokio::test]
async fn test_incremental_sweep() {
    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let node_store = store.get_state_node_store().clone();

    // Prepare a node
    let key = H256::random();
    let value: Vec<u8> = b"x".to_vec();
    let node: Node<H256, Vec<u8>> = Node::new_leaf(
        key,
        SMTObject::<Vec<u8>>::from_origin(value.clone()).unwrap(),
    );
    let hash: H256 = node.get_merkle_hash().into();
    node_store.put(hash, node.encode().unwrap()).unwrap();

    // refcount ==1 then decrement via write_stale_indices => 0
    store.prune_store.inc_node_refcount(hash).unwrap();
    let root = H256::random();
    store
        .prune_store
        .write_stale_indices(&[(root, hash)])
        .unwrap();
    assert_eq!(store.prune_store.get_node_refcount(hash).unwrap(), 0);

    // incremental sweep should delete
    let sweeper = IncrementalSweep::new(Arc::new(store.clone()));
    let deleted = sweeper.sweep(root, 100).unwrap();
    assert_eq!(deleted, 1);
    assert!(node_store.get(&hash).unwrap().is_none());
    assert!(store
        .prune_store
        .get_stale_indice((root, hash))
        .unwrap()
        .is_none());
}
