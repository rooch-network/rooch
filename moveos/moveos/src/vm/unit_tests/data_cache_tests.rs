// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::vm::data_cache::{into_change_set, MoveosDataCache};
#[cfg(test)]
use crate::vm::unit_tests::vm_arguments_tests::{make_script_function, RemoteStore};
use move_binary_format::file_format::{Signature, SignatureToken};
use move_vm_runtime::data_cache::TransactionCache;
use move_vm_runtime::move_vm::MoveVM;
use moveos_object_runtime::runtime::ObjectRuntime;
use moveos_types::{
    moveos_std::{
        module_store::ModuleStore, object::ObjectMeta, timestamp::Timestamp, tx_context::TxContext,
    },
    state::{MoveState, ObjectState},
};
use parking_lot::RwLock;
use std::rc::Rc;

#[test]
#[allow(clippy::arc_with_non_send_sync)]
fn publish_and_load_module() {
    let signature = Signature(vec![SignatureToken::U8]);
    let (module, _) = make_script_function(signature);

    let mut bytes = vec![];
    let module_id = module.self_id();
    module.serialize(&mut bytes).unwrap();

    let move_vm = MoveVM::new(vec![]).unwrap();
    let remote_view = RemoteStore::new();
    let loader = move_vm.runtime.loader();
    let object_runtime = Rc::new(RwLock::new(ObjectRuntime::genesis(
        TxContext::random_for_testing_only(),
        ObjectMeta::genesis_root(),
        &remote_view,
        vec![
            (
                ObjectState::new_timestamp(Timestamp { milliseconds: 0 }),
                Timestamp::type_layout(),
            ),
            (
                ObjectState::genesis_module_store(),
                ModuleStore::type_layout(),
            ),
        ],
    )));

    let mut data_cache = MoveosDataCache::new(&remote_view, loader, object_runtime.clone());

    // check
    assert!(!data_cache.exists_module(&module_id).unwrap());
    data_cache
        .publish_module(&module_id, bytes.clone(), false)
        .unwrap();
    assert!(data_cache.exists_module(&module_id).unwrap());
    let loaded_bytes = data_cache.load_module(&module_id).unwrap();
    assert_eq!(loaded_bytes, bytes);

    drop(data_cache);
    let (_tx_context, changes) = into_change_set(object_runtime).unwrap();
    drop(changes);
}
