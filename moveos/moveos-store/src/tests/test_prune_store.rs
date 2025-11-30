// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::MoveOSStore;
use moveos_types::h256::{StoreKeyH256, H256};
use raw_store::CodecKVStore;

// Test that StoreKeyH256 correctly serializes to raw 32 bytes for database storage
#[tokio::test]
async fn test_store_key_h256_fixes_serialization() {
    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();

    let test_hash = H256::random();
    let store_key: StoreKeyH256 = test_hash.into();

    // Test StoreKeyH256 serialization directly
    let serialized_store_key = moveos_common::utils::to_bytes(&store_key).unwrap();

    // StoreKeyH256 should serialize to exactly 32 bytes (raw H256 bytes)
    assert_eq!(
        serialized_store_key.len(),
        32,
        "StoreKeyH256 should be 32 bytes"
    );
    assert_eq!(
        serialized_store_key,
        test_hash.as_bytes(),
        "StoreKeyH256 should match raw H256 bytes"
    );

    // Test that the three fixed stores now use StoreKeyH256 correctly

    // 1. NodeRefcountStore
    store.prune_store.inc_node_refcount(test_hash).unwrap();
    let refcount = store.prune_store.get_node_refcount(test_hash).unwrap();
    assert_eq!(
        refcount,
        Some(1),
        "NodeRefcountStore should work with StoreKeyH256"
    );

    // 2. ReachSeenDBStore
    let test_data = vec![1u8, 2u8, 3u8];
    store
        .prune_store
        .reach_seen_store
        .kv_put(store_key, test_data.clone())
        .unwrap();
    let retrieved_data = store
        .prune_store
        .reach_seen_store
        .kv_get(store_key)
        .unwrap();
    assert_eq!(
        retrieved_data,
        Some(test_data),
        "ReachSeenDBStore should work with StoreKeyH256"
    );

    // 3. StaleIndexStore
    let order_hash = H256::random();
    let order_key: StoreKeyH256 = order_hash.into();
    store
        .prune_store
        .stale_index_store
        .kv_put((order_key, store_key), vec![])
        .unwrap();
    let retrieved = store
        .prune_store
        .stale_index_store
        .kv_get((order_key, store_key))
        .unwrap();
    assert_eq!(
        retrieved,
        Some(vec![]),
        "StaleIndexStore should work with StoreKeyH256"
    );
}
