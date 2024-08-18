// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{MoveOSStore, StoreMeta};
use anyhow::Result;
use moveos_config::store_config::RocksdbConfig;
use moveos_types::h256::H256;
use moveos_types::test_utils::random_state_change_set;
use raw_store::metrics::DBMetrics;
use raw_store::rocks::RocksDB;
use raw_store::{StoreInstance, CF_METRICS_REPORT_PERIOD_MILLIS};
use smt::NodeReader;
use std::time::Duration;

#[tokio::test]
async fn test_reopen() -> Result<()> {
    let temp_dir = moveos_config::temp_dir();
    let registry_service = metrics::RegistryService::default();
    DBMetrics::init(&registry_service.default_registry());

    let key = H256::random();
    let node = b"testnode".to_vec();
    {
        let db_metrics = DBMetrics::get_or_init(&registry_service.default_registry()).clone();
        let mut store_instance = StoreInstance::new_db_instance(
            RocksDB::new(
                temp_dir.path(),
                StoreMeta::get_column_family_names().to_vec(),
                RocksdbConfig::default(),
            )?,
            db_metrics,
        );
        let moveos_store = MoveOSStore::new_with_instance(
            store_instance.clone(),
            &registry_service.default_registry(),
        )
        .unwrap();
        let node_store = moveos_store.get_state_node_store();
        node_store
            .put(key, node.clone())
            .map_err(|e| anyhow::anyhow!("test_state_store test_reopen error: {:?}", e))
            .ok();
        assert_eq!(node_store.get(&key).unwrap(), Some(node.clone()));
        let _ = store_instance.cancel_metrics_task();
        // Wait for rocksdb cancel metrics task to avoid db lock
        tokio::time::sleep(Duration::from_millis(CF_METRICS_REPORT_PERIOD_MILLIS)).await;
    }
    {
        // To aviod AlreadyReg for re open the same db
        let new_registry = prometheus::Registry::new();
        let moveos_store = MoveOSStore::new(temp_dir.path(), &new_registry).unwrap();
        let node_store = moveos_store.get_state_node_store();
        assert_eq!(node_store.get(&key).unwrap(), Some(node));
    }
    Ok(())
}

#[tokio::test]
async fn test_statedb_state_root() -> Result<()> {
    let (moveos_store, _) =
        MoveOSStore::mock_moveos_store().expect("moveos store mock should succ");
    let mut change_set = random_state_change_set();
    moveos_store
        .get_state_store()
        .apply_change_set(&mut change_set)?;
    let mut new_change_set = random_state_change_set();
    moveos_store
        .get_state_store()
        .apply_change_set(&mut new_change_set)?;
    assert_ne!(change_set.state_root, new_change_set.state_root);
    Ok(())
}

// #[tokio::test]
// async fn test_child_state_db_dump_and_apply() -> Result<()> {
//     let mut moveos_store = MoveOSStore::mock_moveos_store().expect("moveos store mock should succ");
//
//     let base_state_change_set = random_state_change_set();
//     let (new_state_root, global_size) = moveos_store
//         .get_state_store_mut()
//         .apply_change_set(base_state_change_set)?;
//
//     let parent_id = ObjectID::from(AccountAddress::random());
//     let mut state_change_set = random_state_change_set_for_child_object(parent_id.clone());
//     state_change_set.global_size += global_size;
//     state_change_set.state_root = new_state_root;
//     let (new_state_root, _global_size) = moveos_store
//         .get_state_store_mut()
//         .apply_change_set(state_change_set)?;
//
//     let mut dump_state_change_set = StateChangeSet::default();
//     let (child_object_state, _next_key) = moveos_store.get_state_store().dump_child_object_states(
//         parent_id.clone(),
//         new_state_root,
//         None,
//         true,
//     )?;
//     for object_state in child_object_state.clone() {
//         let (field_states, _next_key) = moveos_store.get_state_store().dump_field_states(
//             object_state.object_id.clone(),
//             object_state.state_root,
//             None,
//         )?;
//
//         let object_id = object_state.object_id.clone();
//         let mut child_object_change = object_state.object_change.clone();
//         //reset object state root for ObjectChange
//         child_object_change.reset_state_root(*GENESIS_STATE_ROOT)?;
//
//         child_object_change.add_field_changes(field_states);
//         dump_state_change_set
//             .changes
//             .insert(object_id, child_object_change);
//     }
//
//     let mut moveos_store2 =
//         MoveOSStore::mock_moveos_store().expect("moveos store mock should succ");
//     let (new_state_root, _global_size) = moveos_store2
//         .get_state_store_mut()
//         .apply_change_set(dump_state_change_set.clone())?;
//     let (new_child_object_state, _next_key) = moveos_store2
//         .get_state_store()
//         .dump_child_object_states(parent_id, new_state_root, None, true)?;
//     for (idx, new_object_state) in new_child_object_state.iter().enumerate() {
//         assert_eq!(
//             new_object_state.state_root,
//             child_object_state[idx].state_root
//         );
//     }
//     Ok(())
// }
