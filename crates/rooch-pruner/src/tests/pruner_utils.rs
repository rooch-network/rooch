// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_common::bloom_filter::BloomFilter;
use moveos_store::MoveOSStore;
use moveos_types::h256::H256;

#[test]
fn test_bloom_filter_basic() {
    let mut bloom = BloomFilter::new(1 << 20, 4);
    let hash = H256::random();
    assert!(!bloom.contains(&hash));
    bloom.insert(&hash);
    assert!(bloom.contains(&hash));
}

#[test]
fn test_refcount_inc_dec() {
    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let key = H256::random();
    // initial ref == 0
    assert_eq!(store.get_prune_store().get_node_refcount(key).unwrap(), 0);
    // inc -> 1
    store.get_prune_store().inc_node_refcount(key).unwrap();
    assert_eq!(store.get_prune_store().get_node_refcount(key).unwrap(), 1);
    // dec -> 0 and removed
    store.get_prune_store().inc_node_refcount(key).unwrap();
    assert_eq!(store.get_prune_store().get_node_refcount(key).unwrap(), 0);
}

#[test]
fn test_write_stale_indices_and_refcount() {
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
    // stale index present
    assert!(store
        .get_prune_store()
        .get_stale_indice((root, node_hash))
        .unwrap()
        .is_some());
}
