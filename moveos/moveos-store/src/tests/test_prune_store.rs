// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::prune::PruneStore;
use crate::MoveOSStore;
use moveos_common::bloom_filter::BloomFilter;
use moveos_types::prune::PrunePhase;
use primitive_types::H256;

// Verify that prune meta phase can be persisted and reloaded.
#[tokio::test]
async fn test_prune_meta_phase_persist() {
    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();

    // Default should be BuildReach
    let phase = store.prune_store.load_prune_meta_phase().unwrap();
    assert_eq!(phase, PrunePhase::BuildReach);

    // Save SweepExpired and reload
    store
        .save_prune_meta_phase(PrunePhase::SweepExpired)
        .unwrap();
    let phase2 = store.load_prune_meta_phase().unwrap();
    assert_eq!(phase2, PrunePhase::SweepExpired);
}

// Verify bloom snapshot persistence round-trip.
#[tokio::test]
async fn test_prune_meta_bloom_persist() {
    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();

    let mut bloom = BloomFilter::new(1 << 20, 4);
    let hash = H256::random();
    bloom.insert(&hash);

    store.save_prune_meta_bloom(bloom.clone()).unwrap();
    let loaded = store.load_prune_meta_bloom().unwrap().unwrap();
    assert!(loaded.contains(&hash));
}

// // Verify list_before returns indices whose root is smaller than cutoff_root.
// #[tokio::test]
// async fn test_list_before() {
//     let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();
//     let prune_store = store.get_prune_store();
//
//     // Manually write some stale indices with different roots
//     let now = chrono::Utc::now().timestamp_millis() as u64;
//
//     let old_root = H256::from_low_u64_be(now - 100);
//     let mid_root = H256::from_low_u64_be(now - 10);
//     let new_root = H256::from_low_u64_be(now + 10);
//
//     let node1 = H256::random();
//     let node2 = H256::random();
//     let node3 = H256::random();
//
//     // Put directly into stale_index_store
//     prune_store
//         .write_stale_indices(vec![(old_root, node1)].as_slice())
//         .unwrap();
//     prune_store
//         .write_stale_indices(vec![(mid_root, node2)].as_slice())
//         .unwrap();
//     prune_store
//         .write_stale_indices(vec![(new_root, node3)].as_slice())
//         .unwrap();
//
//     let cutoff = H256::from_low_u64_be(now);
//     let mut list = prune_store.list_before(cutoff, 10).unwrap();
//     list.sort();
//
//     assert_eq!(list.len(), 2);
//     assert!(list.contains(&(old_root, node1)));
//     assert!(list.contains(&(mid_root, node2)));
//     assert!(!list.contains(&(new_root, node3)));
// }

/// Verify that deleted roots bloom filter can be persisted and reloaded.
#[tokio::test]
async fn test_deleted_state_root_bloom_persist() {
    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();

    // Initially, no bloom filter should exist
    let loaded = store.load_deleted_state_root_bloom().unwrap();
    assert!(loaded.is_none());

    // Create a bloom filter and mark some roots as deleted
    let mut bloom = BloomFilter::new(1 << 20, 4);
    let root1 = H256::random();
    let root2 = H256::random();
    let root3 = H256::random();

    bloom.insert(&root1);
    bloom.insert(&root2);

    // Save the bloom filter
    store.save_deleted_state_root_bloom(bloom.clone()).unwrap();

    // Reload and verify
    let loaded = store.load_deleted_state_root_bloom().unwrap().unwrap();
    assert!(loaded.contains(&root1));
    assert!(loaded.contains(&root2));
    assert!(!loaded.contains(&root3)); // This one was not inserted

    // Add more roots and save again
    let mut bloom2 = loaded;
    bloom2.insert(&root3);
    store.save_deleted_state_root_bloom(bloom2.clone()).unwrap();

    // Reload and verify all three are present
    let loaded2 = store.load_deleted_state_root_bloom().unwrap().unwrap();
    assert!(loaded2.contains(&root1));
    assert!(loaded2.contains(&root2));
    assert!(loaded2.contains(&root3));
}

/// Test that the bloom filter correctly filters duplicate state roots
#[tokio::test]
async fn test_deleted_state_root_bloom_deduplication() {
    let (store, _tmpdir) = MoveOSStore::mock_moveos_store().unwrap();

    let mut bloom = BloomFilter::new(1 << 20, 4);

    // Generate some state roots
    let roots: Vec<H256> = (0..100).map(|_| H256::random()).collect();

    // Mark first 50 as deleted
    for root in roots.iter().take(50) {
        bloom.insert(root);
    }

    store.save_deleted_state_root_bloom(bloom).unwrap();

    // Load and filter
    let loaded_bloom = store.load_deleted_state_root_bloom().unwrap().unwrap();

    let roots_to_process: Vec<_> = roots
        .iter()
        .filter(|root| !loaded_bloom.contains(root))
        .collect();

    // Should only have the last 50 roots
    assert_eq!(roots_to_process.len(), 50);

    // Verify the filtered roots are the correct ones
    // First 50 roots were marked deleted, so they should NOT be in roots_to_process
    for item in roots.iter().take(50) {
        assert!(!roots_to_process.contains(&item));
    }
    // Last 50 roots were NOT marked deleted, so they SHOULD be in roots_to_process
    for item in roots.iter().skip(50) {
        assert!(roots_to_process.contains(&item));
    }
}
