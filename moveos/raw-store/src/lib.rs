// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod errors;
pub mod metrics;
pub mod rocks;
pub mod store_macros;
pub mod traits;

use crate::metrics::DBMetrics;
use crate::rocks::batch::{WriteBatch, WriteBatchCF};
use crate::rocks::{RocksDB, SchemaIterator};
use crate::traits::{DBStore, KVStore};
use anyhow::{bail, format_err, Result};
use moveos_common::utils::{from_bytes, to_bytes};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::convert::TryInto;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

/// Type alias to improve readability.
pub type ColumnFamilyName = &'static str;

///Store instance type define
#[derive(Clone)]
#[allow(clippy::upper_case_acronyms)]
pub enum StoreInstance {
    DB {
        db: Arc<RocksDB>,
        db_metrics: Arc<DBMetrics>,
    },
}

unsafe impl Send for StoreInstance {}

impl StoreInstance {
    pub fn new_db_instance(db: RocksDB, db_metrics: Arc<DBMetrics>) -> Self {
        let db_arc = Arc::new(db);
        Self::DB {
            db: db_arc,
            db_metrics,
        }
    }

    pub fn db(&self) -> Option<&RocksDB> {
        match self {
            StoreInstance::DB { db, db_metrics: _ } => Some(db.as_ref()),
        }
    }

    pub fn db_metrics(&self) -> Option<&DBMetrics> {
        match self {
            StoreInstance::DB { db: _, db_metrics } => Some(db_metrics.as_ref()),
        }
    }

    pub fn db_mut(&mut self) -> Option<&mut RocksDB> {
        match self {
            StoreInstance::DB { db, db_metrics: _ } => Arc::get_mut(db),
        }
    }

    pub fn db_metrics_mut(&mut self) -> Option<&mut DBMetrics> {
        match self {
            StoreInstance::DB { db: _, db_metrics } => Arc::get_mut(db_metrics),
        }
    }
}

impl DBStore for StoreInstance {
    fn get(&self, cf_name: &str, key: &[u8]) -> Result<Option<Vec<u8>>> {
        match self {
            StoreInstance::DB { db, db_metrics } => {
                let _timer = db_metrics
                    .raw_store_metrics
                    .raw_store_get_latency_seconds
                    .with_label_values(&[cf_name])
                    .start_timer();
                let res = db.get(cf_name, key)?;
                db_metrics
                    .raw_store_metrics
                    .raw_store_get_bytes
                    .with_label_values(&[cf_name])
                    .observe(res.as_ref().map_or(0.0, |v| v.len() as f64));
                Ok(res)
            }
        }
    }

    fn put(&self, cf_name: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        match self {
            StoreInstance::DB { db, db_metrics } => {
                let _timer = db_metrics
                    .raw_store_metrics
                    .raw_store_put_latency_seconds
                    .with_label_values(&[cf_name])
                    .start_timer();
                let put_bytes = key.len() + value.len();
                db.put(cf_name, key, value)?;
                db_metrics
                    .raw_store_metrics
                    .raw_store_put_bytes
                    .with_label_values(&[cf_name])
                    .observe(put_bytes as f64);
                Ok(())
            }
        }
    }

    fn contains_key(&self, cf_name: &str, key: &[u8]) -> Result<bool> {
        match self {
            StoreInstance::DB { db, db_metrics } => {
                let _timer = db_metrics
                    .raw_store_metrics
                    .raw_store_get_latency_seconds
                    .with_label_values(&[cf_name])
                    .start_timer();
                let res = db.contains_key(cf_name, key)?;
                Ok(res)
            }
        }
    }

    fn may_contains_key(&self, cf_name: &str, key: &[u8]) -> Result<bool> {
        match self {
            StoreInstance::DB {
                db,
                db_metrics: _db_metrics,
            } => {
                let res = db.may_contains_key(cf_name, key)?;
                Ok(res)
            }
        }
    }

