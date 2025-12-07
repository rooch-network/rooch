// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use async_trait::async_trait;
use clap::Parser;
use rooch_types::error::RoochResult;
use std::path::PathBuf;

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

/// Create a snapshot containing only active state nodes
#[derive(Debug, Parser)]
pub struct SnapshotCommand {
    /// Target tx_order to create snapshot from (default: latest)
    #[clap(long)]
    pub tx_order: Option<u64>,

    /// State root hash to create snapshot from (overrides tx_order)
    #[clap(long)]
    pub state_root: Option<String>,

    /// Output directory for the snapshot
    #[clap(long, short = 'o', required = true)]
    pub output: PathBuf,

    /// Batch size for processing nodes
    #[clap(long, default_value = "10000")]
    pub batch_size: usize,

    /// Number of parallel workers
    #[clap(long, default_value = "4")]
    pub workers: usize,

    /// Skip confirmation prompts
    #[clap(long)]
    pub skip_confirm: bool,

    /// Enable verbose logging
    #[clap(long)]
    pub verbose: bool,
}

/// Replay incremental changesets onto a snapshot
#[derive(Debug, Parser)]
pub struct ReplayCommand {
    /// Path to the snapshot directory
    #[clap(long, required = true)]
    pub snapshot: PathBuf,

    /// Starting tx_order for replay (inclusive)
    #[clap(long, required = true)]
    pub from_order: u64,

    /// Ending tx_order for replay (inclusive)
    #[clap(long, required = true)]
    pub to_order: u64,

    /// Output directory for the final pruned database
    #[clap(long, short = 'o', required = true)]
    pub output: PathBuf,

    /// Batch size for processing changesets
    #[clap(long, default_value = "1000")]
    pub batch_size: usize,

    /// Verify final state root consistency
    #[clap(long, default_value = "true")]
    pub verify_root: bool,

    /// Skip confirmation prompts
    #[clap(long)]
    pub skip_confirm: bool,

    /// Enable verbose logging
    #[clap(long)]
    pub verbose: bool,
}

#[async_trait]
impl CommandAction<String> for StatePruneCommand {
    async fn execute(self) -> RoochResult<String> {
        match self.action {
            StatePruneAction::Snapshot(cmd) => {
                let snapshot_info = serde_json::json!({
                    "command": "snapshot",
                    "tx_order": cmd.tx_order,
                    "state_root": cmd.state_root,
                    "output": cmd.output,
                    "batch_size": cmd.batch_size,
                    "workers": cmd.workers,
                    "status": "planned - Phase 2 implementation pending"
                });

                Ok(serde_json::to_string_pretty(&snapshot_info)?)
            }
            StatePruneAction::Replay(cmd) => {
                let replay_info = serde_json::json!({
                    "command": "replay",
                    "snapshot": cmd.snapshot,
                    "from_order": cmd.from_order,
                    "to_order": cmd.to_order,
                    "output": cmd.output,
                    "batch_size": cmd.batch_size,
                    "verify_root": cmd.verify_root,
                    "status": "planned - Phase 2 implementation pending"
                });

                Ok(serde_json::to_string_pretty(&replay_info)?)
            }
        }
    }
}
