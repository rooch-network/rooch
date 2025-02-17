// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::db::commands::init;
use clap::Parser;
use rooch_config::R_OPT_NET_HELP;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use std::path::PathBuf;

/// Repair the database offline.
/// Help to reach consistency of the database.
#[derive(Debug, Parser)]
pub struct RepairCommand {
    #[clap(
        long,
        help = "perform a thorough and detailed check, which may take more time"
    )]
    pub thorough: bool,
    #[clap(
        long = "exec",
        help = "execute repair, otherwise only report issues. default is false"
    )]
    pub exec: bool,

    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,
}

impl RepairCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let (_root, rooch_db, _start_time) = init(self.base_data_dir, self.chain_id);

        let (issues, fixed) = rooch_db.repair(self.thorough, self.exec)?;

        println!("issues found: {}, fixed: {}", issues, fixed);

        Ok(())
    }
}
