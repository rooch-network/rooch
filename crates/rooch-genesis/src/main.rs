// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use clap::Parser;
use rooch_genesis::{genesis_file, RoochGenesis};
use rooch_types::rooch_network::{BuiltinChainID, RoochNetwork};
use tracing::info;

#[derive(Parser)]
#[clap(name = "genesis-release", author = "The Rooch Core Contributors")]
struct GenesisOpts {
    /// The builtin chain id for the genesis
    #[clap(long, short = 'n', default_value = "test")]
    chain_id: BuiltinChainID,
}

fn main() -> Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let opts: GenesisOpts = GenesisOpts::parse();
    match &opts.chain_id {
        BuiltinChainID::Test | BuiltinChainID::Main => {}
        _ => {
            bail!(
                "chain_id {:?} is not supported, only support release test and main",
                opts.chain_id
            );
        }
    }
    info!("start to build genesis for chain: {:?}", opts.chain_id);
    let network: RoochNetwork = RoochNetwork::builtin(opts.chain_id);
    let genesis = RoochGenesis::build(network)?;
    let genesis_file = genesis_file(opts.chain_id);
    genesis.save_to(genesis_file)?;
    Ok(())
}
