// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::store_config::R_DEFAULT_DB_DIR;
use crate::{BaseConfig, ConfigModule, RoochOpt};
use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

pub const ROOCH_INDEXER_DB_FILENAME: &str = "indexer.sqlite";

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, Parser)]
#[clap(name = "Rooch indexer")]
pub struct IndexerConfig {
    // #[clap(skip)]
    // pub db_url: Option<String>,
    #[serde(skip)]
    #[clap(skip)]
    base: Option<Arc<BaseConfig>>,
}

impl IndexerConfig {
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
        let indexer_db = self.clone().get_indexer_db();
        let indexer_db_parent_dir = indexer_db
            .parent()
            .ok_or(anyhow::anyhow!("Invalid indexer db dir"))?;
        if !indexer_db_parent_dir.exists() {
            std::fs::create_dir_all(indexer_db_parent_dir)?;
        }
        if !indexer_db.exists() {
            std::fs::File::create(indexer_db.clone())?;
        }
        println!("IndexerConfig init store dir {:?}", indexer_db);
        Ok(())
    }

    fn base(&self) -> &BaseConfig {
        self.base.as_ref().expect("Config should init.")
    }

    pub fn data_dir(&self) -> &Path {
        self.base().data_dir()
    }

    pub fn get_indexer_db(&self) -> PathBuf {
        self.data_dir()
            .join(R_DEFAULT_DB_DIR.as_path())
            .join(ROOCH_INDEXER_DB_FILENAME)
    }
}

impl ConfigModule for IndexerConfig {
    fn merge_with_opt(&mut self, _opt: &RoochOpt, base: Arc<BaseConfig>) -> Result<()> {
        self.base = Some(base);

        Ok(())
    }
}

impl std::fmt::Display for IndexerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).map_err(|_e| std::fmt::Error)?
        )
    }
}

impl FromStr for IndexerConfig {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let deserialized: IndexerConfig = serde_json::from_str(s)?;
        Ok(deserialized)
    }
}
