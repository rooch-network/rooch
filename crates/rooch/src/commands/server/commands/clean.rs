// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use rooch_config::store_config::StoreConfig;
use rooch_config::R_OPT_NET_HELP;
use rooch_config::{BaseConfig, RoochOpt};
use rooch_types::chain_id::RoochChainID;
use rooch_types::error::{RoochError, RoochResult};
use std::sync::Arc;
use std::{fs, path::Path};

/// Clean the Rooch server storage
#[derive(Debug, Parser)]
pub struct CleanCommand {
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,
}

impl CleanCommand {
    pub fn execute(self) -> RoochResult<()> {
        let opt = RoochOpt {
            base_data_dir: None,
            chain_id: self.chain_id.clone(),
            store: None,
        };
        let base_config = BaseConfig::load_with_opt(&opt)?;
        let mut store_config = StoreConfig::default();
        store_config.merge_with_opt_then_init(&opt, Arc::new(base_config))?;

        let rooch_store_dir = store_config.get_rooch_store_dir();
        let moveos_store_dir = store_config.get_moveos_store_dir();

        self.remove_store_dir(&rooch_store_dir, "Rooch")?;
        self.remove_store_dir(&moveos_store_dir, "MoveOS")?;

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
}
