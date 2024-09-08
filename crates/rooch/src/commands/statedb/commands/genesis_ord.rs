// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;
use std::sync::{mpsc, Arc};
use std::thread;

use clap::Parser;

use rooch_config::R_OPT_NET_HELP;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;

use crate::commands::statedb::commands::genesis::{
    apply_inscription_updates, produce_inscription_updates,
};
use crate::commands::statedb::commands::init_rooch_db;
use crate::commands::statedb::commands::inscription::InscriptionStats;

/// Import BTC ordinals & UTXO for genesis
#[derive(Debug, Parser)]
pub struct GenesisOrdCommand {
    #[clap(long)]
    /// ord source data file. like ~/.rooch/local/ord or ord, ord_input must be sorted by sequence_number
    /// The file format is JSON, and the first line is block height info: # export at block height <N>, ord range: [0, N).
    /// ord_input & utxo_input must be at the same height
    pub ord_source: PathBuf,
    #[clap(long)]
    /// ord stats file, like ~/.rooch/local/ord_stats or ord_stats
    pub ord_stats: PathBuf,
    #[clap(
        long,
        default_value = "1048576",
        help = "batch size submitted to state db. Set it smaller if memory is limited."
    )] // ord may have a large body, so set a smaller batch
    pub ord_batch_size: Option<usize>,

    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    /// If local chainid, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,
}

impl GenesisOrdCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let rooch_db = init_rooch_db(self.base_data_dir.clone(), self.chain_id.clone());
        let moveos_store = rooch_db.moveos_store;
        let moveos_store = Arc::new(moveos_store);

        let inscription_stats = InscriptionStats::load_from_file(self.ord_stats.clone());
        let input_path = self.ord_source.clone();
        let batch_size = self.ord_batch_size.unwrap();
        let (ord_tx, ord_rx) = mpsc::sync_channel(2);
        let produce_inscription_updates_thread =
            thread::spawn(move || produce_inscription_updates(ord_tx, input_path, batch_size));
        let moveos_store_clone = Arc::clone(&moveos_store);
        let apply_inscription_updates_thread = thread::spawn(move || {
            apply_inscription_updates(ord_rx, moveos_store_clone, None, inscription_stats);
        });

        produce_inscription_updates_thread.join().unwrap();

        apply_inscription_updates_thread.join().unwrap();

        Ok(())
    }
}
