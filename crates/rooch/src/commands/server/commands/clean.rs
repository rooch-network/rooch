// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use rooch_config::store_config::StoreConfig;
use rooch_types::error::{RoochError, RoochResult};
use std::{fs, path::Path};

/// Clean the Rooch server storage
#[derive(Debug, Parser)]
pub struct CleanCommand;

impl CleanCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let rooch_store = StoreConfig::get_rooch_store_dir();
        let moveos_store = StoreConfig::get_moveos_store_dir();

        self.remove_store_dir(&rooch_store, "Rooch")?;
        self.remove_store_dir(&moveos_store, "MoveOS")?;

        println!("Rooch server storage successfully cleaned");

        Ok(())
    }

    fn remove_store_dir(&self, store_path: &Path, name: &str) -> RoochResult<()> {
        if !store_path.exists() {
            return Ok(());
        }

        if !store_path.is_dir() {
            return Err(RoochError::CleanServerError(
                format!("{} database path is not a valid directory", name).to_owned(),
            ));
        }

        fs::remove_dir_all(store_path).map_err(|e| RoochError::CleanServerError(e.to_string()))
    }
}
