// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::utils::open_rooch_db_readonly;
use clap::Parser;
use raw_store::SchemaStore;
use rocksdb::checkpoint::Checkpoint;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use std::path::PathBuf;

/// generate RocksDB's checkpoint to directory
#[derive(Debug, Parser)]
pub struct GenerateDBCheckPointCommand {
    #[clap(long, short = 'o')]
    output_dir: PathBuf,

    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,

    #[clap(long, short = 'n')]
    pub chain_id: Option<RoochChainID>,
}

impl GenerateDBCheckPointCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let (_root, rooch_db, _start_time) =
            open_rooch_db_readonly(self.base_data_dir.clone(), self.chain_id.clone());
        let rocks_db = rooch_db
            .moveos_store
            .node_store
            .get_store()
            .store()
            .db()
            .expect("Failed to open RocksDB instance")
            .inner();

        let check_point = Checkpoint::new(rocks_db)
            .expect("failed to create Checkpoint object from RocksDB instance.");
        check_point
            .create_checkpoint(self.output_dir.as_path())
            .map_err(|e| {
                rooch_types::error::RoochError::from(anyhow::anyhow!(
                    "failed to create checkpoint directory at {:?}: {}",
                    self.output_dir,
                    e
                ))
            })?;
        println!("create checkpoint succeeded.");

        Ok(())
    }
}
