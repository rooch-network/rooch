// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use clap::Parser;

use rooch_types::error::RoochResult;

use crate::cli_types::CommandAction;
use crate::commands::indexer::commands::bench::BenchCommand;
use crate::commands::indexer::commands::rebuild::RebuildCommand;

pub mod commands;

/// Indexer Commands
#[derive(Parser)]
pub struct Indexer {
    #[clap(subcommand)]
    cmd: IndexerCommand,
}

#[async_trait]
impl CommandAction<String> for Indexer {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            IndexerCommand::Rebuild(rebuild) => rebuild.execute().await.map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
            IndexerCommand::Bench(bench) => bench.execute().await.map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
        }
    }
}

#[derive(clap::Subcommand)]
#[clap(name = "indexer")]
pub enum IndexerCommand {
    Rebuild(RebuildCommand),
    Bench(BenchCommand),
}
