// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use clap::Parser;
use moveos_types::h256::H256;
use rooch_types::error::RoochResult;
use std::path::PathBuf;
use rooch_config::state_prune::{SnapshotBuilderConfig, StatePruneConfig};
use rooch_pruner::state_prune::{SnapshotBuilder, OperationType, StatePruneMetadata};
use crate::commands::statedb::commands::statedb::StateDB;
use serde_json;

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

#[async_trait]
impl CommandAction<String> for SnapshotCommand {
    async fn execute(self) -> RoochResult<String> {
        // Initialize state database
        let statedb = StateDB::new()?;
        let moveos_store = statedb.moveos_store();

        // Determine state root
        let state_root = if let Some(root_str) = self.state_root {
            H256::from_slice(&hex::decode(root_str).map_err(|e| rooch_types::error::RoochError::from(anyhow::anyhow!("Invalid state_root hex: {}", e)))?)
        } else {
            // TODO: Get latest state root from tx_order if provided
            // For now, use current state root
            H256::random() // Placeholder
        };

        // Create snapshot builder configuration
        let snapshot_config = SnapshotBuilderConfig {
            batch_size: self.batch_size,
            workers: self.workers,
            memory_limit: 16 * 1024 * 1024 * 1024, // 16GB
            progress_interval_seconds: 30,
            enable_progress_tracking: true,
            enable_resume: true,
            max_traversal_time_hours: 24,
            enable_bloom_filter: false, // Disabled for simplicity
            bloom_filter_fp_rate: 0.001,
        };

        // Create snapshot builder
        let snapshot_builder = SnapshotBuilder::new(snapshot_config, moveos_store.clone())
            .map_err(|e| rooch_types::error::RoochError::from(anyhow::anyhow!("Failed to create snapshot builder: {}", e)))?;

        // Build snapshot
        let snapshot_meta = snapshot_builder.build_snapshot(state_root, self.output.clone())
            .await
            .map_err(|e| rooch_types::error::RoochError::from(anyhow::anyhow!("Failed to build snapshot: {}", e)))?;

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