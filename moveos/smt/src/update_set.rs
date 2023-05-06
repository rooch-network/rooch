// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{Key, SMTObject, Value};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct UpdateSet<K, V> {
    updates: BTreeMap<K, Option<V>>,
}

impl<K, V> UpdateSet<K, V>
where
    K: Key,
    V: Value,
{
    pub fn new() -> Self {
        Self {
            updates: BTreeMap::new(),
        }
    }

    pub fn new_with_value(key: K, value: Option<V>) -> Self {
        let mut updates = BTreeMap::new();
        updates.insert(key, value);
        Self { updates }
    }

    /// Add a put operation to the batch.
    pub fn put(&mut self, key: K, value: V) {
        self.updates.insert(key, Some(value));
    }

    /// Add batch puts operation to the batch.
    pub fn puts(&mut self, updates: impl Iterator<Item = (K, Option<V>)>) {
        for (key, value) in updates {
            self.updates.insert(key, value);
        }
    }

    /// Add a remove operation to the batch.
    pub fn remove(&mut self, key: K) {
        self.updates.insert(key, None);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &Option<V>)> {
        self.updates.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut Option<V>)> {
        self.updates.iter_mut()
    }

    pub fn is_empty(&self) -> bool {
        self.updates.is_empty()
    }

    pub fn len(&self) -> usize {
        self.updates.len()
    }

    pub fn clear(&mut self) {
        self.updates.clear();
    }

    pub(crate) fn into_updates(self) -> Vec<(SMTObject<K>, Option<SMTObject<V>>)> {
        self.into_iter()
            .map(|(k, v)| (k.into_object(), v.map(|v| v.into_object())))
            .collect()
    }
}

impl<K, V> Default for UpdateSet<K, V>
where
    K: Key,
    V: Value,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> IntoIterator for UpdateSet<K, V> {
    type Item = (K, Option<V>);
    type IntoIter = std::collections::btree_map::IntoIter<K, Option<V>>;

    fn into_iter(self) -> Self::IntoIter {
        self.updates.into_iter()
    }
}

impl<K, V> From<(K, Option<V>)> for UpdateSet<K, V>
where
    K: Key,
    V: Value,
{
    fn from(update: (K, Option<V>)) -> Self {
        Self::new_with_value(update.0, update.1)
    }
}

impl<K, V> From<(K, V)> for UpdateSet<K, V>
where
    K: Key,
    V: Value,
{
    fn from(update: (K, V)) -> Self {
        Self::new_with_value(update.0, Some(update.1))
    }
}

impl<K, V> From<Vec<(K, Option<V>)>> for UpdateSet<K, V>
where
    K: Key,
    V: Value,
{
    fn from(updates: Vec<(K, Option<V>)>) -> Self {
        let mut update_set = Self::new();
        update_set.puts(updates.into_iter());
        update_set
    }
}
