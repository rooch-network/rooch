// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use rooch_config::da_config::derive_namespace_from_genesis;
use rooch_genesis::{RoochGenesis, RoochGenesisV2};
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::{BuiltinChainID, RoochNetwork};
use std::path::PathBuf;

/// Derive DA namespace from genesis file.
#[derive(Debug, Parser)]
pub struct NamespaceCommand {
    #[clap(long)]
    genesis_file: Option<PathBuf>,
    #[clap(long, short = 'n', default_value = "test")]
    chain_id: Option<BuiltinChainID>,
}

impl NamespaceCommand {
    pub fn execute(self) -> RoochResult<()> {
        let genesis = if let Some(genesis_file) = self.genesis_file {
            load_genesis_from_file(genesis_file)?
        } else {
            RoochGenesisV2::load_or_build(RoochNetwork::builtin(self.chain_id.unwrap()))?
        };

        let genesis_v1 = RoochGenesis::from(genesis);
        let genesis_hash = genesis_v1.genesis_hash();
        let namespace = derive_namespace_from_genesis(genesis_hash);
        println!("namespace: {}", namespace);
        let encoded_hash = hex::encode(genesis_hash.0);
        println!("genesis hash: {}", encoded_hash);
        Ok(())
    }
}

fn load_genesis_from_file(path: PathBuf) -> anyhow::Result<RoochGenesisV2> {
    let contents = std::fs::read(path)?;
    RoochGenesisV2::decode(&contents)
}
