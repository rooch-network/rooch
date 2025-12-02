// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use crate::commands::db::commands::recycle_bin::{
    RecycleCleanCommand, RecycleDumpCommand, RecycleListCommand, RecycleRestoreCommand,
};
use async_trait::async_trait;
use clap::Parser;
use rooch_types::error::RoochResult;

/// Recycle bin management commands
#[derive(Debug, Parser)]
pub struct RecycleCommand {
    #[clap(subcommand)]
    pub subcommand: RecycleSubCommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum RecycleSubCommand {
    /// Dump recycle bin record for a specific node hash
    Dump(RecycleDumpCommand),
    /// Restore a node from recycle bin back to state_node
    Restore(RecycleRestoreCommand),
    /// List recycle bin entries with filtering options
    List(RecycleListCommand),
    /// Clean up recycle bin entries with explicit manual control
    Clean(RecycleCleanCommand),
}

#[async_trait]
impl CommandAction<String> for RecycleCommand {
    async fn execute(self) -> RoochResult<String> {
        match self.subcommand {
            RecycleSubCommand::Dump(cmd) => cmd.execute().await,
            RecycleSubCommand::Restore(cmd) => cmd.execute().await,
            RecycleSubCommand::List(cmd) => cmd.execute().await,
            RecycleSubCommand::Clean(cmd) => cmd.execute().await,
        }
    }
}
