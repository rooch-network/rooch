// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::utils::open_rooch_db_readonly;
use crate::CommandAction;
use async_trait::async_trait;
use clap::Parser;
use rooch_config::state_prune::ReplayConfig;
use rooch_config::RoochOpt;
use rooch_pruner::state_prune::IncrementalReplayer;
use rooch_types::error::RoochResult;
use serde_json;
use std::path::PathBuf;

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
impl CommandAction<String> for ReplayCommand {
    async fn execute(self, opt: RoochOpt) -> RoochResult<String> {
        // Initialize state database
        let (_root, rooch_db, _start_time) =
            open_rooch_db_readonly(opt.store_config_base_data_dir(), Some(opt.chain_id.id));
        let moveos_store = rooch_db.moveos_store();

        // Create replay configuration
        let replay_config = ReplayConfig {
            default_batch_size: self.batch_size,
            verify_final_state_root: self.verify_root,
            validate_after_batch: false, // Simplified for basic implementation
            enable_checkpoints: false,   // Simplified for basic implementation
            checkpoint_interval: self.batch_size, // placeholder: every batch_size changesets
            max_retry_attempts: 3,
        };

        // Create incremental replayer
        let replayer = IncrementalReplayer::new(replay_config, moveos_store).map_err(|e| {
            rooch_types::error::RoochError::from(anyhow::anyhow!(
                "Failed to create replayer: {}",
                e
            ))
        })?;

        // Execute replay
        let replay_report = replayer
            .replay_changesets(&self.snapshot, self.from_order, self.to_order, &self.output)
            .await
            .map_err(|e| {
                rooch_types::error::RoochError::from(anyhow::anyhow!(
                    "Failed to execute replay: {}",
                    e
                ))
            })?;

        let result = serde_json::json!({
            "command": "replay",
            "snapshot": self.snapshot,
            "from_order": self.from_order,
            "to_order": self.to_order,
            "output": self.output,
            "batch_size": self.batch_size,
            "verify_root": self.verify_root,
            "replay_report": {
                "changesets_processed": replay_report.changesets_processed,
                "nodes_updated": replay_report.nodes_updated,
                "final_state_root": format!("{:x}", replay_report.final_state_root),
                "verification_passed": replay_report.verification_passed,
                "duration_seconds": replay_report.duration_seconds,
                "errors": replay_report.errors,
                "is_success": replay_report.is_success()
            },
            "status": "completed"
        });

        Ok(serde_json::to_string_pretty(&result)?)
    }
}
