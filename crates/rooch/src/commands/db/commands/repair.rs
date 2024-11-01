// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::db::commands::init;
use clap::Parser;
use rooch_config::R_OPT_NET_HELP;
use rooch_types::rooch_network::RoochChainID;
use std::path::PathBuf;

/// drop column family by column family name
#[derive(Debug, Parser)]
pub struct RepairCommand {
    #[clap(
        long = "exec",
        help = "execute repair, otherwise only report issues. default is false"
    )]
    pub exec: bool,

    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    /// If local chainid, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,
}

impl RepairCommand {
    pub async fn execute(self) -> anyhow::Result<()> {
        let (_root, rooch_db, _start_time) = init(self.base_data_dir, self.chain_id);

        let issues = rooch_db.repair(self.exec)?;

        if issues.is_empty() {
            println!("No issues found");
        } else {
            println!("Issues found:");
            for issue in issues {
                println!("{}", issue);
            }
        }

        Ok(())
    }
}
