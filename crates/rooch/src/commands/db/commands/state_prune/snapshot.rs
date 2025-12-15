// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::utils::open_rooch_db_readonly;
use crate::CommandAction;
use async_trait::async_trait;
use clap::Parser;
use moveos_types::h256::H256;
use rooch_pruner::state_prune::{SnapshotBuilder, SnapshotBuilderConfig};
use rooch_pruner::state_prune::config::DeduplicationStrategy;
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
        let (_root, rooch_db, _start_time) = open_rooch_db_readonly(
            self.base_data_dir,
            Some(rooch_types::rooch_network::RoochChainID::Builtin(
                self.chain_id,
            )),
        );
        let moveos_store = rooch_db.moveos_store;

        // Determine state root
        let state_root = if let Some(root_str) = self.state_root {
            H256::from_slice(&hex::decode(root_str).map_err(|e| {
                rooch_types::error::RoochError::from(anyhow::anyhow!(
                    "Invalid state_root hex: {}",
                    e
                ))
            })?)
        } else if let Some(startup_info) = moveos_store.get_config_store().get_startup_info()? {
            startup_info.state_root
        } else {
            return Err(rooch_types::error::RoochError::from(anyhow::anyhow!(
                "Unable to determine state_root: provide --state-root or ensure startup_info exists"
            )));
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
            deduplication_strategy: DeduplicationStrategy::RocksDB,
            enable_bloom_filter: false, // Disabled in favor of RocksDB strategy
            bloom_filter_fp_rate: 0.001,
            deduplication_batch_size: 0, // Use same as processing batch size
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
            .build_snapshot(state_root, self.output.clone())
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
