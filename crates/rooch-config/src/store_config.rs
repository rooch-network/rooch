// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BaseConfig, ConfigModule, RoochOpt};
use anyhow::Result;
use clap::Parser;
use moveos_config::store_config::RocksdbConfig;
use moveos_config::DataDirPath;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

pub static R_DEFAULT_DB_DIR: Lazy<PathBuf> = Lazy::new(|| PathBuf::from("roochdb"));
static R_DEFAULT_DB_MOVEOS_SUBDIR: Lazy<PathBuf> = Lazy::new(|| PathBuf::from("moveos_store"));
static R_DEFAULT_DB_ROOCH_SUBDIR: Lazy<PathBuf> = Lazy::new(|| PathBuf::from("rooch_store"));

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

    // TODO row cache is expensive (not so memory efficient), we need block cache at the same time.
    // #[clap(name="rocksdb-block-cache-size", long, help="rocksdb block cache size")]
    // pub block_cache_size:u64,
    // TODO share large block cache between column families, dozens GB is required. block_size is 16KB by default for balancing bulk scan and read amplification
    // #[clap(name="rocksdb-block-size", long, help="rocksdb block size")]
    // pub block_size:u64,
    // #[clap(name="rocksdb-cache-index-and-filter-blocks", long, help="rocksdb cache index and filter blocks")]
    // TODO filter cache configs

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
    pub fn merge_with_opt_with_init(
        &mut self,
        opt: &RoochOpt,
        base: Arc<BaseConfig>,
        with_init: bool,
    ) -> Result<()> {
        self.merge_with_opt(opt, base)?;
        if with_init {
            self.init()?;
        }
        Ok(())
    }

    pub fn init(&self) -> Result<()> {
        let rooch_db_dir = self.get_rooch_store_dir();
        let moveos_db_dir = self.get_moveos_store_dir();
        if !rooch_db_dir.exists() {
            std::fs::create_dir_all(rooch_db_dir.clone())?;
        }
        if !moveos_db_dir.exists() {
            std::fs::create_dir_all(moveos_db_dir.clone())?;
        }
        println!(
            "StoreConfig init store dir {:?} {:?}",
            rooch_db_dir, moveos_db_dir
        );
        Ok(())
    }

    fn base(&self) -> &BaseConfig {
        self.base.as_ref().expect("Config should init.")
    }

    pub fn data_dir(&self) -> &Path {
        self.base().data_dir()
    }

    pub fn get_moveos_store_dir(&self) -> PathBuf {
        self.data_dir()
            .join(R_DEFAULT_DB_DIR.as_path())
            .join(R_DEFAULT_DB_MOVEOS_SUBDIR.as_path())
    }

    pub fn get_rooch_store_dir(&self) -> PathBuf {
        self.data_dir()
            .join(R_DEFAULT_DB_DIR.as_path())
            .join(R_DEFAULT_DB_ROOCH_SUBDIR.as_path())
    }

    pub fn rocksdb_config(&self) -> RocksdbConfig {
        let default = RocksdbConfig::default();
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
            row_cache_size: self.row_cache_size.unwrap_or(default.row_cache_size),
            max_write_buffer_numer: self
                .max_write_buffer_number
                .unwrap_or(default.max_write_buffer_numer),
        }
    }

    pub fn get_mock_moveos_store_dir(data_dir: &DataDirPath) -> PathBuf {
        data_dir
            .path()
            .join(R_DEFAULT_DB_DIR.as_path())
            .join(R_DEFAULT_DB_MOVEOS_SUBDIR.as_path())
    }

    pub fn get_mock_rooch_store_dir(data_dir: &DataDirPath) -> PathBuf {
        data_dir
            .path()
            .join(R_DEFAULT_DB_DIR.as_path())
            .join(R_DEFAULT_DB_ROOCH_SUBDIR.as_path())
    }
}

impl ConfigModule for StoreConfig {
    fn merge_with_opt(&mut self, opt: &RoochOpt, base: Arc<BaseConfig>) -> Result<()> {
        self.base = Some(base);

        let store_config = opt.store.clone();
        if store_config.max_open_files.is_some() {
            self.max_open_files = store_config.max_open_files;
        }
        if store_config.max_total_wal_size.is_some() {
            self.max_total_wal_size = store_config.max_total_wal_size;
        }
        if store_config.row_cache_size.is_some() {
            self.row_cache_size = store_config.row_cache_size;
        }
        if store_config.bytes_per_sync.is_some() {
            self.bytes_per_sync = store_config.bytes_per_sync;
        }
        if store_config.wal_bytes_per_sync.is_some() {
            self.wal_bytes_per_sync = store_config.wal_bytes_per_sync;
        }

        Ok(())
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
