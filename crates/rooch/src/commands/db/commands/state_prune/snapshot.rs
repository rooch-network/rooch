// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::utils::open_rooch_db_readonly;
use crate::CommandAction;
use async_trait::async_trait;
use clap::Parser;
use moveos_types::h256::H256;
use rooch_pruner::state_prune::{SnapshotBuilder, SnapshotBuilderConfig};
use rooch_types::error::RoochResult;
use serde_json;
use std::path::PathBuf;

/// Create a snapshot containing only active state nodes
#[derive(Debug, Parser)]
pub struct SnapshotCommand {
    /// Base data directory for the blockchain data
    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,

    /// Chain ID to specify which blockchain network
    #[clap(long, short = 'n')]
    pub chain_id: rooch_types::rooch_network::BuiltinChainID,

    /// Target tx_order to create snapshot from (default: latest)
    /// When provided, the snapshot will use the state_root and global_size from this transaction
    #[clap(long)]
    pub tx_order: Option<u64>,

    /// State root hash to create snapshot from (overrides --tx_order)
    /// If both --tx_order and --state-root are provided, --state-root takes precedence
    #[clap(long)]
    pub state_root: Option<String>,

    /// Output directory for the snapshot
    #[clap(long, short = 'o', required = true)]
    pub output: PathBuf,

    /// Batch size for processing nodes
    #[clap(long, default_value = "10000")]
    pub batch_size: usize,

    /// Skip confirmation prompts
    #[clap(long)]
    pub skip_confirm: bool,

    /// Enable verbose logging
    #[clap(long)]
    pub verbose: bool,

    /// Force restart even if progress exists
    #[clap(long)]
    pub force_restart: bool,

    /// Disable resume functionality
    #[clap(long)]
    pub no_resume: bool,
}

#[async_trait]
impl CommandAction<String> for SnapshotCommand {
    async fn execute(self) -> RoochResult<String> {
        // Initialize state database
        let (_root, rooch_db, _start_time) = open_rooch_db_readonly(
            self.base_data_dir,
            Some(rooch_types::rooch_network::RoochChainID::Builtin(
                self.chain_id,
            )),
        );
        let moveos_store = rooch_db.moveos_store;
        let rooch_store = rooch_db.rooch_store;

        // Determine tx_order, state_root, and global_size
        // Priority: --state-root > --tx_order > startup_info
        let (tx_order, state_root, global_size) = if let Some(root_str) = self.state_root {
            // --state-root takes precedence over --tx_order
            let state_root = H256::from_slice(&hex::decode(root_str).map_err(|e| {
                rooch_types::error::RoochError::from(anyhow::anyhow!(
                    "Invalid state_root hex: {}",
                    e
                ))
            })?);
            // When state_root is provided explicitly, we don't have tx_order or global_size
            (None, state_root, None)
        } else if let Some(tx_order) = self.tx_order {
            // Look up state_root and global_size from tx_order
            match rooch_store.state_store.get_state_change_set(tx_order)? {
                Some(changeset_ext) => {
                    let state_root = changeset_ext.state_change_set.state_root;
                    let global_size = changeset_ext.state_change_set.global_size;
                    (Some(tx_order), state_root, Some(global_size))
                }
                None => {
                    return Err(rooch_types::error::RoochError::from(anyhow::anyhow!(
                        "No state change set found for tx_order: {}. Ensure the tx_order exists.",
                        tx_order
                    )));
                }
            }
        } else if let Some(startup_info) = moveos_store.get_config_store().get_startup_info()? {
            // Default to latest (startup_info)
            (None, startup_info.state_root, None)
        } else {
            return Err(rooch_types::error::RoochError::from(anyhow::anyhow!(
                "Unable to determine state_root: provide --tx-order, --state-root, or ensure startup_info exists"
            )));
        };

        // Create snapshot builder configuration
        let snapshot_config = SnapshotBuilderConfig {
            batch_size: self.batch_size,
            memory_limit: 16 * 1024 * 1024 * 1024, // 16GB
            progress_interval_seconds: 30,
            enable_resume: !self.no_resume, // Respect CLI flag
            enable_adaptive_batching: true,
            memory_pressure_threshold: 0.8,
        };

        // Create snapshot builder
        let snapshot_builder = SnapshotBuilder::new(snapshot_config, moveos_store.clone())
            .map_err(|e| {
                rooch_types::error::RoochError::from(anyhow::anyhow!(
                    "Failed to create snapshot builder: {}",
                    e
                ))
            })?;

        // Build snapshot
        let snapshot_meta = snapshot_builder
            .build_snapshot(
                state_root,
                tx_order.unwrap_or(0),
                global_size.unwrap_or(0),
                self.output.clone(),
                self.force_restart,
            )
            .await
            .map_err(|e| {
                rooch_types::error::RoochError::from(anyhow::anyhow!(
                    "Failed to build snapshot: {}",
                    e
                ))
            })?;

        let result = serde_json::json!({
            "command": "snapshot",
            "state_root": format!("{:x}", state_root),
            "output": self.output,
            "snapshot_meta": {
                "tx_order": snapshot_meta.tx_order,
                "state_root": format!("{:x}", snapshot_meta.state_root),
                "global_size": snapshot_meta.global_size,
                "node_count": snapshot_meta.node_count,
                "version": snapshot_meta.version,
                "created_at": snapshot_meta.created_at
            },
            "status": "completed"
        });

        Ok(serde_json::to_string_pretty(&result)?)
    }
}
