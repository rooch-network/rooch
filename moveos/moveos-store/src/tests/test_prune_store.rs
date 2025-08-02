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
