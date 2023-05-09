// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[test]
fn test_smt() {
    let node_store = InMemoryNodeStore::default();
    let smt = SMTree::new(node_store, None);
    let key = "key";
    let value = "value";
    let state_root = smt.put(key.to_string(), value.to_string()).unwrap();

    let (result, proof) = smt.get_with_proof(key.to_string()).unwrap();
    assert_eq!(result.unwrap(), value.to_string());
    assert!(proof
        .verify(state_root, key.to_string(), Some(value.to_string()))
        .is_ok());

    let (result, proof) = smt.get_with_proof("key2".to_string()).unwrap();
    assert_eq!(result, None);
    assert!(proof
        .verify::<String, String>(state_root, "key2".to_string(), None)
        .is_ok());

    let mut iter = smt.iter(None).unwrap();

    let item = iter.next();
    assert_eq!(item.unwrap().unwrap(), (key.to_string(), value.to_string()));

    let key2 = "key2".to_string();
    let value2 = "value2".to_string();
    let key3 = "key3".to_string();
    let value3 = "value3".to_string();

    let state_root = smt
        .puts(vec![
            (key2.clone(), Some(value2.clone())),
            (key3, Some(value3)),
        ])
        .unwrap();

    let (result, proof) = smt.get_with_proof("key2".to_string()).unwrap();
    assert_eq!(result, Some(value2.clone()));
    assert!(proof
        .verify::<String, String>(state_root, key2.clone(), Some(value2))
        .is_ok());

    let iter = smt.iter(None).unwrap();
    assert_eq!(iter.count(), 3);

    smt.remove(key2).unwrap();
    let iter = smt.iter(None).unwrap();
    assert_eq!(iter.count(), 2);
}
