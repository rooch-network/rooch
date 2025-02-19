// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::utils::open_rooch_db;
use clap::Parser;
use moveos_types::state::StateChangeSetExt;
use rooch_config::R_OPT_NET_HELP;
use rooch_store::state_store::StateStore;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use std::path::PathBuf;

/// Get changeset by order
#[derive(Debug, Parser)]
pub struct GetChangesetByOrderCommand {
    #[clap(long)]
    pub order: u64,

    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,
}

impl GetChangesetByOrderCommand {
    pub async fn execute(self) -> RoochResult<Option<StateChangeSetExt>> {
        let (_root, rooch_db, _start_time) = open_rooch_db(self.base_data_dir, self.chain_id);
        let rooch_store = rooch_db.rooch_store;
        let tx_order = self.order;
        let state_change_set_ext_opt = rooch_store.get_state_change_set(tx_order)?;
        Ok(state_change_set_ext_opt)
    }
}
