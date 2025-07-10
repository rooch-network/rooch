use crate::state_store::incremental_sweep::IncrementalSweep;
use crate::state_store::metrics::StateDBMetrics;
use crate::state_store::reachability::ReachableBuilder;
use crate::state_store::sweep_expired::SweepExpired;
use crate::state_store::{NodeRefcountStore, StaleIndexStore};
use crate::{MoveOSStore, StoreMeta};
use moveos_config::store_config::RocksdbConfig;
use moveos_types::h256::H256;
use prometheus::Registry;
use raw_store::metrics::DBMetrics;
use raw_store::rocks::RocksDB;
use raw_store::StoreInstance;
use rooch_common::bloom::BloomFilter;
use smt::jellyfish_merkle::node_type::LeafNode;
use smt::jellyfish_merkle::node_type::Node;
use smt::jellyfish_merkle::node_type::SMTHash;
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn test_reachable_and_sweep() {
    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let node_store = store.get_state_node_store().clone();

    // create reachable leaf
    let key1 = H256::random();
    let value1: Vec<u8> = b"val1".to_vec();
    let node1: Node<H256, Vec<u8>> = Node::new_leaf(key1, value1.clone());
    let hash1: H256 = node1.merkle_hash().into();
    node_store.put(hash1, node1.encode().unwrap()).unwrap();

    // create unreachable leaf
    let key2 = H256::random();
    let value2: Vec<u8> = b"val2".to_vec();
    let node2: Node<H256, Vec<u8>> = Node::new_leaf(key2, value2.clone());
    let hash2: H256 = node2.merkle_hash().into();
    node_store.put(hash2, node2.encode().unwrap()).unwrap();

    let registry = Registry::new();
    let metrics = Arc::new(StateDBMetrics::new(&registry));
    let bloom = Arc::new(Mutex::new(BloomFilter::new(1 << 16, 4)));

    // Build reachable set with hash1
    let builder = ReachableBuilder::new(
        Arc::new(node_store.clone()),
        None,
        bloom.clone(),
        metrics.clone(),
    );
    let scanned = builder.build(vec![hash1], 1).unwrap();
    assert_eq!(scanned, 1);

    // Sweep expired roots containing both hashes
    let sweeper = SweepExpired::new(
        Arc::new(node_store.clone()),
        None,
        bloom.clone(),
        metrics.clone(),
    );
    let deleted = sweeper.sweep(vec![hash1, hash2], 1).unwrap();
    assert_eq!(deleted, 1);

    // Validate storage
    assert!(node_store.get(&hash1).unwrap().is_some());
    assert!(node_store.get(&hash2).unwrap().is_none());
}

#[tokio::test]
async fn test_incremental_sweep() {
    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
    let node_store = store.get_state_node_store().clone();
    let instance = node_store.get_store().store().clone();
    let stale_store = StaleIndexStore::new(instance.clone());
    let ref_store = NodeRefcountStore::new(instance.clone());

    // Prepare a node
    let key = H256::random();
    let value: Vec<u8> = b"x".to_vec();
    let node: Node<H256, Vec<u8>> = Node::new_leaf(key, value.clone());
    let hash: H256 = node.merkle_hash().into();
    node_store.put(hash, node.encode().unwrap()).unwrap();

    // refcount ==1 then decrement via write_stale_indices => 0
    ref_store.inc(hash).unwrap();
    let root = H256::random();
    node_store.write_stale_indices(&[(root, hash)]).unwrap();
    assert_eq!(ref_store.get_ref(hash).unwrap(), 0);

    // incremental sweep should delete
    let sweeper = IncrementalSweep::new(
        Arc::new(node_store.clone()),
        Arc::new(stale_store.clone()),
        Arc::new(ref_store.clone()),
    );
    let deleted = sweeper.sweep(root, 100).unwrap();
    assert_eq!(deleted, 1);
    assert!(node_store.get(&hash).unwrap().is_none());
    assert!(stale_store.kv_get((root, hash)).unwrap().is_none());
}
