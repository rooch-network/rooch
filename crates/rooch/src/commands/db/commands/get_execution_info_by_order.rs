// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::da::commands::TxDAIndexer;
use crate::commands::db::commands::init;
use clap::Parser;
use rooch_config::R_OPT_NET_HELP;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use std::path::PathBuf;

/// Get ExecutionInfo by order
#[derive(Debug, Parser)]
pub struct GetExecutionInfoByOrderCommand {
    #[clap(long)]
    pub order: u64,

    #[clap(
        long = "order-hash-path",
        help = "Path to tx_order:tx_hash:block_number file"
    )]
    pub order_hash_path: PathBuf,

    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    /// If local chainid, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,
}

impl GetExecutionInfoByOrderCommand {
    pub fn execute(self) -> RoochResult<()> {
        let (_root, rooch_db, _start_time) = init(self.base_data_dir, self.chain_id);
        let moveos_store = rooch_db.moveos_store.clone();
        let tx_da_indexer = TxDAIndexer::load_from_file(
            self.order_hash_path.clone(),
            moveos_store.transaction_store,
            rooch_db.rooch_store.clone(),
        )?;

        let tx_order = self.order;

        let execution_info = tx_da_indexer.get_execution_info_by_order(tx_order)?;
        match execution_info {
            Some(_) => {
                println!("{}:{:?}", tx_order, execution_info.unwrap());
            }
            None => {
                tracing::warn!("tx_order {} execution_info not found", tx_order);
            }
        }

        Ok(())
    }
}
