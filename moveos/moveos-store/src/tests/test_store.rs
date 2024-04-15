// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

extern crate chrono;

use crate::event_store::EventStore;
use crate::MoveOSStore;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{StructTag, TypeTag};
use move_core_types::vm_status::KeptVMStatus;
use moveos_config::store_config::RocksdbConfig;
use moveos_types::h256::H256;
use moveos_types::moveos_std::event::TransactionEvent;
use moveos_types::transaction::TransactionExecutionInfo;
use raw_store::rocks::{RocksDB, DEFAULT_PREFIX_NAME};
use raw_store::traits::DBStore;
use raw_store::CodecKVStore;

#[test]
fn test_reopen() {
    let tmpdir = moveos_config::temp_dir();
    let key = H256::random();
    let value = H256::zero();
    let cfs = vec![DEFAULT_PREFIX_NAME];
    {
        let db = RocksDB::new(tmpdir.path(), cfs.clone(), RocksdbConfig::default(), None).unwrap();
        db.put(
            DEFAULT_PREFIX_NAME,
            bcs::to_bytes(&key).unwrap(),
            bcs::to_bytes(&value).unwrap(),
        )
        .unwrap();
        assert_eq!(
            db.get(DEFAULT_PREFIX_NAME, bcs::to_bytes(&key).unwrap())
                .unwrap(),
            Some(bcs::to_bytes(&value).unwrap())
        );
    }
    {
        let db = RocksDB::new(tmpdir.path(), cfs, RocksdbConfig::default(), None).unwrap();
        assert_eq!(
            db.get(DEFAULT_PREFIX_NAME, bcs::to_bytes(&key).unwrap())
                .unwrap(),
            Some(bcs::to_bytes(&value).unwrap())
        );
    }
}

#[test]
fn test_open_read_only() {
    let tmpdir = moveos_config::temp_dir();
    let cfs = vec![DEFAULT_PREFIX_NAME];
    let db = RocksDB::new(tmpdir.path(), cfs.clone(), RocksdbConfig::default(), None).unwrap();
    let key = H256::random();
    let value = H256::zero();
    let result = db.put(
        DEFAULT_PREFIX_NAME,
        bcs::to_bytes(&key).unwrap(),
        bcs::to_bytes(&value).unwrap(),
    );
    assert!(result.is_ok());
    let path = tmpdir.as_ref();
    let db = RocksDB::open_with_cfs(path, cfs, true, RocksdbConfig::default(), None).unwrap();
    let result = db.put(
        DEFAULT_PREFIX_NAME,
        bcs::to_bytes(&key).unwrap(),
        bcs::to_bytes(&value).unwrap(),
    );
    assert!(result.is_err());
    let result = db
        .get(DEFAULT_PREFIX_NAME, bcs::to_bytes(&key).unwrap())
        .unwrap();
    assert_eq!(result, Some(bcs::to_bytes(&value).unwrap()));
}

#[test]
fn test_store() {
    let store = MoveOSStore::mock_moveos_store().unwrap();

    let transaction_info1 = TransactionExecutionInfo::new(
        H256::random(),
        H256::random(),
        rand::random(),
        H256::random(),
        rand::random(),
        KeptVMStatus::Executed,
    );
    let id = transaction_info1.tx_hash;
    store
        .get_transaction_store()
        .kv_put(id, transaction_info1.clone())
        .unwrap();
    let transaction_info2 = store.get_transaction_store().kv_get(id).unwrap();
    assert!(transaction_info2.is_some());
    assert_eq!(transaction_info1, transaction_info2.unwrap());
}

#[test]
fn test_event_store() {
    let store = MoveOSStore::mock_moveos_store().unwrap();

    let test_struct_tag = StructTag {
        address: AccountAddress::random(),
        module: Identifier::new("Module").unwrap(),
        name: Identifier::new("Name").unwrap(),
        type_params: vec![TypeTag::Bool],
    };
    let tx_events = vec![
        TransactionEvent::new(test_struct_tag.clone(), b"data0".to_vec(), 100),
        TransactionEvent::new(test_struct_tag, b"data1".to_vec(), 101),
    ];

    let event_ids = store.save_events(tx_events.clone()).unwrap();
    assert_eq!(event_ids.len(), 2);
    let event0 = store
        .get_event(event_ids.first().cloned().unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(event0.event_type, tx_events[0].event_type);
    assert_eq!(event0.event_data, tx_events[0].event_data);
    assert_eq!(event0.event_index, tx_events[0].event_index);
    assert_eq!(event0.event_id.event_seq, 0);

    let event1 = store
        .get_event(event_ids.get(1).cloned().unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(event1.event_type, tx_events[1].event_type);
    assert_eq!(event1.event_data, tx_events[1].event_data);
    assert_eq!(event1.event_index, tx_events[1].event_index);
    assert_eq!(event1.event_id.event_seq, 1);
}

#[test]
fn test_iter() {
    let store = MoveOSStore::mock_moveos_store().unwrap();
    let transaction_info1 = TransactionExecutionInfo::new(
        H256::random(),
        H256::random(),
        rand::random(),
        H256::random(),
        rand::random(),
        KeptVMStatus::Executed,
    );
    let id = transaction_info1.tx_hash;
    store
        .get_transaction_store()
        .kv_put(id, transaction_info1.clone())
        .unwrap();
    let mut iter = store.get_transaction_store().iter().unwrap();
    iter.seek_to_first();
    let item2 = iter.next().and_then(|item| item.ok());
    assert!(item2.is_some());
    let (_, transaction_info2) = item2.unwrap();
    assert_eq!(transaction_info1, transaction_info2);
    assert!(iter.next().is_none());
}
