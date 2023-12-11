// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::vm::data_cache::{into_change_set, MoveosDataCache};
#[cfg(test)]
use crate::vm::unit_tests::vm_arguments_tests::{make_script_function, RemoteStore};
use move_binary_format::file_format::{Signature, SignatureToken};
use move_vm_runtime::data_cache::TransactionCache;
use move_vm_runtime::move_vm::MoveVM;
use moveos_stdlib::natives::moveos_stdlib::raw_table::TableData;
use parking_lot::RwLock;
use std::sync::Arc;

#[test]
fn publish_and_load_module() {
    let signature = Signature(vec![SignatureToken::U8]);
    let (module, _) = make_script_function(signature);

    let mut bytes = vec![];
    let module_id = module.self_id();
    module.serialize(&mut bytes).unwrap();

    let move_vm = MoveVM::new(vec![]).unwrap();
    let remote_view = RemoteStore::new();
    let loader = move_vm.runtime.loader();
    let table_data = Arc::new(RwLock::new(TableData::default()));
    let mut data_cache = MoveosDataCache::new(&remote_view, loader, table_data.clone());

    // check
    assert!(!data_cache.exists_module(&module_id).unwrap());
    data_cache
        .publish_module(&module_id, bytes.clone(), false)
        .unwrap();
    assert!(data_cache.exists_module(&module_id).unwrap());
    let loaded_bytes = data_cache.load_module(&module_id).unwrap();
    assert_eq!(loaded_bytes, bytes);

    drop(data_cache);
    let changes = into_change_set(table_data).unwrap();
    drop(changes);
}
