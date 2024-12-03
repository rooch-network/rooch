// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
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
        let namespace = derive_genesis_namespace(&genesis_bytes);
        println!("DA genesis namespace: {}", namespace);
        Ok(())
    }
}
