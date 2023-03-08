// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::hash::HashValue;
use super::{
    mock_tree_store::{MockTestStore, TestKey, TestValue},
    JellyfishMerkleTree,
};
use crate::EncodeToObject;
use std::collections::HashMap;

/// Computes the key immediately after `key`.
pub fn plus_one(key: HashValue) -> HashValue {
    assert_ne!(key, HashValue::new([0xff; HashValue::LENGTH]));

    let mut buf = key.to_vec();
    for i in (0..HashValue::LENGTH).rev() {
        if buf[i] == 255 {
            buf[i] = 0;
        } else {
            buf[i] += 1;
            break;
        }
    }
    HashValue::from_slice(&buf).unwrap()
}

/// Initializes a DB with a set of key-value pairs by inserting one key at each version.
#[allow(clippy::all)]
pub fn init_mock_db(kvs: &HashMap<TestKey, TestValue>) -> (MockTestStore, Option<HashValue>) {
    assert!(!kvs.is_empty());

    let db = MockTestStore::new_test();
    let tree: JellyfishMerkleTree<TestKey, TestValue, MockTestStore> =
        JellyfishMerkleTree::new(&db);
    let mut current_state_root = None;
    for (_i, (key, value)) in kvs.iter().enumerate() {
        let (_root_hash, write_batch) = tree
            .insert_all(
                current_state_root,
                vec![(key.clone().into_object(), value.clone().into_object())],
            )
            .unwrap();
        db.write_tree_update_batch(write_batch).unwrap();
        current_state_root = Some(_root_hash);
    }

    (db, current_state_root)
}
