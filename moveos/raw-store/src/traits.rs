// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::rocks::batch::{WriteBatch, WriteBatchCF};
use anyhow::Result;

#[allow(clippy::upper_case_acronyms)]
pub trait KVStore: Send + Sync {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn multiple_get(&self, keys: Vec<Vec<u8>>) -> Result<Vec<Option<Vec<u8>>>>;
    fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    fn contains_key(&self, key: &[u8]) -> Result<bool>;
    fn remove(&self, key: Vec<u8>) -> Result<()>;
    fn write_batch(&self, batch: WriteBatch) -> Result<()>;
    fn get_len(&self) -> Result<u64>;
    fn keys(&self) -> Result<Vec<Vec<u8>>>;
    fn put_sync(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    fn write_batch_sync(&self, batch: WriteBatch) -> Result<()>;
}

pub trait DBStore: Send + Sync {
    fn get(&self, cf_name: &str, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn put(&self, cf_name: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    fn contains_key(&self, cf_name: &str, key: &[u8]) -> Result<bool>;
    fn remove(&self, cf_name: &str, key: Vec<u8>) -> Result<()>;
    fn write_batch(&self, cf_name: &str, batch: WriteBatch) -> Result<()>;
    fn get_len(&self) -> Result<u64>;
    fn keys(&self) -> Result<Vec<Vec<u8>>>;
    fn put_sync(&self, cf_name: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    fn write_batch_sync(&self, cf_name: &str, batch: WriteBatch) -> Result<()>;
    // write batch across column families with sync-write, assert_eq!(cf_names.len(), batch.rows.len())
    fn write_batch_across_cfs(
        &self,
        cf_names: Vec<&str>,
        batch: WriteBatch,
        sync: bool,
    ) -> Result<()>;
    // write batch across column families, each batch is for one column family for avoiding redundant column family name passing(we may have big batch)
    fn write_cf_batch(&self, cf_batches: Vec<WriteBatchCF>, sync: bool) -> Result<()>;
    fn multi_get(&self, cf_name: &str, keys: Vec<Vec<u8>>) -> Result<Vec<Option<Vec<u8>>>>;
}
