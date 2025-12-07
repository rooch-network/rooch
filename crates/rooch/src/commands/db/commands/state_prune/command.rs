// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
use crate::commands::db::commands::state_prune::replay::ReplayCommand;
use crate::commands::db::commands::state_prune::snapshot::SnapshotCommand;
use async_trait::async_trait;
use clap::Parser;
use rooch_config::RoochOpt;
use rooch_types::error::RoochResult;

/// State prune operations for large-scale state data management
#[derive(Debug, Parser)]
pub struct StatePruneCommand {
    /// The data directory for the node
    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<String>,

    /// Chain ID for the network
    #[clap(long, short = 'n', default_value = "local")]
    pub chain_id: String,

    #[clap(subcommand)]
    pub action: StatePruneAction,
}

#[derive(Debug, Parser)]
pub enum StatePruneAction {
    /// Create a snapshot of active state nodes
    Snapshot(SnapshotCommand),
    /// Replay incremental changesets onto a snapshot
    Replay(ReplayCommand),
}

#[async_trait]
impl CommandAction<String> for StatePruneCommand {
    async fn execute(self) -> RoochResult<String> {
        let context_options = WalletContextOptions {
            base_data_dir: self.base_data_dir,
            chain_id: Some(self.chain_id.clone()),
        };

        let opt = RoochOpt::new_with_args(context_options, vec![])?;

        match self.action {
            StatePruneAction::Snapshot(cmd) => {
                cmd.execute(opt).await
            }
            StatePruneAction::Replay(cmd) => {
                cmd.execute(opt).await
            }
        }
    }
}