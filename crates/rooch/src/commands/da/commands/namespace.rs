// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use rooch_config::da_config::derive_namespace_from_genesis;
use rooch_genesis::RoochGenesis;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::{BuiltinChainID, RoochNetwork};

/// Derive DA namespace from genesis file.
#[derive(Debug, Parser)]
pub struct NamespaceCommand {
    #[clap(long, short = 'n', default_value = "test")]
    chain_id: BuiltinChainID,
}

impl NamespaceCommand {
    pub fn execute(self) -> RoochResult<()> {
        let network: RoochNetwork = RoochNetwork::builtin(self.chain_id);
        let genesis = RoochGenesis::load_or_build(network)?;
        let genesis_hash = genesis.genesis_hash();
        let namespace = derive_namespace_from_genesis(genesis_hash);
        println!("namespace: {}", namespace);
        let encoded_hash = hex::encode(genesis_hash.0);
        println!("genesis hash: {}", encoded_hash);
        Ok(())
    }
}
