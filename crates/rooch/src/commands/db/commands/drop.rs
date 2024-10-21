// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::db::commands::open_rocks;
use clap::Parser;
use rooch_config::R_OPT_NET_HELP;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use std::path::PathBuf;

/// drop column family by column family name
#[derive(Debug, Parser)]
pub struct DropCommand {
    #[clap(long = "cf-name")]
    /// column family name
    pub cf_name: String,

    #[clap(
        long = "re-create",
        help = "re-create column family after drop to clear column family, default is false"
    )]
    /// re-create column family after drop, default is false
    pub re_create: Option<bool>,

    #[clap(
        long = "force",
        help = "force to execute the command: drop column family is a dangerous operation, make sure you know what you are doing. default is false"
    )]
    pub force: Option<bool>,

    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    /// If local chainid, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,
}

impl DropCommand {
    pub async fn execute(self) -> RoochResult<()> {
        if !self.force.unwrap_or(false) {
            println!("This operation is dangerous, make sure you know what you are doing. If you are sure, please add --force to execute this command.");
            return Ok(());
        }
        let mut rocks = open_rocks(self.base_data_dir.clone(), self.chain_id)?;
        let re_create = self.re_create.unwrap_or(false);
        let cf_name = self.cf_name.clone();
        let mut op = "drop";
        if re_create {
            op = "clear";
            rocks.clear_cfs(vec![&cf_name])?;
        } else {
            rocks.drop_cf(&cf_name)?;
        }
        println!("{} column family {} success", op, cf_name);

        Ok(())
    }
}
