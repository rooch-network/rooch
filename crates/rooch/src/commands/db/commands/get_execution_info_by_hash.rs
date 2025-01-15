// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::da::commands::TxDAIndexer;
use crate::commands::db::commands::init;
use clap::Parser;
use moveos_store::transaction_store::TransactionStore;
use moveos_types::h256::H256;
use rooch_config::R_OPT_NET_HELP;
use rooch_rpc_api::jsonrpc_types::transaction_view::TransactionWithInfoView;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use std::path::PathBuf;

/// Get ExecutionInfo by tx_hash
#[derive(Debug, Parser)]
pub struct GetExecutionInfoByHashCommand {
    /// Transaction's hash
    #[clap(long)]
    pub hash: H256,

    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    /// If local chainid, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,
}

impl GetExecutionInfoByHashCommand {
    pub fn execute(self) -> RoochResult<()> {
        let (_root, rooch_db, _start_time) = init(self.base_data_dir, self.chain_id);
        let moveos_store = rooch_db.moveos_store.clone();

        let tx_hash = self.hash;
        let execution_info = moveos_store
            .get_transaction_store()
            .get_tx_execution_info(self.hash)?;
        match execution_info {
            Some(_) => {
                println!("{:?}:{:?}", tx_hash, execution_info.unwrap());
            }
            None => {
                tracing::warn!("execution_info not found for tx_hash: {} ", tx_hash);
            }
        }

        Ok(())
    }
}
