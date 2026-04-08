// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::db::commands::state_prune::replay::{
    FinalizeReplayOutputCommand, ReplayCommand, TailReplayCommand,
};
use crate::commands::db::commands::state_prune::snapshot::SnapshotCommand;
use crate::CommandAction;
use async_trait::async_trait;
use clap::Parser;
use rooch_types::error::RoochResult;

/// State prune operations for large-scale state data management
#[derive(Debug, Parser)]
pub struct StatePruneCommand {
    #[clap(subcommand)]
    pub action: StatePruneAction,
}

#[derive(Debug, Parser)]
pub enum StatePruneAction {
    /// Create a snapshot of active state nodes
    Snapshot(SnapshotCommand),
    /// Replay incremental changesets onto a snapshot using a fresh output DB
    Replay(ReplayCommand),
    /// Replay only the delta order range onto an existing replay output DB
    TailReplay(TailReplayCommand),
    /// Finalize an existing replay output directory after replay body has completed
    FinalizeReplayOutput(FinalizeReplayOutputCommand),
}

#[async_trait]
impl CommandAction<String> for StatePruneCommand {
    async fn execute(self) -> RoochResult<String> {
        match self.action {
            StatePruneAction::Snapshot(cmd) => cmd.execute().await,
            StatePruneAction::Replay(cmd) => cmd.execute().await,
            StatePruneAction::TailReplay(cmd) => cmd.execute().await,
            StatePruneAction::FinalizeReplayOutput(cmd) => cmd.execute().await,
        }
    }
}
