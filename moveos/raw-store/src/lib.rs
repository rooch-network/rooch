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
use crate::rocks::batch::WriteBatch;
use crate::rocks::{RocksDB, SchemaIterator};
use crate::traits::{DBStore, KVStore};
use anyhow::{bail, format_err, Result};
use moveos_common::utils::{from_bytes, to_bytes};
use parking_lot::Mutex;
use rocksdb::{properties, AsColumnFamilyRef};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::convert::TryInto;
use std::ffi::CStr;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;

/// Type alias to improve readability.
pub type ColumnFamilyName = &'static str;

pub const CF_METRICS_REPORT_PERIOD_MILLIS: u64 = 5000;
pub const METRICS_ERROR: i64 = -1;

// TODO: remove this after Rust rocksdb has the TOTAL_BLOB_FILES_SIZE property built-in.
// From https://github.com/facebook/rocksdb/blob/bd80433c73691031ba7baa65c16c63a83aef201a/include/rocksdb/db.h#L1169
const ROCKSDB_PROPERTY_TOTAL_BLOB_FILES_SIZE: &CStr =
    unsafe { CStr::from_bytes_with_nul_unchecked("rocksdb.total-blob-file-size\0".as_bytes()) };

///Store instance type define
#[derive(Clone)]
#[allow(clippy::upper_case_acronyms)]
pub enum StoreInstance {
    DB {
        db: Arc<RocksDB>,
        db_metrics: Arc<DBMetrics>,
        // Send consumes self, but we only have a &self of the containing struct,
        // so we put the sender in an Option containing
        metrics_task_cancel_handle: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    },
}

unsafe impl Send for StoreInstance {}

impl StoreInstance {
    pub fn new_db_instance(db: RocksDB, db_metrics: Arc<DBMetrics>) -> Self {
        let db_arc = Arc::new(db);
        let db_clone = db_arc.clone();
        let db_metrics_clone = db_metrics.clone();
        let (sender, mut cancel_receiver) = tokio::sync::oneshot::channel();

        // Introducing tokio 1.x runtime dependency in the raw store layer,
        // which would cause upper-level unit test cases and framework tests to depend on tokio.
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(Duration::from_millis(CF_METRICS_REPORT_PERIOD_MILLIS));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let cfs = db_clone.cfs.clone();
                        for cf_name in cfs {
                            let db_clone_clone = db_clone.clone();
                            let db_metrics_clone_clone = db_metrics_clone.clone();
                            if let Err(e) = tokio::task::spawn_blocking(move || {
                                let _ = Self::report_rocksdb_metrics(&db_clone_clone, cf_name, &db_metrics_clone_clone);
                            }).await {
                                tracing::error!("Failed to report cf metrics with error: {}", e);
                            }
                        }
                    }
                    _ = &mut cancel_receiver => {
                        tracing::info!("Metrics task cancelled for store instance");
                        break;
                    }
                }
            }
            tracing::debug!("Returning to report cf metrics task for store instance");
        });

        Self::DB {
            db: db_arc,
            db_metrics,
            metrics_task_cancel_handle: Arc::new(Mutex::new(Some(sender))),
        }
    }

    pub fn cancel_metrics_task(&mut self) -> Result<()> {
        match self {
            StoreInstance::DB {
                db: _,
                db_metrics: _,
                metrics_task_cancel_handle,
            } => {
                // Send a cancellation signal
                let mut handle = metrics_task_cancel_handle.lock();
                if let Some(sender) = handle.take() {
                    let _r = sender.send(());
                }
            }
        };
        Ok(())
    }

    pub fn db(&self) -> Option<&RocksDB> {
        match self {
            StoreInstance::DB {
                db,
                db_metrics: _,
                metrics_task_cancel_handle: _,
            } => Some(db.as_ref()),
        }
    }

    pub fn db_metrics(&self) -> Option<&DBMetrics> {
        match self {
            StoreInstance::DB {
                db: _,
                db_metrics,
                metrics_task_cancel_handle: _,
            } => Some(db_metrics.as_ref()),
        }
    }

    pub fn db_mut(&mut self) -> Option<&mut RocksDB> {
        match self {
            StoreInstance::DB {
                db,
                db_metrics: _,
                metrics_task_cancel_handle: _,
            } => Arc::get_mut(db),
        }
    }

    pub fn db_metrics_mut(&mut self) -> Option<&mut DBMetrics> {
        match self {
            StoreInstance::DB {
                db: _,
                db_metrics,
                metrics_task_cancel_handle: _,
            } => Arc::get_mut(db_metrics),
        }
    }

    fn report_rocksdb_metrics(
        rocksdb: &Arc<RocksDB>,
        cf_name: &str,
        db_metrics: &Arc<DBMetrics>,
    ) -> Result<()> {
        let cf = rocksdb.get_cf_handle(cf_name);
        db_metrics
            .rocksdb_metrics
            .rocksdb_total_sst_files_size
            .with_label_values(&[cf_name])
            .set(
                Self::get_int_property(rocksdb, &cf, properties::TOTAL_SST_FILES_SIZE)
                    .unwrap_or(METRICS_ERROR),
            );
        db_metrics
            .rocksdb_metrics
            .rocksdb_total_blob_files_size
            .with_label_values(&[cf_name])
            .set(
                Self::get_int_property(rocksdb, &cf, ROCKSDB_PROPERTY_TOTAL_BLOB_FILES_SIZE)
                    .unwrap_or(METRICS_ERROR),
            );
        db_metrics
            .rocksdb_metrics
            .rocksdb_size_all_mem_tables
            .with_label_values(&[cf_name])
            .set(
                Self::get_int_property(rocksdb, &cf, properties::SIZE_ALL_MEM_TABLES)
                    .unwrap_or(METRICS_ERROR),
            );
        db_metrics
            .rocksdb_metrics
            .rocksdb_block_cache_capacity
            .with_label_values(&[cf_name])
            .set(
                Self::get_int_property(rocksdb, &cf, properties::BLOCK_CACHE_CAPACITY)
                    .unwrap_or(METRICS_ERROR),
            );
        db_metrics
            .rocksdb_metrics
            .rocksdb_block_cache_usage
            .with_label_values(&[cf_name])
            .set(
                Self::get_int_property(rocksdb, &cf, properties::BLOCK_CACHE_USAGE)
                    .unwrap_or(METRICS_ERROR),
            );
        // db_metrics
        //     .rocksdb_metrics
        //     .rocksdb_block_cache_hit
        //     .with_label_values(&[cf_name])
        //     .set(
        //         Self::get_int_property(rocksdb, &cf, properties::BLOCK_CACHE_HIT_COUNT)
        //             .unwrap_or(METRICS_ERROR),
        //     );
        // db_metrics
        //     .rocksdb_metrics
        //     .rocksdb_block_cache_miss
        //     .with_label_values(&[cf_name])
        //     .set(
        //         Self::get_int_property(rocksdb, &cf, properties::BLOCK_CACHE_MISS_COUNT)
        //             .unwrap_or(METRICS_ERROR),
        //     );
        db_metrics
            .rocksdb_metrics
            .rocksdb_mem_table_flush_pending
            .with_label_values(&[cf_name])
            .set(
                Self::get_int_property(rocksdb, &cf, properties::MEM_TABLE_FLUSH_PENDING)
                    .unwrap_or(METRICS_ERROR),
            );
        db_metrics
            .rocksdb_metrics
            .rocskdb_compaction_pending
            .with_label_values(&[cf_name])
            .set(
                Self::get_int_property(rocksdb, &cf, properties::COMPACTION_PENDING)
                    .unwrap_or(METRICS_ERROR),
            );
        db_metrics
            .rocksdb_metrics
            .rocskdb_num_running_compactions
            .with_label_values(&[cf_name])
            .set(
                Self::get_int_property(rocksdb, &cf, properties::NUM_RUNNING_COMPACTIONS)
                    .unwrap_or(METRICS_ERROR),
            );
        db_metrics
            .rocksdb_metrics
            .rocksdb_num_running_flushes
            .with_label_values(&[cf_name])
            .set(
                Self::get_int_property(rocksdb, &cf, properties::NUM_RUNNING_FLUSHES)
                    .unwrap_or(METRICS_ERROR),
            );
        db_metrics
            .rocksdb_metrics
            .rocskdb_background_errors
            .with_label_values(&[cf_name])
            .set(
                Self::get_int_property(rocksdb, &cf, properties::BACKGROUND_ERRORS)
                    .unwrap_or(METRICS_ERROR),
            );
        Ok(())
    }

    #[allow(dead_code)]
    fn get_int_property(
        rocksdb: &RocksDB,
        cf: &impl AsColumnFamilyRef,
        property_name: &'static std::ffi::CStr,
    ) -> Result<i64, anyhow::Error> {
        match rocksdb.property_int_value_cf(cf, property_name) {
            Ok(Some(value)) => Ok(value as i64),
            Ok(None) => Ok(0),
            Err(e) => Err(anyhow::Error::new(e)),
            // Err(anyhow::Error::msg(format!("get_int_property error {}", e))),
        }
    }
}

