// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::hash::HashValue;
use super::{
    node_type::{Node, NodeKey},
    NodeBatch, StaleNodeIndex, TreeReader, TreeUpdateBatch, TreeWriter,
};
use crate::{DecodeToObject, EncodeToObject, Key, SMTObject, Value};
use anyhow::{bail, ensure, Result};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, BTreeSet, HashMap},
    sync::RwLock,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct TestKey(pub HashValue);

impl TestKey {
    pub fn new(value: [u8; HashValue::LENGTH]) -> TestKey {
        Self(HashValue::new(value))
    }

    pub fn new_with_hash(hash_value: HashValue) -> TestKey {
        Self(hash_value)
    }

    pub fn random() -> TestKey {
        Self::new_with_hash(HashValue::random())
    }

    pub fn to_vec(self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn into_object(self) -> SMTObject<Self> {
        let raw = self.0.to_vec();
        let hash = self.0;
        SMTObject::new_for_test(self, raw, hash)
    }
}

impl EncodeToObject for TestKey {
    fn into_object(self) -> SMTObject<Self>
    where
        Self: std::marker::Sized,
    {
        self.into_object()
    }
}

impl DecodeToObject for TestKey {
    fn from_raw(raw: Vec<u8>) -> Result<SMTObject<Self>>
    where
        Self: std::marker::Sized,
    {
        let key = TestKey::new_with_hash(HashValue::from_slice(&raw).unwrap());
        Ok(key.into_object())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct TestValue {
    pub value: Vec<u8>,
}

impl TestValue {
    pub fn random() -> Self {
        Self {
            value: HashValue::random().to_vec(),
        }
    }
}

impl From<Vec<u8>> for TestValue {
    fn from(value: Vec<u8>) -> Self {
        Self { value }
    }
}

#[derive(Default)]
pub struct MockTreeStore<K, V>(RwLock<(HashMap<NodeKey, Node<K, V>>, BTreeSet<StaleNodeIndex>)>);

pub type MockTestStore = MockTreeStore<TestKey, TestValue>;

impl MockTestStore {
    pub fn new_test() -> Self {
        MockTreeStore(RwLock::new((HashMap::new(), BTreeSet::new())))
    }
}

impl<K, V> TreeReader<K, V> for MockTreeStore<K, V>
where
    K: Key,
    V: Value,
{
    fn get_node_option(&self, node_key: &NodeKey) -> Result<Option<Node<K, V>>> {
        Ok(self.0.read().unwrap().0.get(node_key).cloned())
    }
}

impl<K, V> TreeWriter<K, V> for MockTreeStore<K, V>
where
    K: Key,
    V: Value,
{
    fn write_node_batch(&self, node_batch: &NodeBatch<K, V>) -> Result<()> {
        let mut locked = self.0.write().unwrap();
        for (node_key, node) in node_batch.clone() {
            ensure!(locked.0.insert(node_key, node).is_none());
        }
        Ok(())
    }
}

impl<K, V> MockTreeStore<K, V> {
    pub fn put_node(&self, node_key: NodeKey, node: Node<K, V>) -> Result<()> {
        match self.0.write().unwrap().0.entry(node_key) {
            Entry::Occupied(o) => bail!("Key {:?} exists.", o.key()),
            Entry::Vacant(v) => {
                v.insert(node);
            }
        }
        Ok(())
    }

    fn put_stale_node_index(&self, index: StaleNodeIndex) -> Result<()> {
        let is_new_entry = self.0.write().unwrap().1.insert(index);
        ensure!(is_new_entry, "Duplicated retire log.");
        Ok(())
    }

    pub fn write_tree_update_batch(&self, batch: TreeUpdateBatch<K, V>) -> Result<()> {
        batch
            .node_batch
            .into_iter()
            .map(|(k, v)| self.put_node(k, v))
            .collect::<Result<Vec<_>>>()?;
        batch
            .stale_node_index_batch
            .into_iter()
            .map(|i| self.put_stale_node_index(i))
            .collect::<Result<Vec<_>>>()?;
        Ok(())
    }

    pub fn purge_stale_nodes(&self, state_root_hash: HashValue) -> Result<()> {
        let mut wlocked = self.0.write().unwrap();

        // Only records retired before or at `least_readable_version` can be purged in order
        // to keep that version still readable.
        let to_prune = wlocked
            .1
            .iter()
            .take_while(|log| log.stale_since_version == state_root_hash)
            .cloned()
            .collect::<Vec<_>>();

        for log in to_prune {
            let removed = wlocked.0.remove(&log.node_key).is_some();
            ensure!(removed, "Stale node index refers to non-existent node.");
            wlocked.1.remove(&log);
        }

        Ok(())
    }

    pub fn num_nodes(&self) -> usize {
        self.0.read().unwrap().0.len()
    }
}
