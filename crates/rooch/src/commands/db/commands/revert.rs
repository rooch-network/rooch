// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use rooch_config::R_OPT_NET_HELP;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use std::path::PathBuf;

use crate::cli_types::WalletContextOptions;
use crate::commands::db::commands::init;

/// Revert tx by db command.
#[derive(Debug, Parser)]
pub struct RevertCommand {
    #[clap(long, short = 'o')]
    /// tx order which expect to revert, it must be the last tx order
    pub tx_order: Option<u64>,

    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    /// If local chainid, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

impl RevertCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let (_root, rooch_db, _start_time) = init(self.base_data_dir, self.chain_id);

        rooch_db.revert_tx(self.tx_order)?;

        Ok(())
    }
}
