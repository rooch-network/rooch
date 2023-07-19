// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::borrow::Borrow;
// use std::error::Error;
// pub use crate::batch::WriteBatch;
// use crate::cache_store::CacheStore;
// use crate::rocks::{RocksDB, SchemaIterator};
// use crate::upgrade::DBUpgrade;
use anyhow::{bail, format_err, Result};
use byteorder::{BigEndian, ReadBytesExt};
// use starcoin_config::NodeConfig;
// use starcoin_crypto::HashValue;
// use starcoin_logger::prelude::info;
// use starcoin_vm_types::state_store::table::TableHandle;
use crate::rocks::batch::WriteBatch;
use std::fmt::Debug;
// use serde::de::DeserializeOwned;
// use serde::Serialize;

/// Type alias to improve readability.
pub type ColumnFamilyName = &'static str;

#[allow(clippy::upper_case_acronyms)]
pub trait KVStore: Send + Sync {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn multiple_get(&self, keys: Vec<Vec<u8>>) -> Result<Vec<Option<Vec<u8>>>>;
    fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    fn contains_key(&self, key: Vec<u8>) -> Result<bool>;
    fn remove(&self, key: Vec<u8>) -> Result<()>;
    fn write_batch(&self, batch: WriteBatch) -> Result<()>;
    fn get_len(&self) -> Result<u64>;
    fn keys(&self) -> Result<Vec<Vec<u8>>>;
    fn put_sync(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    fn write_batch_sync(&self, batch: WriteBatch) -> Result<()>;
}

pub trait DBStore: Send + Sync {
    fn get(&self, prefix_name: &str, key: Vec<u8>) -> Result<Option<Vec<u8>>>;
    fn put(&self, prefix_name: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    fn contains_key(&self, prefix_name: &str, key: Vec<u8>) -> Result<bool>;
    fn remove(&self, prefix_name: &str, key: Vec<u8>) -> Result<()>;
    fn write_batch(&self, prefix_name: &str, batch: WriteBatch) -> Result<()>;
    fn get_len(&self) -> Result<u64>;
    fn keys(&self) -> Result<Vec<Vec<u8>>>;
    fn put_sync(&self, prefix_name: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    fn write_batch_sync(&self, prefix_name: &str, batch: WriteBatch) -> Result<()>;
    fn multi_get(&self, prefix_name: &str, keys: Vec<Vec<u8>>) -> Result<Vec<Option<Vec<u8>>>>;
}
