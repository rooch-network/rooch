// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::get_rooch_config_dir;
use anyhow::Result;
use clap::Parser;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use moveos_config::store_config::RocksdbConfig;


static R_DEFAULT_DB_DIR: Lazy<PathBuf> = Lazy::new(|| PathBuf::from("roochdb"));
pub const DEFAULT_CACHE_SIZE: usize = 20000;

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
    #[clap(name = "cache-sizes", long, help = "cache sizes")]
    pub cache_size: Option<usize>,

    // #[serde(skip)]
    // #[clap(skip)]
    // base: Option<Arc<BaseConfig>>,
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

    pub db_path: PathBuf,
    pub rooch_store_path: PathBuf,
}

impl StoreConfig {
    pub fn init(&self) -> Result<()> {
        let db_dir = get_rooch_config_dir()
            .unwrap()
            .join(R_DEFAULT_DB_DIR.as_path());
        if !db_dir.exists() {
            fs::create_dir_all(db_dir.clone())?;
        }
        println!("StoreConfig init store dir {:?}", db_dir);
        Ok(())
    }

    pub fn get_store_dir(&self) -> PathBuf {
        get_rooch_config_dir()
            .unwrap()
            .join(R_DEFAULT_DB_DIR.as_path())
    }

    pub fn rocksdb_config(&self) -> RocksdbConfig {
        let default = RocksdbConfig::default();
        RocksdbConfig {
            max_open_files: self.max_open_files.unwrap_or(default.max_open_files),
            max_total_wal_size: self
                .max_total_wal_size
                .unwrap_or(default.max_total_wal_size),
            bytes_per_sync: self.bytes_per_sync.unwrap_or(default.bytes_per_sync),
            wal_bytes_per_sync: self
                .wal_bytes_per_sync
                .unwrap_or(default.wal_bytes_per_sync),
        }
    }
    pub fn cache_size(&self) -> usize {
        self.cache_size.unwrap_or(DEFAULT_CACHE_SIZE)
    }
}