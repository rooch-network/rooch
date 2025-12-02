// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::recycle_bin::{RecycleBinStore, RecycleFilter};
use anyhow::Result;
use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use std::sync::Arc;

#[tokio::test]
async fn test_recycle_bin_list_entries() -> Result<()> {
    let (store, _tmpdir) = MoveOSStore::mock_moveos_store()?;
    let store = Arc::new(store);

    // Create recycle bin store
    let recycle_store = store.get_node_recycle_store();
    let recycle_bin = RecycleBinStore::new(recycle_store.clone())?;

    // Initially empty
    let empty_entries = recycle_bin.list_entries(None, None)?;
    assert_eq!(empty_entries.len(), 0);

    // Create test records
    let key1 = H256::random();
    let key2 = H256::random();
    let key3 = H256::random();

    let record1 = recycle_bin.create_record(vec![1, 2, 3, 4]);

    let record2 = recycle_bin.create_record(vec![5, 6, 7, 8]);

    let record3 = recycle_bin.create_record(vec![9, 10, 11, 12]);

    // Put records
    recycle_bin.put_record(key1, record1.clone())?;
    recycle_bin.put_record(key2, record2.clone())?;
    recycle_bin.put_record(key3, record3.clone())?;

    // Test list all entries
    let all_entries = recycle_bin.list_entries(None, None)?;
    assert_eq!(all_entries.len(), 3);

    // Test list with limit
    let limited_entries = recycle_bin.list_entries(None, Some(2))?;
    assert_eq!(limited_entries.len(), 2);

    // Test list with time filter
    let filter = RecycleFilter {
        older_than: Some(1640995200 + 100), // Filter for older records
        newer_than: None,
        min_size: None,
        max_size: None,
    };
    let filtered_entries = recycle_bin.list_entries(Some(filter), None)?;
    assert_eq!(filtered_entries.len(), 3); // All records should match time filter

    // Test size filter
    let size_filter = RecycleFilter {
        older_than: None,
        newer_than: None,
        min_size: Some(5),
        max_size: None,
    };
    let size_filtered_entries = recycle_bin.list_entries(Some(size_filter), None)?;
    assert_eq!(size_filtered_entries.len(), 0); // All records are size 4

    Ok(())
}

#[tokio::test]
async fn test_recycle_bin_delete_record() -> Result<()> {
    let (store, _tmpdir) = MoveOSStore::mock_moveos_store()?;
    let store = Arc::new(store);

    // Create recycle bin store
    let recycle_store = store.get_node_recycle_store();
    let recycle_bin = RecycleBinStore::new(recycle_store.clone())?;

    // Create and put a record
    let key = H256::random();
    let record = recycle_bin.create_record(vec![1, 2, 3, 4, 5]);

    recycle_bin.put_record(key, record.clone())?;

    // Verify record exists
    let retrieved = recycle_bin.get_record(&key)?;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().bytes, record.bytes);

    // Delete the record
    let deleted = recycle_bin.delete_record(&key)?;
    assert!(deleted);

    // Verify record is deleted
    let should_be_none = recycle_bin.get_record(&key)?;
    assert!(should_be_none.is_none());

    // Try to delete non-existent record
    let not_deleted = recycle_bin.delete_record(&H256::random())?;
    assert!(!not_deleted);

    // Verify stats are updated correctly
    let stats = recycle_bin.get_stats();
    assert_eq!(stats.current_entries, 0);

    Ok(())
}

#[tokio::test]
async fn test_recycle_bin_delete_entries() -> Result<()> {
    let (store, _tmpdir) = MoveOSStore::mock_moveos_store()?;
    let store = Arc::new(store);

    // Create recycle bin store
    let recycle_store = store.get_node_recycle_store();
    let recycle_bin = RecycleBinStore::new(recycle_store.clone())?;

    // Create and put multiple records
    let keys = vec![
        H256::random(),
        H256::random(),
        H256::random(),
        H256::random(),
    ];

    // Put records
    for (i, &key) in keys.iter().enumerate() {
        let record = recycle_bin.create_record(vec![i as u8; 4]);
        recycle_bin.put_record(key, record)?;
    }

    // Verify all records exist
    let all_entries = recycle_bin.list_entries(None, None)?;
    assert_eq!(all_entries.len(), 4);

    // Delete entries by time filter (older than current time)
    let filter = RecycleFilter {
        older_than: Some(1640995200 + 1000), // Time-based filter
        newer_than: None,
        min_size: None,
        max_size: None,
    };

    let deleted_count = recycle_bin.delete_entries(&filter, 10)?;
    assert_eq!(deleted_count, 4); // All records should match the time filter

    // Verify remaining entries
    let remaining_entries = recycle_bin.list_entries(None, None)?;
    assert_eq!(remaining_entries.len(), 0); // All records should be deleted

    // Delete all remaining entries
    let all_filter = RecycleFilter {
        older_than: None,
        newer_than: None,
        min_size: None,
        max_size: None,
    };

    let final_deleted_count = recycle_bin.delete_entries(&all_filter, 10)?;
    assert_eq!(final_deleted_count, 0); // No remaining entries to delete

    // Verify empty
    let final_entries = recycle_bin.list_entries(None, None)?;
    assert_eq!(final_entries.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_recycle_bin_record_operations() -> Result<()> {
    let (store, _tmpdir) = MoveOSStore::mock_moveos_store()?;
    let store = Arc::new(store);

    // Create recycle bin store
    let recycle_store = store.get_node_recycle_store();
    let recycle_bin = RecycleBinStore::new(recycle_store.clone())?;

    // Create and put records
    let key1 = H256::random();
    let key2 = H256::random();

    let record1 = recycle_bin.create_record(vec![1, 2, 3]);

    let record2 = recycle_bin.create_record(vec![4, 5, 6]);

    recycle_bin.put_record(key1, record1.clone())?;
    recycle_bin.put_record(key2, record2.clone())?;

    // Test that we can retrieve both records individually
    let retrieved1 = recycle_bin.get_record(&key1)?.unwrap();
    let retrieved2 = recycle_bin.get_record(&key2)?.unwrap();

    assert_eq!(retrieved1.bytes, record1.bytes);
    assert_eq!(retrieved1.tx_order, record1.tx_order);
    assert_eq!(retrieved2.bytes, record2.bytes);
    assert_eq!(retrieved2.tx_order, record2.tx_order);

    Ok(())
}
