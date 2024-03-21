// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use rooch_config::da_config::DAConfig;
use rooch_config::indexer_config::IndexerConfig;
use rooch_config::store_config::StoreConfig;
use rooch_config::{BaseConfig, RoochOpt};
use rooch_types::error::{RoochError, RoochResult};
use std::sync::Arc;
use std::{fs, path::Path};

/// Clean the Rooch server storage
#[derive(Debug, Parser)]
pub struct CleanCommand {
    #[clap(flatten)]
    opt: RoochOpt,
}

#[allow(dead_code)]
impl CleanCommand {
    pub fn execute(self) -> RoochResult<()> {
        let base_config = BaseConfig::load_with_opt(&self.opt)?;
        let arc_base_config = Arc::new(base_config);
        let mut store_config = StoreConfig::default();
        store_config.merge_with_opt_with_init(&self.opt, Arc::clone(&arc_base_config), false)?;
        let mut indexer_config = IndexerConfig::default();
        indexer_config.merge_with_opt_with_init(&self.opt, Arc::clone(&arc_base_config), false)?;
        let mut da_config = DAConfig::default();
        da_config.merge_with_opt_with_init(&self.opt, Arc::clone(&arc_base_config), false)?;

        let rooch_store_dir = store_config.get_rooch_store_dir();
        let moveos_store_dir = store_config.get_moveos_store_dir();
        let indexer_store_file = indexer_config.get_indexer_db();
        let openda_fs_dir = da_config.get_openda_fs_dir();

        self.remove_store_dir(&rooch_store_dir, "Rooch")?;
        self.remove_store_dir(&moveos_store_dir, "MoveOS")?;
        self.remove_store_dir(&indexer_store_file, "Indexer")?;
        self.remove_store_dir(&openda_fs_dir, "OpenDA")?;

        println!("Rooch server storage successfully cleaned");

        Ok(())
    }

    fn remove_store_dir(&self, store_dir: &Path, name: &str) -> RoochResult<()> {
        if !store_dir.exists() {
            return Ok(());
        }

        if !store_dir.is_dir() {
            return Err(RoochError::CleanServerError(format!(
                "{} database path is not a valid directory",
                name
            )));
        }

        fs::remove_dir_all(store_dir).map_err(|e| RoochError::CleanServerError(e.to_string()))
    }

    fn remove_store_file(&self, store_file: &Path, name: &str) -> RoochResult<()> {
        if !store_file.exists() {
            return Ok(());
        }

        if !store_file.is_file() {
            return Err(RoochError::CleanServerError(format!(
                "{} database path is not a valid file",
                name
            )));
        }

        fs::remove_file(store_file).map_err(|e| RoochError::CleanServerError(e.to_string()))
    }
}
