// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::db::commands::load_accumulator;
use crate::utils::open_rooch_db;
use accumulator::Accumulator;
use clap::Parser;
use rooch_config::R_OPT_NET_HELP;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use std::path::PathBuf;

/// Verify Order by Accumulator
#[derive(Debug, Parser)]
pub struct GetAccumulatorLeafByIndexCommand {
    #[clap(long)]
    pub index: u64,
    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,
}

impl GetAccumulatorLeafByIndexCommand {
    pub fn execute(self) -> RoochResult<()> {
        let (_root, rooch_db, _start_time) = open_rooch_db(self.base_data_dir, self.chain_id);
        let rooch_store = rooch_db.rooch_store;
        let (tx_accumulator, _last_tx_order_in_db) = load_accumulator(rooch_store.clone())?;

        let leaf = tx_accumulator.get_leaf(self.index)?;

        println!("{:?}", leaf);
        Ok(())
    }
}
