// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::utils::open_rooch_db;
use clap::Parser;
use rooch_config::R_OPT_NET_HELP;
use rooch_store::meta_store::MetaStore;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use rooch_types::sequencer::SequencerInfo;
use std::path::PathBuf;

/// Get ExecutionInfo by tx_hash
#[derive(Debug, Parser)]
pub struct GetSequencerInfoCommand {
    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,
}

impl GetSequencerInfoCommand {
    pub fn execute(self) -> RoochResult<Option<SequencerInfo>> {
        let (_root, rooch_db, _start_time) = open_rooch_db(self.base_data_dir, self.chain_id);
        let moveos_store = rooch_db.rooch_store.clone();

        let sequencer_info = moveos_store.get_sequencer_info()?;
        Ok(sequencer_info)
    }
}
