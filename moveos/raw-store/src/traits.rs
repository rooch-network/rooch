// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::rocks::batch::WriteBatch;
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
    fn write_batch_sync_across_cfs(&self, cf_names: Vec<&str>, batch: WriteBatch) -> Result<()>;
    fn multi_get(&self, cf_name: &str, keys: Vec<Vec<u8>>) -> Result<Vec<Option<Vec<u8>>>>;
}