impl DBStore for StoreInstance {
    fn get(&self, cf_name: &str, key: &[u8]) -> Result<Option<Vec<u8>>> {
        match self {
            StoreInstance::DB {
                db,
                db_metrics,
                metrics_task_cancel_handle: _,
            } => {
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
            StoreInstance::DB {
                db,
                db_metrics,
                metrics_task_cancel_handle: _,
            } => {
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
            StoreInstance::DB {
                db,
                db_metrics,
                metrics_task_cancel_handle: _,
            } => {
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

    fn remove(&self, cf_name: &str, key: Vec<u8>) -> Result<()> {
        match self {
            StoreInstance::DB {
                db,
                db_metrics,
                metrics_task_cancel_handle: _,
            } => {
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
            StoreInstance::DB {
                db,
                db_metrics,
                metrics_task_cancel_handle: _,
            } => {
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
            StoreInstance::DB {
                db,
                db_metrics,
                metrics_task_cancel_handle: _,
            } => {
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
            StoreInstance::DB {
                db,
                db_metrics,
                metrics_task_cancel_handle: _,
            } => {
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

    fn multi_get(&self, cf_name: &str, keys: Vec<Vec<u8>>) -> Result<Vec<Option<Vec<u8>>>> {
        match self {
            StoreInstance::DB {
                db,
                db_metrics,
                metrics_task_cancel_handle: _,
            } => {
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
        let mut rows = Vec::new();
        rows.extend(kvs.into_iter().map(|(k, v)| (k, WriteOp::Value(v))));
        Self { rows }
    }

    pub fn new_deletes(ks: Vec<K>) -> Self {
        let mut rows = Vec::new();
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
