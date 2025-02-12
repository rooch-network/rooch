// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::db::commands::init;
use clap::Parser;
use rooch_config::R_OPT_NET_HELP;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use rooch_types::transaction::LedgerTransaction;
use std::path::PathBuf;

/// Get LedgerTransaction by tx_order
#[derive(Debug, Parser)]
pub struct GetTxByOrderCommand {
    /// Transaction's order
    #[clap(long)]
    pub order: u64,

    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    /// If local chainid, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,
}

impl GetTxByOrderCommand {
    pub fn execute(self) -> RoochResult<Option<LedgerTransaction>> {
        let (_root, rooch_db, _start_time) = init(self.base_data_dir, self.chain_id);
        let rooch_store = rooch_db.rooch_store.clone();

        let tx_opt = rooch_store
            .get_transaction_store()
            .get_tx_by_order(self.order)?;
        Ok(tx_opt)
    }
}
