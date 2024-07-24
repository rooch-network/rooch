// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[test]
fn test_smt() {
    let node_store = InMemoryNodeStore::default();
    let registry = prometheus::Registry::new();
    let smt = SMTree::new(node_store.clone(), &registry);
    let key = H256::random();
    let value = "value";
    let genesis_root = *SPARSE_MERKLE_PLACEHOLDER_HASH;
    let changeset = smt.put(genesis_root, key, value.to_string()).unwrap();
    node_store.write_nodes(changeset.nodes).unwrap();
    let (result, proof) = smt.get_with_proof(changeset.state_root, key).unwrap();
    assert_eq!(result.unwrap(), value.to_string());
    assert!(proof
        .verify(changeset.state_root, key, Some(value.to_string()))
        .is_ok());
    let key2 = H256::random();

    let (result, proof) = smt.get_with_proof(changeset.state_root, key2).unwrap();
    assert_eq!(result, None);
    assert!(proof
        .verify::<H256, String>(changeset.state_root, key2, None)
        .is_ok());

    let mut iter = smt.iter(changeset.state_root, None).unwrap();

    let item = iter.next();
    assert_eq!(item.unwrap().unwrap(), (key, value.to_string()));

    let value2 = "value2".to_owned();
    let key3 = H256::random();
    let value3 = "value3".to_owned();

    let changeset2 = smt
        .puts(
            changeset.state_root,
            vec![(key2, Some(value2.clone())), (key3, Some(value3))],
        )
        .unwrap();

    node_store.write_nodes(changeset2.nodes).unwrap();
    let (result, proof) = smt.get_with_proof(changeset2.state_root, key2).unwrap();
    assert_eq!(result, Some(value2.clone()));
    assert!(proof
        .verify::<H256, String>(changeset2.state_root, key2, Some(value2))
        .is_ok());

    let iter = smt.iter(changeset2.state_root, None).unwrap();
    assert_eq!(iter.count(), 3);

    let changeset3 = smt.remove(changeset2.state_root, key2).unwrap();
    node_store.write_nodes(changeset3.nodes).unwrap();
    let iter = smt.iter(changeset3.state_root, None).unwrap();
    assert_eq!(iter.count(), 2);
}
