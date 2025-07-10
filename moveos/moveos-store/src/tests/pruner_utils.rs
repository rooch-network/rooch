use crate::state_store::{NodeRefcountStore, StaleIndexStore};
use crate::{MoveOSStore, StoreMeta};
use moveos_config::store_config::RocksdbConfig;
use moveos_types::h256::H256;
use raw_store::metrics::DBMetrics;
use raw_store::rocks::RocksDB;
use raw_store::StoreInstance;
use std::path::PathBuf;

#[test]
fn test_bloom_filter_basic() {
    use rooch_common::bloom::BloomFilter;
    let mut bloom = BloomFilter::new(1 << 20, 4);
    let hash = H256::random();
    assert!(!bloom.contains(&hash));
    bloom.insert(&hash);
    assert!(bloom.contains(&hash));
}

#[test]
fn test_refcount_inc_dec() {
    let temp_dir = moveos_config::temp_dir();
    let registry = prometheus::Registry::new();
    DBMetrics::init(&registry);

    let db_metrics = DBMetrics::get_or_init(&registry).clone();
    let instance = StoreInstance::new_db_instance(
        RocksDB::new(
            temp_dir.path(),
            StoreMeta::get_column_family_names().to_vec(),
            RocksdbConfig::default(),
        )
        .unwrap(),
        db_metrics,
    );

    let ref_store = NodeRefcountStore::new(instance.clone());
    let key = H256::random();
    // initial ref == 0
    assert_eq!(ref_store.get_ref(key).unwrap(), 0);
    // inc -> 1
    ref_store.inc(key).unwrap();
    assert_eq!(ref_store.get_ref(key).unwrap(), 1);
    // dec -> 0 and removed
    ref_store.dec(key).unwrap();
    assert_eq!(ref_store.get_ref(key).unwrap(), 0);
}

#[test]
fn test_write_stale_indices_and_refcount() {
    // Use MoveOSStore helper to get fully configured stores
    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let node_store = store.get_state_node_store();
    let instance = node_store.get_store().store().clone();
    let stale_store = StaleIndexStore::new(instance.clone());
    let ref_store = NodeRefcountStore::new(instance);

    let root = H256::random();
    let node_hash = H256::random();

    // preset refcount to 1
    ref_store.inc(node_hash).unwrap();
    assert_eq!(ref_store.get_ref(node_hash).unwrap(), 1);

    // write stale index which should dec ref to 0 and create stale entry
    node_store
        .write_stale_indices(&[(root, node_hash)])
        .unwrap();

    // refcount removed -> 0
    assert_eq!(ref_store.get_ref(node_hash).unwrap(), 0);
    // stale index present
    assert!(stale_store.kv_get((root, node_hash)).unwrap().is_some());
} 