    fn remove(&self, cf_name: &str, key: Vec<u8>) -> Result<()> {
        match self {
            StoreInstance::DB { db, db_metrics } => {
                let _timer = db_metrics
                    .raw_store_metrics
                    .raw_store_get_latency_seconds
                    .with_label_values(&[cf_name])
                    .start_timer();
                db.remove(cf_name, key)?;
                db_metrics
                    .raw_store_metrics
                    .raw_store_deletes
                    .with_label_values(&[cf_name])
                    .inc();
                Ok(())
            }
        }
    }

    fn write_batch(&self, cf_name: &str, batch: WriteBatch) -> Result<()> {
        match self {
            StoreInstance::DB { db, db_metrics } => {
                let _timer = db_metrics
                    .raw_store_metrics
                    .raw_store_write_batch_latency_seconds
                    .with_label_values(&[cf_name])
                    .start_timer();
                let write_batch_bytes = batch.size_in_bytes();
                db.write_batch(cf_name, batch)?;
                db_metrics
                    .raw_store_metrics
                    .raw_store_write_batch_bytes
                    .with_label_values(&[cf_name])
                    .observe(write_batch_bytes as f64);
                Ok(())
            }
        }
    }

    fn get_len(&self) -> Result<u64> {
        bail!("DB instance not support get length method!")
    }

    fn keys(&self) -> Result<Vec<Vec<u8>>> {
        bail!("DB instance not support keys method!")
    }

    fn put_sync(&self, cf_name: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        match self {
            StoreInstance::DB { db, db_metrics } => {
                let _timer = db_metrics
                    .raw_store_metrics
                    .raw_store_put_sync_latency_seconds
                    .with_label_values(&[cf_name])
                    .start_timer();
                let put_bytes = key.len() + value.len();
                db.put_sync(cf_name, key, value)?;
                db_metrics
                    .raw_store_metrics
                    .raw_store_put_sync_bytes
                    .with_label_values(&[cf_name])
                    .observe(put_bytes as f64);
                Ok(())
            }
        }
    }

    fn write_batch_sync(&self, cf_name: &str, batch: WriteBatch) -> Result<()> {
        match self {
            StoreInstance::DB { db, db_metrics } => {
                let _timer = db_metrics
                    .raw_store_metrics
                    .raw_store_write_batch_sync_latency_seconds
                    .with_label_values(&[cf_name])
                    .start_timer();
                let write_batch_bytes = batch.size_in_bytes();
                db.write_batch_sync(cf_name, batch)?;
                db_metrics
                    .raw_store_metrics
                    .raw_store_write_batch_sync_bytes
                    .with_label_values(&[cf_name])
                    .observe(write_batch_bytes as f64);
                Ok(())
            }
        }
    }

    fn write_batch_across_cfs(
        &self,
        cf_names: Vec<&str>,
        batch: WriteBatch,
        sync: bool,
    ) -> Result<()> {
        match self {
            StoreInstance::DB { db, db_metrics } => {
                let _timer = if sync {
                    db_metrics
                        .raw_store_metrics
                        .raw_store_write_batch_sync_latency_seconds
                        .with_label_values(&["across_cfs"])
                        .start_timer()
                } else {
                    db_metrics
                        .raw_store_metrics
                        .raw_store_write_batch_latency_seconds
                        .with_label_values(&["across_cfs"])
                        .start_timer()
                };
                let write_batch_bytes = batch.size_in_bytes();
                db.write_batch_across_cfs(cf_names, batch, sync)?;
                if sync {
                    db_metrics
                        .raw_store_metrics
                        .raw_store_write_batch_sync_bytes
                        .with_label_values(&["across_cfs"])
                        .observe(write_batch_bytes as f64);
                } else {
                    db_metrics
                        .raw_store_metrics
                        .raw_store_write_batch_bytes
                        .with_label_values(&["across_cfs"])
                        .observe(write_batch_bytes as f64);
                }
                Ok(())
            }
        }
    }

