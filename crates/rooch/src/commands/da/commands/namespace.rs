// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hex::encode;
use moveos_types::h256::sha2_256_of;
use rooch_config::da_config::derive_genesis_namespace;
use rooch_types::error::RoochResult;
use std::path::PathBuf;

/// Derive DA namespace from genesis file.
#[derive(Debug, Parser)]
pub struct NamespaceCommand {
    #[clap(long = "genesis-file-path")]
    pub genesis_file_path: PathBuf,
}

impl NamespaceCommand {
    pub fn execute(self) -> RoochResult<()> {
        let genesis_bytes = std::fs::read(&self.genesis_file_path)?;
        let full_hash = encode(sha2_256_of(&genesis_bytes).0);
        let namespace = derive_genesis_namespace(&genesis_bytes);
        println!("DA genesis namespace: {}", namespace);
        println!("DA genesis full hash: {}", full_hash);
        Ok(())
    }
}
