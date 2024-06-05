// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::BaseConfig;
use anyhow::Result;
use clap::Parser;
use moveos_config::store_config::RocksdbConfig;
use moveos_config::DataDirPath;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

pub const DEFAULT_DB_DIR: &str = "roochdb";
pub const DEFAULT_DB_ROOCH_SUBDIR: &str = "rooch_store";
pub const DEFAULT_DB_MOVEOS_SUBDIR: &str = "moveos_store";
pub const DEFAULT_DB_INDEXER_SUBDIR: &str = "indexer";

// for Rooch DB instance, doesn't need too much row cache:
// store ledger tx and several meta. Most of the time, they are always requested for newer data
pub const DEFAULT_ROCKSDB_ROW_CACHE_SIZE: u64 = 1 << 24; // 16MB,
pub const DEFAULT_ROCKSDB_BLOCK_CACHE_SIZE: u64 = 1 << 26; // 64MB

#[derive(Clone, Default, Debug, Deserialize, PartialEq, Serialize, Parser)]
#[serde(deny_unknown_fields)]
pub struct StoreConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "rocksdb-max-open-files", long, help = "rocksdb max open files")]
    pub max_open_files: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(
        name = "rocksdb-max-total-wal-sizes",
        long,
        help = "rocksdb max total WAL sizes"
    )]
    pub max_total_wal_size: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(
        name = "rocksdb-wal-bytes-per-sync",
        long,
        help = "rocksdb wal bytes per sync"
    )]
    pub wal_bytes_per_sync: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "rocksdb-bytes-per-sync", long, help = "rocksdb bytes per sync")]
    pub bytes_per_sync: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(
        name = "rocksdb-max-background-jobs",
        long,
        help = "rocksdb max background jobs"
    )]
    pub max_background_jobs: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "rocksdb-row-cache-size", long, help = "rocksdb row cache size")]
    pub row_cache_size: Option<u64>, // get: memtable -> row cache -> block cache -> disk

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(
        name = "rocksdb-block-cache-size",
        long,
        help = "rocksdb block cache size"
    )]
    pub block_cache_size: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "rocksdb-block-size", long, help = "rocksdb block size")]
    pub block_size: Option<u64>,
    // TODO filter cache configs
    // #[clap(name="rocksdb-cache-index-and-filter-blocks", long, help="rocksdb cache index and filter blocks")]

    // Once the limit is reached, RocksDB will not create more memtables and will stall all new write operations until the memtables count is reduced below the max-write-buffer-number limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(
        name = "rocksdb-max-write-buffer-number",
        long,
        help = "rocksdb max write buffer number, maximum number of memtables that you can create"
    )]
    pub max_write_buffer_number: Option<u64>,

    #[serde(skip)]
    #[clap(skip)]
    base: Option<Arc<BaseConfig>>,
}

impl StoreConfig {
    pub(crate) fn init(&mut self, base: Arc<BaseConfig>) -> Result<()> {
        self.base = Some(base);
        let rooch_store_dir = self.get_rooch_store_dir();
        let moveos_store_dir = self.get_moveos_store_dir();
        let indexer_store_dir = self.get_indexer_store_dir();
        if !rooch_store_dir.exists() {
            std::fs::create_dir_all(rooch_store_dir.clone())?;
        }
        if !moveos_store_dir.exists() {
            std::fs::create_dir_all(moveos_store_dir.clone())?;
        }
        if !indexer_store_dir.exists() {
            std::fs::create_dir_all(indexer_store_dir.clone())?;
        }
        Ok(())
    }

    fn base(&self) -> &BaseConfig {
        self.base.as_ref().expect("Config should init.")
    }

    pub fn data_dir(&self) -> &Path {
        self.base().data_dir()
    }

    pub fn get_rooch_db_dir(&self) -> PathBuf {
        self.data_dir().join(DEFAULT_DB_DIR)
    }

    pub fn get_moveos_store_dir(&self) -> PathBuf {
        self.get_rooch_db_dir().join(DEFAULT_DB_MOVEOS_SUBDIR)
    }

    pub fn get_rooch_store_dir(&self) -> PathBuf {
        self.get_rooch_db_dir().join(DEFAULT_DB_ROOCH_SUBDIR)
    }

    pub fn get_indexer_store_dir(&self) -> PathBuf {
        self.get_rooch_db_dir().join(DEFAULT_DB_INDEXER_SUBDIR)
    }

    pub fn rocksdb_config(&self, is_moveos_db: bool) -> RocksdbConfig {
        let default = RocksdbConfig::default();
        let mut block_cache_size = default.block_cache_size;
        let mut row_cache_size = default.row_cache_size;
        if !is_moveos_db {
            block_cache_size = DEFAULT_ROCKSDB_BLOCK_CACHE_SIZE;
            row_cache_size = DEFAULT_ROCKSDB_ROW_CACHE_SIZE;
        }

        RocksdbConfig {
            max_open_files: self.max_open_files.unwrap_or(default.max_open_files),
            max_total_wal_size: self
                .max_total_wal_size
                .unwrap_or(default.max_total_wal_size),
            bytes_per_sync: self.bytes_per_sync.unwrap_or(default.bytes_per_sync),
            max_background_jobs: self
                .max_background_jobs
                .unwrap_or(default.max_background_jobs),
            wal_bytes_per_sync: self
                .wal_bytes_per_sync
                .unwrap_or(default.wal_bytes_per_sync),
            row_cache_size: self.row_cache_size.unwrap_or(row_cache_size),
            max_write_buffer_numer: self
                .max_write_buffer_number
                .unwrap_or(default.max_write_buffer_numer),
            block_cache_size: self.block_cache_size.unwrap_or(block_cache_size),
            block_size: self.block_size.unwrap_or(default.block_size),
        }
    }

    pub fn get_mock_moveos_store_dir(data_dir: &DataDirPath) -> PathBuf {
        data_dir
            .path()
            .join(DEFAULT_DB_DIR)
            .join(DEFAULT_DB_MOVEOS_SUBDIR)
    }

    pub fn get_mock_rooch_store_dir(data_dir: &DataDirPath) -> PathBuf {
        data_dir
            .path()
            .join(DEFAULT_DB_DIR)
            .join(DEFAULT_DB_ROOCH_SUBDIR)
    }
}

impl std::fmt::Display for StoreConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).map_err(|_e| std::fmt::Error)?
        )
    }
}

impl FromStr for StoreConfig {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let deserialized: StoreConfig = serde_json::from_str(s)?;
        Ok(deserialized)
    }
}
