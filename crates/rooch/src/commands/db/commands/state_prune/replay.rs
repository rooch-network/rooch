// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::utils::open_rooch_db_readonly;
use crate::CommandAction;
use async_trait::async_trait;
use clap::Parser;
use moveos_types::h256::H256;
use rooch_config::state_prune::{HistoryPruneConfig, ReplayConfig};
use rooch_config::store_config::{DEFAULT_DB_DIR, DEFAULT_DB_STORE_SUBDIR};
use rooch_pruner::state_prune::IncrementalReplayer;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use serde_json;
use std::path::PathBuf;
use std::str::FromStr;

/// Replay incremental changesets onto a snapshot using a fresh output DB
#[derive(Debug, Parser)]
pub struct ReplayCommand {
    /// Base data directory for the blockchain data
    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,

    /// Chain ID to specify which blockchain network
    #[clap(long, short = 'n')]
    pub chain_id: rooch_types::rooch_network::BuiltinChainID,

    /// Path to the snapshot directory
    #[clap(long, required = true)]
    pub snapshot: PathBuf,

    /// Starting tx_order for replay (inclusive)
    #[clap(long, required = true)]
    pub from_order: u64,

    /// Ending tx_order for replay (inclusive)
    #[clap(long, required = true)]
    pub to_order: u64,

    /// Output data directory (base dir). Store will be created at
    /// <output>/<chain>/roochdb/store as a fresh RocksDB database.
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

    /// Keep history from this tx_order (inclusive). Default is from_order
    #[clap(long)]
    pub history_retain_from: Option<u64>,

    /// Alternative: keep history for last N orders
    #[clap(long, conflicts_with = "history_retain_from")]
    pub history_retain_window: Option<u64>,
}

#[async_trait]
impl CommandAction<String> for ReplayCommand {
    async fn execute(self) -> RoochResult<String> {
        // Initialize state database
        let (_root, rooch_db, _start_time) = open_rooch_db_readonly(
            self.base_data_dir,
            Some(rooch_types::rooch_network::RoochChainID::Builtin(
                self.chain_id,
            )),
        );
        let rooch_store = rooch_db.rooch_store;

        let retain_from = if let Some(window) = self.history_retain_window {
            self.to_order.saturating_sub(window).saturating_add(1)
        } else if let Some(from) = self.history_retain_from {
            from
        } else {
            self.from_order
        };

        let history_prune = Some(HistoryPruneConfig {
            enabled: true,
            retain_from,
            retain_window: self.history_retain_window,
            prune_cfs: vec![
                "transaction".to_string(),
                "transaction_execution_info".to_string(),
                "state_change_set".to_string(),
                "tx_sequence_info_mapping".to_string(),
            ],
            dry_run: false,
        });

        // Create replay configuration
        let replay_config = ReplayConfig {
            default_batch_size: self.batch_size,
            verify_final_state_root: self.verify_root,
            validate_after_batch: false, // Simplified for basic implementation
            enable_checkpoints: false,   // Simplified for basic implementation
            checkpoint_interval: self.batch_size, // placeholder: every batch_size changesets
            max_retry_attempts: 3,
            history_prune,
        };

        // Create incremental replayer
        let replayer = IncrementalReplayer::new(replay_config, rooch_store).map_err(|e| {
            rooch_types::error::RoochError::from(anyhow::anyhow!(
                "Failed to create replayer: {}",
                e
            ))
        })?;

        let output_store_dir = self
            .output
            .join(RoochChainID::Builtin(self.chain_id).dir_name())
            .join(DEFAULT_DB_DIR)
            .join(DEFAULT_DB_STORE_SUBDIR);

        // Execute replay
        let replay_report = replayer
            .replay_changesets(
                &self.snapshot,
                self.from_order,
                self.to_order,
                &output_store_dir,
            )
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
            "output_store_dir": output_store_dir,
            "batch_size": self.batch_size,
            "verify_root": self.verify_root,
            "history_prune_enabled": true,
            "history_retain_from": retain_from,
            "replay_report": {
                "changesets_processed": replay_report.changesets_processed,
                "nodes_updated": replay_report.nodes_updated,
                "final_state_root": format!("{:x}", replay_report.final_state_root),
                "verification_passed": replay_report.verification_passed,
                "duration_seconds": replay_report.duration_seconds,
                "errors": replay_report.errors,
                "is_success": replay_report.is_success(),
                "history_prune": replay_report.history_prune_report
            },
            "status": "completed"
        });

        Ok(serde_json::to_string_pretty(&result)?)
    }
}

/// Finalize an existing replay output directory without re-running replay.
#[derive(Debug, Parser)]
pub struct FinalizeReplayOutputCommand {
    /// Base data directory for the blockchain data
    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,

    /// Chain ID to specify which blockchain network
    #[clap(long, short = 'n')]
    pub chain_id: rooch_types::rooch_network::BuiltinChainID,

    /// Output data directory (base dir). Store is expected at
    /// <output>/<chain>/roochdb/store.
    #[clap(long, short = 'o', required = true)]
    pub output: PathBuf,

    /// Ending tx_order for replay (inclusive)
    #[clap(long, required = true)]
    pub to_order: u64,

    /// Optional expected final state root to verify before finalizing.
    #[clap(long)]
    pub expected_state_root: Option<String>,
}

#[async_trait]
impl CommandAction<String> for FinalizeReplayOutputCommand {
    async fn execute(self) -> RoochResult<String> {
        let (_root, rooch_db, _start_time) = open_rooch_db_readonly(
            self.base_data_dir,
            Some(rooch_types::rooch_network::RoochChainID::Builtin(
                self.chain_id,
            )),
        );
        let rooch_store = rooch_db.rooch_store;
        let replay_config = ReplayConfig::default();
        let replayer = IncrementalReplayer::new(replay_config, rooch_store).map_err(|e| {
            rooch_types::error::RoochError::from(anyhow::anyhow!(
                "Failed to create replayer: {}",
                e
            ))
        })?;

        let output_store_dir = self
            .output
            .join(RoochChainID::Builtin(self.chain_id).dir_name())
            .join(DEFAULT_DB_DIR)
            .join(DEFAULT_DB_STORE_SUBDIR);

        let expected_state_root = self
            .expected_state_root
            .as_deref()
            .map(H256::from_str)
            .transpose()
            .map_err(|e| {
                rooch_types::error::RoochError::from(anyhow::anyhow!(
                    "Invalid expected_state_root: {}",
                    e
                ))
            })?;

        let replay_report = replayer
            .finalize_existing_output(&output_store_dir, self.to_order, expected_state_root)
            .map_err(|e| {
                rooch_types::error::RoochError::from(anyhow::anyhow!(
                    "Failed to finalize replay output: {}",
                    e
                ))
            })?;

        let result = serde_json::json!({
            "command": "finalize-replay-output",
            "output": self.output,
            "output_store_dir": output_store_dir,
            "to_order": self.to_order,
            "expected_state_root": self.expected_state_root,
            "replay_report": {
                "final_state_root": format!("{:x}", replay_report.final_state_root),
                "verification_passed": replay_report.verification_passed,
                "duration_seconds": replay_report.duration_seconds,
                "errors": replay_report.errors,
                "is_success": replay_report.is_success(),
                "history_prune": replay_report.history_prune_report
            },
            "status": "completed"
        });

        Ok(serde_json::to_string_pretty(&result)?)
    }
}
