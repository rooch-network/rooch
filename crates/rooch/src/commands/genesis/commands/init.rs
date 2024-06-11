// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use rooch_config::{RoochOpt, R_OPT_NET_HELP};
use rooch_db::RoochDB;
use rooch_genesis::RoochGenesis;
use rooch_types::{error::RoochResult, rooch_network::RoochChainID};
use std::path::PathBuf;

/// Init genesis statedb
#[derive(Debug, Parser)]
pub struct InitCommand {
    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    /// If local chainid, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,

    #[clap(long)]
    /// The genesis config file path for custom chain network.
    /// If the file path equals to builtin chain network name(local/dev/test/main), will use builtin genesis config.
    pub genesis_config: Option<String>,
}

impl InitCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let opt =
            RoochOpt::new_with_default(self.base_data_dir, self.chain_id, self.genesis_config)?;
        let store_config = opt.store_config();
        let rooch_db = RoochDB::init(store_config)?;
        let network = opt.network();
        let genesis = RoochGenesis::build(network)?;
        let root = genesis.init_genesis(&rooch_db)?;
        println!(
            "Genesis statedb initialized at {:?} successfully, state_root: {:?}",
            opt.base().data_dir(),
            root.state_root()
        );
        Ok(())
    }
}
