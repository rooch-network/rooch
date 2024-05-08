// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use crate::commands::statedb::commands::genesis_utxo::GenesisUTXOCommand;
use crate::commands::statedb::commands::import::ImportCommand;
use async_trait::async_trait;
use clap::Parser;
use commands::export::ExportCommand;
use rooch_types::error::RoochResult;

pub mod commands;

/// Statedb Commands
#[derive(Parser)]
pub struct Statedb {
    #[clap(subcommand)]
    cmd: StatedbCommand,
}

#[async_trait]
impl CommandAction<String> for Statedb {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            StatedbCommand::Export(export) => export.execute().await.map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
            StatedbCommand::Import(import) => import.execute().await.map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
            StatedbCommand::GenesisUTXO(genesis_utxo) => genesis_utxo.execute().await.map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
        }
    }
}

#[derive(clap::Subcommand)]
#[clap(name = "statedb")]
pub enum StatedbCommand {
    Export(ExportCommand),
    Import(ImportCommand),
    GenesisUTXO(GenesisUTXOCommand),
}
