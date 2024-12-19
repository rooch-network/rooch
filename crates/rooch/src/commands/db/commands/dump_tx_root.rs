// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::da::commands::TxDAIndexer;
use crate::commands::db::commands::init;
use clap::Parser;
use rooch_config::R_OPT_NET_HELP;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

/// Get changeset by order
#[derive(Debug, Parser)]
pub struct DumpTxRootCommand {
    #[clap(long, help = "start tx order")]
    pub start: u64,
    #[clap(long, help = "total tx count, [start, start+limit)")]
    pub limit: u64,
    #[clap(
        long = "order-hash-path",
        help = "Path to tx_order:tx_hash:block_number file"
    )]
    pub order_hash_path: PathBuf,
    #[clap(long, help = "tx_order:state_root output file path")]
    pub output: PathBuf,

    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    /// If local chainid, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,
}

impl DumpTxRootCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let (_root, rooch_db, _start_time) = init(self.base_data_dir, self.chain_id);
        let moveos_store = rooch_db.moveos_store.clone();
        let tx_da_indexer = TxDAIndexer::load_from_file(
            self.order_hash_path.clone(),
            moveos_store.transaction_store,
        )?;

        let file = File::create(self.output.clone())?;
        let mut writer = BufWriter::with_capacity(8 * 1024 * 1024, file.try_clone().unwrap());

        for tx_order in self.start..self.start + self.limit {
            let execution_info = tx_da_indexer.get_execution_info_by_order(tx_order)?;
            if execution_info.is_none() {
                tracing::warn!("tx_order {} execution_info not found", tx_order);
                continue;
            }
            writeln!(
                writer,
                "{}:{:?}",
                tx_order,
                execution_info.unwrap().state_root
            )?;
        }
        writer.flush()?;
        file.sync_data()?;

        Ok(())
    }
}