    fn write_cf_batch(&self, cf_batches: Vec<WriteBatchCF>, sync: bool) -> Result<()> {
        match self {
            StoreInstance::DB { db, db_metrics } => {
                let _timer = if sync {
                    db_metrics
                        .raw_store_metrics
                        .raw_store_write_batch_sync_latency_seconds
                        .with_label_values(&["across_cfs"])
                        .start_timer()
                } else {
                    db_metrics
                        .raw_store_metrics
                        .raw_store_write_batch_latency_seconds
                        .with_label_values(&["across_cfs"])
                        .start_timer()
                };
                let write_batch_bytes = cf_batches
                    .iter()
                    .map(|cf_batch| cf_batch.batch.size_in_bytes())
                    .sum::<usize>();
                db.write_cf_batch(cf_batches, sync)?;
                if sync {
                    db_metrics
                        .raw_store_metrics
                        .raw_store_write_batch_sync_bytes
                        .with_label_values(&["across_cfs"])
                        .observe(write_batch_bytes as f64);
                } else {
                    db_metrics
                        .raw_store_metrics
                        .raw_store_write_batch_bytes
                        .with_label_values(&["across_cfs"])
                        .observe(write_batch_bytes as f64);
                }
                Ok(())
            }
        }
    }

    fn multi_get(&self, cf_name: &str, keys: Vec<Vec<u8>>) -> Result<Vec<Option<Vec<u8>>>> {
        match self {
            StoreInstance::DB { db, db_metrics } => {
                let _timer = db_metrics
                    .raw_store_metrics
                    .raw_store_multiget_latency_seconds
                    .with_label_values(&[cf_name])
                    .start_timer();
                let res = db.multi_get(cf_name, keys)?;
                let res_size = res.iter().flatten().map(|entry| entry.len()).sum::<usize>();
                db_metrics
                    .raw_store_metrics
                    .raw_store_multiget_bytes
                    .with_label_values(&[cf_name])
                    .observe(res_size as f64);
                Ok(res)
            }
        }
    }
}

pub trait ColumnFamily: Send + Sync {
    type Key;
    type Value;
    fn name() -> ColumnFamilyName;
}

/// Define inner store implement
#[derive(Clone)]
pub struct InnerStore<CF>
where
    CF: ColumnFamily,
{
    pub cf_name: ColumnFamilyName,
    instance: StoreInstance,
    cf: PhantomData<CF>,
}

impl<CF> InnerStore<CF>
where
    CF: ColumnFamily,
{
    pub fn new(instance: StoreInstance) -> Self {
        Self {
            instance,
            cf_name: CF::name(),
            cf: PhantomData,
        }
    }

    pub fn store(&self) -> &StoreInstance {
        &self.instance
    }
}

impl<CF> KVStore for InnerStore<CF>
where
    CF: ColumnFamily,
{
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.instance.get(self.cf_name, key)
    }

    fn multiple_get(&self, keys: Vec<Vec<u8>>) -> Result<Vec<Option<Vec<u8>>>> {
        self.instance.multi_get(self.cf_name, keys)
    }

    fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.instance.put(self.cf_name, key, value)
    }

    fn contains_key(&self, key: &[u8]) -> Result<bool> {
        self.instance.contains_key(self.cf_name, key)
    }

    fn may_contains_key(&self, key: &[u8]) -> Result<bool> {
        self.instance.may_contains_key(self.cf_name, key)
    }

    fn remove(&self, key: Vec<u8>) -> Result<()> {
        self.instance.remove(self.cf_name, key)
    }

    fn write_batch(&self, batch: WriteBatch) -> Result<()> {
        self.instance.write_batch(self.cf_name, batch)
    }

    fn get_len(&self) -> Result<u64> {
        self.instance.get_len()
    }

    fn keys(&self) -> Result<Vec<Vec<u8>>> {
        self.instance.keys()
    }

    fn put_sync(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.instance.put_sync(self.cf_name, key, value)
    }

    fn write_batch_sync(&self, batch: WriteBatch) -> Result<()> {
        self.instance.write_batch_sync(self.cf_name, batch)
    }
}

