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
    // let temp_dir = moveos_config::temp_dir();
    // let registry = prometheus::Registry::new();
    // DBMetrics::init(&registry);

    // let db_metrics = DBMetrics::get_or_init(&registry).clone();
    // let instance = StoreInstance::new_db_instance(
    //     RocksDB::new(
    //         temp_dir.path(),
    //         StoreMeta::get_column_family_names().to_vec(),
    //         RocksdbConfig::default(),
    //     )
    //     .unwrap(),
    //     db_metrics,
    // );

    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    // let ref_store = NodeRefcountStore::new(instance.clone());
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
    // let node_store = store.get_state_node_store();
    // let instance = node_store.get_store().store().clone();
    // let stale_store = StaleIndexStore::new(instance.clone());
    // let ref_store = NodeRefcountStore::new(instance);

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
        .stale_index_store
        .kv_get((root, node_hash))
        .unwrap()
        .is_some());
}
