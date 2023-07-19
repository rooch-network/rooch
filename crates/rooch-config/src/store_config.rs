// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{get_rooch_config_dir, Config};
use std::fs;
// use anyhow::Result;
use clap::Parser;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Parser)]
#[serde(default, deny_unknown_fields)]
pub struct RocksdbConfig {}

impl RocksdbConfig {}

impl Default for RocksdbConfig {
    fn default() -> Self {
        Self {}
    }
}

static R_DEFAULT_DB_DIR: Lazy<PathBuf> = Lazy::new(|| PathBuf::from("roochdb"));

#[derive(Clone, Default, Debug, Deserialize, PartialEq, Serialize, Parser)]
#[serde(deny_unknown_fields)]
pub struct StoreConfig {
    pub db_path: PathBuf,
    // pub db_path: String,
}

impl Default for StoreConfig {
    fn default() -> Self {
        StoreConfig {
            db_path: R_DEFAULT_DB_DIR,
        }
    }
}

impl StoreConfig {
    pub fn init(&self) {
        let db_dir = get_rooch_config_dir()?.join(R_DEFAULT_DB_DIR.as_path());
        if !db_dir.exists() {
            fs::create_dir_all(db_dir.clone())?
        }
        println!("StoreConfig init store dir {:?}", db_dir);
    }

    pub fn get_store_dir(&self) -> PathBuf {
        get_rooch_config_dir()?.join(R_DEFAULT_DB_DIR.as_path())
    }

    pub fn rocksdb_config(&self) -> RocksdbConfig {
        let default = RocksdbConfig::default();
        RocksdbConfig {}
    }
}

impl Config for StoreConfig {}