pub trait SchemaStore: Sized + ColumnFamily {
    fn get_store(&self) -> &InnerStore<Self>;
}

#[derive(Debug, Clone)]
pub enum WriteOp<V> {
    Value(V),
    Deletion,
}

impl<V> WriteOp<V>
where
    V: Serialize + DeserializeOwned,
{
    pub fn into_raw_op(self) -> Result<WriteOp<Vec<u8>>> {
        Ok(match self {
            WriteOp::Value(v) => WriteOp::Value(to_bytes(&v)?),
            WriteOp::Deletion => WriteOp::Deletion,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CodecWriteBatch<K, V>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
{
    rows: Vec<(K, WriteOp<V>)>,
}

impl<K, V> Default for CodecWriteBatch<K, V>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
{
    fn default() -> Self {
        Self { rows: Vec::new() }
    }
}

impl<K, V> CodecWriteBatch<K, V>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
{
    /// Creates an empty batch.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_puts(kvs: Vec<(K, V)>) -> Self {
        let mut rows = Vec::with_capacity(kvs.len());
        rows.extend(kvs.into_iter().map(|(k, v)| (k, WriteOp::Value(v))));
        Self { rows }
    }

    pub fn new_deletes(ks: Vec<K>) -> Self {
        let mut rows = Vec::with_capacity(ks.len());
        rows.extend(ks.into_iter().map(|k| (k, WriteOp::Deletion)));
        Self { rows }
    }

    /// Adds an insert/update operation to the batch.
    pub fn put(&mut self, key: K, value: V) -> Result<()> {
        self.rows.push((key, WriteOp::Value(value)));
        Ok(())
    }

    /// Adds a delete operation to the batch.
    pub fn delete(&mut self, key: K) -> Result<()> {
        self.rows.push((key, WriteOp::Deletion));
        Ok(())
    }

    ///Clear all operation to the next batch.
    pub fn clear(&mut self) -> Result<()> {
        self.rows.clear();
        Ok(())
    }
}

impl<K, V> IntoIterator for CodecWriteBatch<K, V>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
{
    type Item = (K, WriteOp<V>);
    type IntoIter = std::vec::IntoIter<(K, WriteOp<V>)>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.into_iter()
    }
}

#[allow(clippy::upper_case_acronyms)]
pub trait CodecKVStore<K, V>: Send + Sync
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
{
    fn kv_get(&self, key: K) -> Result<Option<V>>;

    fn multiple_get(&self, keys: Vec<K>) -> Result<Vec<Option<V>>>;

    fn kv_put(&self, key: K, value: V) -> Result<()>;

    fn put_sync(&self, key: K, value: V) -> Result<()>;

    fn contains_key(&self, key: K) -> Result<bool>;

    fn may_contains_key(&self, key: K) -> Result<bool>;

    fn remove(&self, key: K) -> Result<()>;

    fn write_batch(&self, batch: CodecWriteBatch<K, V>) -> Result<()>;

    fn write_batch_sync(&self, batch: CodecWriteBatch<K, V>) -> Result<()>;

    fn write_batch_raw(&self, batch: WriteBatch) -> Result<()>;

    fn put_all(&self, kvs: Vec<(K, V)>) -> Result<()> {
        self.write_batch(CodecWriteBatch::new_puts(kvs))
    }

    fn delete_all(&self, ks: Vec<K>) -> Result<()> {
        self.write_batch(CodecWriteBatch::new_deletes(ks))
    }

    fn get_len(&self) -> Result<u64>;

    fn keys(&self) -> Result<Vec<K>>;

    fn put_raw(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;

    fn get_raw(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;

    fn iter(&self) -> Result<SchemaIterator<K, V>>;

    fn multiple_get_raw(&self, keys: Vec<K>) -> Result<Vec<Option<Vec<u8>>>>;
}

impl<K, V, S> CodecKVStore<K, V> for S
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
    S: SchemaStore,
    S: ColumnFamily<Key = K, Value = V>,
{
    fn kv_get(&self, key: K) -> Result<Option<V>> {
        match KVStore::get(self.get_store(), to_bytes(&key)?.as_slice())? {
            Some(value) => Ok(Some(from_bytes::<V>(value.as_slice())?)),
            None => Ok(None),
        }
    }

    fn multiple_get(&self, keys: Vec<K>) -> Result<Vec<Option<V>>> {
        let encoded_keys = keys
            .into_iter()
            .map(|key| to_bytes(&key))
            .collect::<Result<Vec<_>, _>>()?;
        let values = KVStore::multiple_get(self.get_store(), encoded_keys)?;
        values
            .into_iter()
            .map(|value| match value {
                Some(value) => Ok(Some(from_bytes::<V>(value.as_slice())?)),
                None => Ok(None),
            })
            .collect()
    }

    fn kv_put(&self, key: K, value: V) -> Result<()> {
        KVStore::put(self.get_store(), to_bytes(&key)?, to_bytes(&value)?)
    }

    fn put_sync(&self, key: K, value: V) -> Result<()> {
        KVStore::put_sync(self.get_store(), to_bytes(&key)?, to_bytes(&value)?)
    }

    fn contains_key(&self, key: K) -> Result<bool> {
        KVStore::contains_key(self.get_store(), to_bytes(&key)?.as_slice())
    }

    fn may_contains_key(&self, key: K) -> Result<bool> {
        KVStore::may_contains_key(self.get_store(), to_bytes(&key)?.as_slice())
    }

    fn remove(&self, key: K) -> Result<()> {
        KVStore::remove(self.get_store(), to_bytes(&key)?)
    }

    fn write_batch(&self, batch: CodecWriteBatch<K, V>) -> Result<()> {
        KVStore::write_batch(self.get_store(), batch.try_into()?)
    }

    fn write_batch_sync(&self, batch: CodecWriteBatch<K, V>) -> Result<()> {
        KVStore::write_batch_sync(self.get_store(), batch.try_into()?)
    }

    fn write_batch_raw(&self, batch: WriteBatch) -> Result<()> {
        KVStore::write_batch(self.get_store(), batch)
    }

    fn get_len(&self) -> Result<u64> {
        KVStore::get_len(self.get_store())
    }

    fn keys(&self) -> Result<Vec<K>> {
        let keys = KVStore::keys(self.get_store())?;
        keys.into_iter()
            .map(|key| from_bytes::<K>(key.as_slice()))
            .collect::<Result<Vec<_>, _>>()
    }

    fn put_raw(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        KVStore::put(self.get_store(), key, value)
    }

    fn get_raw(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        KVStore::get(self.get_store(), key)
    }

    fn iter(&self) -> Result<SchemaIterator<K, V>> {
        let db = self
            .get_store()
            .store()
            .db()
            .ok_or_else(|| format_err!("Only support scan on db store instance"))?;
        db.iter::<K, V>(self.get_store().cf_name)
    }

    fn multiple_get_raw(&self, keys: Vec<K>) -> Result<Vec<Option<Vec<u8>>>> {
        let encoded_keys = keys
            .into_iter()
            .map(|key| to_bytes(&key))
            .collect::<Result<Vec<_>, _>>()?;
        KVStore::multiple_get(self.get_store(), encoded_keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use moveos_config::store_config::RocksdbConfig;

    #[test]
    fn test_new_db() {
        let tmpdir = moveos_config::temp_dir();
        let _db = RocksDB::new(tmpdir.path(), vec![], RocksdbConfig::default()).unwrap();
    }
}
