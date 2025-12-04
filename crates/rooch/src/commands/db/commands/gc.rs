// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use crate::utils::open_rooch_db;
use crate::utils::open_rooch_db_readonly;
use async_trait::async_trait;
use clap::Parser;
use clap::ValueEnum;
use rooch_pruner::recycle_bin::RecycleBinConfig;
use rooch_pruner::{GCConfig, GarbageCollector};
use rooch_types::error::RoochResult;
use serde::Serialize;
use std::path::PathBuf;
// tracing used only in other modules; no info! here

/// Stop-the-world garbage collection for unreachable state nodes
///
/// This command implements a safe Mark-Sweep garbage collection algorithm that runs
/// while the database is stopped (no concurrent writes). It provides comprehensive
/// safety checks and reporting to prevent accidental data deletion.
#[derive(Debug, Parser)]
pub struct GCCommand {
    /// Base data directory for the blockchain data
    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,

    /// Chain ID to specify which blockchain network
    #[clap(long, short = 'n')]
    pub chain_id: rooch_types::rooch_network::BuiltinChainID,

    /// Dry run - scan and report without making any changes
    ///
    /// In dry-run mode, the command will:
    /// - Verify database safety
    /// - Estimate node count and select optimal strategy
    /// - Run the mark phase to identify reachable nodes
    /// - Report what would be deleted without actually deleting
    #[clap(long)]
    pub dry_run: bool,

    /// Batch size for deletion operations during sweep phase
    ///
    /// Larger batch sizes can improve performance but use more memory
    /// Default: 10,000
    #[clap(long, default_value_t = 10000)]
    pub batch_size: usize,

    /// Number of worker threads for parallel processing
    ///
    /// Controls parallelism during the mark phase
    /// Default: number of CPU cores
    #[clap(long, default_value_t = 4)]
    pub workers: usize,

    /// Enable recycle bin to store deleted nodes for potential recovery
    ///
    /// Deleted nodes are stored in the recycle bin before deletion
    /// Default: true (strong backup mode)
    #[clap(long = "recycle-bin", default_value_t = true)]
    pub use_recycle_bin: bool,

    /// Disk space warning threshold for recycle bin (percentage)
    ///
    /// When recycle bin usage exceeds this percentage of available space,
    /// a warning will be issued. Requires manual cleanup to free space.
    /// Default: 90
    #[clap(long = "recycle-space-warning-threshold", default_value_t = 90)]
    pub recycle_space_warning_threshold: u64,

    /// Force recycle bin operation despite space warnings
    ///
    /// This flag bypasses disk space safety checks for the recycle bin.
    /// Use with caution - may lead to disk space exhaustion.
    #[clap(long = "force-recycle-despite-space-warning")]
    pub force_recycle_despite_space_warning: bool,

    /// Force RocksDB compaction after garbage collection
    ///
    /// Compaction helps reclaim disk space but can be time-consuming
    #[clap(long)]
    pub force_compaction: bool,

    /// Number of recent state roots to protect from garbage collection
    ///
    /// Higher values provide better historical data protection but may use more memory
    /// Default: 0 (auto-detect based on network: Local=1, Dev=1000, Test=1000, Main=30000)
    #[clap(long = "protected-roots-count", default_value_t = 0)]
    pub protected_roots_count: usize,

    /// Skip interactive confirmation prompts (use with caution)
    ///
    /// This option skips the user confirmation step but still performs
    /// technical safety verification (database lock check)
    /// WARNING: Use only for automation when the service is stopped
    #[clap(long = "skip-confirm", alias = "skipConfirm")]
    pub skip_confirm: bool,

    /// Verbose output with detailed progress information
    #[clap(long, short = 'v')]
    pub verbose: bool,

    /// Output results in JSON format for better parsing by automated tools
    #[clap(long)]
    pub json: bool,

    /// Marker strategy for reachability (atomic or locked Bloom)
    #[clap(long, value_enum, default_value_t = MarkerStrategyArg::Locked)]
    pub marker_strategy: MarkerStrategyArg,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum MarkerStrategyArg {
    Atomic,
    Locked,
}

impl GCCommand {
    /// Validate command parameters
    fn validate(&self) -> Result<(), String> {
        // Validate batch size
        if self.batch_size == 0 {
            return Err("Batch size must be greater than 0".to_string());
        }

        // Validate worker count
        if self.workers == 0 {
            return Err("Worker count must be greater than 0".to_string());
        }

        Ok(())
    }

    /// Create GC configuration from command parameters
    fn create_gc_config(&self) -> Result<GCConfig, String> {
        // Create recycle bin configuration with strong backup defaults
        let recycle_bin_config = RecycleBinConfig {
            strong_backup: true, // Always enabled - immutable default
            disk_space_warning_threshold: self.recycle_space_warning_threshold,
            disk_space_critical_threshold: 10, // 10% - trigger emergency cleanup
            disk_space_stop_threshold: 5,      // 5% - stop GC process
            space_check_enabled: !self.force_recycle_despite_space_warning,
        };

        // Resolve protected_roots_count: 0 means auto-detect based on network
        let protected_roots_count = if self.protected_roots_count == 0 {
            // Use network-aware defaults from HistoricalStateConfig
            use rooch_pruner::historical_state::HistoricalStateConfig;
            use rooch_types::framework::chain_id::ChainID;
            let chain_id = ChainID::from(self.chain_id);
            HistoricalStateConfig::new_with_network(&chain_id).protected_roots_count
        } else {
            self.protected_roots_count
        };

        let config = GCConfig {
            // Runtime Configuration
            dry_run: self.dry_run,
            workers: self.workers,
            use_recycle_bin: self.use_recycle_bin,
            recycle_bin: recycle_bin_config,
            force_compaction: self.force_compaction,
            skip_confirm: self.skip_confirm,

            // Core GC Configuration
            scan_batch: 10000, // Default scan batch size
            batch_size: self.batch_size,
            bloom_bits: 1 << 33, // 2^33 bits (1GB)
            protected_roots_count,

            // Marker Configuration - allow auto sizing by default
            marker_bloom_bits: 0, // 0 => auto-size; use --marker-bloom-bits in config to override
            marker_bloom_hash_fns: 4,
            marker_target_fp_rate: 0.01, // 1% false positive rate
            marker_strategy: self.marker_strategy.into(),
        };

        Ok(config)
    }

    /// Format and display GC report
    fn format_report(&self, report: &rooch_pruner::garbage_collector::GCReport) -> String {
        use std::fmt::Write;

        let mut output = String::new();
        writeln!(output, "=== Garbage Collection Report ===").ok();
        writeln!(output).ok();

        // Execution mode
        writeln!(
            output,
            "Execution Mode: {}",
            if self.dry_run {
                "DRY RUN (no changes made)"
            } else {
                "EXECUTE (modifications applied)"
            }
        )
        .ok();
        writeln!(output).ok();

        // Protected roots
        writeln!(
            output,
            "Protected Roots: {} nodes",
            report.protected_roots.len()
        )
        .ok();
        for (i, root) in report.protected_roots.iter().enumerate() {
            writeln!(output, "  {}: {}", i + 1, root).ok();
        }
        writeln!(output).ok();

        // Mark phase statistics
        writeln!(output, "Mark Phase Statistics:").ok();
        writeln!(
            output,
            "  Nodes Marked Reachable: {}",
            report.mark_stats.marked_count
        )
        .ok();
        writeln!(
            output,
            "  Memory Strategy: {}",
            report.mark_stats.memory_strategy
        )
        .ok();
        writeln!(
            output,
            "  Duration: {:.2} seconds",
            report.mark_stats.duration.as_secs_f64()
        )
        .ok();
        writeln!(output).ok();

        // Sweep phase statistics
        writeln!(output, "Sweep Phase Statistics:").ok();
        writeln!(
            output,
            "  Nodes Scanned: {}",
            report.sweep_stats.scanned_count
        )
        .ok();
        writeln!(
            output,
            "  Nodes Kept (Reachable): {}",
            report.sweep_stats.kept_count
        )
        .ok();
        writeln!(
            output,
            "  Nodes Deleted (Unreachable): {}",
            report.sweep_stats.deleted_count
        )
        .ok();
        writeln!(
            output,
            "  Nodes Sent to Recycle Bin: {}",
            report.sweep_stats.recycle_bin_entries
        )
        .ok();
        writeln!(
            output,
            "  Duration: {:.2} seconds",
            report.sweep_stats.duration.as_secs_f64()
        )
        .ok();
        writeln!(output).ok();

        // Summary statistics
        writeln!(output, "Summary:").ok();
        writeln!(
            output,
            "  Memory Strategy Used: {:?}",
            report.memory_strategy_used
        )
        .ok();
        writeln!(
            output,
            "  Total Execution Time: {:.2} seconds",
            report.duration.as_secs_f64()
        )
        .ok();

        if report.sweep_stats.scanned_count > 0 {
            let deletion_rate = (report.sweep_stats.deleted_count as f64
                / report.sweep_stats.scanned_count as f64)
                * 100.0;
            writeln!(
                output,
                "  Space Reclaimed: {:.1}% of scanned nodes",
                deletion_rate
            )
            .ok();
        }
        writeln!(output).ok();

        // Recovery information
        if self.use_recycle_bin && report.sweep_stats.recycle_bin_entries > 0 {
            writeln!(output, "Recovery Information:").ok();
            writeln!(
                output,
                "  {} nodes are stored in the recycle bin",
                report.sweep_stats.recycle_bin_entries
            )
            .ok();
            writeln!(
                output,
                "  Use 'rooch db recycle list' to view recycle bin entries"
            )
            .ok();
            writeln!(
                output,
                "  Use 'rooch db recycle dump <hash>' to view specific deleted nodes"
            )
            .ok();
            writeln!(
                output,
                "  Use 'rooch db recycle restore <hash>' to recover specific nodes"
            )
            .ok();
            writeln!(output).ok();
        }

        // Dry run specific information
        if self.dry_run {
            writeln!(
                output,
                "⚠️  DRY RUN MODE - No changes were made to the database"
            )
            .ok();
            writeln!(
                output,
                "   Run without --dry-run flag to execute the actual garbage collection"
            )
            .ok();
        }

        output
    }

    /// Format report as JSON when --json is specified
    fn format_report_json(
        &self,
        report: &rooch_pruner::garbage_collector::GCReport,
    ) -> RoochResult<String> {
        #[derive(Serialize)]
        struct JsonRoots {
            count: usize,
            roots: Vec<String>,
        }

        #[derive(Serialize)]
        struct JsonMarkStats {
            #[serde(rename = "markedCount")]
            marked_count: u64,
            #[serde(rename = "durationMs")]
            duration_ms: u128,
            #[serde(rename = "memoryStrategy")]
            memory_strategy: String,
        }

        #[derive(Serialize)]
        struct JsonSweepStats {
            #[serde(rename = "scannedCount")]
            scanned_count: u64,
            #[serde(rename = "keptCount")]
            kept_count: u64,
            #[serde(rename = "deletedCount")]
            deleted_count: u64,
            #[serde(rename = "recycleBinEntries")]
            recycle_bin_entries: u64,
            #[serde(rename = "durationMs")]
            duration_ms: u128,
        }

        #[derive(Serialize)]
        struct JsonReport {
            #[serde(rename = "executionMode")]
            execution_mode: String,
            #[serde(rename = "protectedRoots")]
            protected_roots: JsonRoots,
            #[serde(rename = "markStats")]
            mark_stats: JsonMarkStats,
            #[serde(rename = "sweepStats")]
            sweep_stats: JsonSweepStats,
            #[serde(rename = "memoryStrategyUsed")]
            memory_strategy_used: String,
            #[serde(rename = "durationMs")]
            duration_ms: u128,
            #[serde(rename = "spaceReclaimed")]
            space_reclaimed: f64,
        }

        let deletion_ratio = if report.sweep_stats.scanned_count > 0 {
            report.sweep_stats.deleted_count as f64 / report.sweep_stats.scanned_count as f64
                * 100.0
        } else {
            0.0
        };

        let json = JsonReport {
            execution_mode: if self.dry_run {
                "dry-run".to_string()
            } else {
                "execute".to_string()
            },
            protected_roots: JsonRoots {
                count: report.protected_roots.len(),
                roots: report
                    .protected_roots
                    .iter()
                    .map(|h| format!("{:#x}", h))
                    .collect(),
            },
            mark_stats: JsonMarkStats {
                marked_count: report.mark_stats.marked_count,
                duration_ms: report.mark_stats.duration.as_millis(),
                memory_strategy: report.mark_stats.memory_strategy.clone(),
            },
            sweep_stats: JsonSweepStats {
                scanned_count: report.sweep_stats.scanned_count,
                kept_count: report.sweep_stats.kept_count,
                deleted_count: report.sweep_stats.deleted_count,
                recycle_bin_entries: report.sweep_stats.recycle_bin_entries,
                duration_ms: report.sweep_stats.duration.as_millis(),
            },
            memory_strategy_used: report.memory_strategy_used.to_string(),
            duration_ms: report.duration.as_millis(),
            space_reclaimed: deletion_ratio,
        };

        serde_json::to_string_pretty(&json)
            .map_err(|e| rooch_types::error::RoochError::UnexpectedError(e.to_string()))
    }
}

#[async_trait]
impl CommandAction<String> for GCCommand {
    async fn execute(self) -> RoochResult<String> {
        // Validate command parameters
        if let Err(e) = self.validate() {
            return Err(rooch_types::error::RoochError::CommandArgumentError(e));
        }

        // Create GC configuration
        let config = match self.create_gc_config() {
            Ok(config) => config,
            Err(e) => return Err(rooch_types::error::RoochError::CommandArgumentError(e)),
        };

        // Open database (readonly for dry-run, writable for actual execution)
        let (_root_meta, rooch_db, _start) = if self.dry_run {
            open_rooch_db_readonly(
                self.base_data_dir.clone(),
                Some(rooch_types::rooch_network::RoochChainID::Builtin(
                    self.chain_id,
                )),
            )
        } else {
            open_rooch_db(
                self.base_data_dir.clone(),
                Some(rooch_types::rooch_network::RoochChainID::Builtin(
                    self.chain_id,
                )),
            )
        };

        // Create garbage collector - database path will be obtained from store
        let gc = GarbageCollector::new(rooch_db, config)
            .map_err(|e| rooch_types::error::RoochError::UnexpectedError(e.to_string()))?;

        // Execute garbage collection
        let report = gc
            .execute_gc()
            .map_err(|e| rooch_types::error::RoochError::UnexpectedError(e.to_string()))?;

        // Format and return results
        let output = if self.json {
            self.format_report_json(&report)?
        } else {
            self.format_report(&report)
        };

        Ok(output)
    }
}

impl From<MarkerStrategyArg> for rooch_pruner::marker::MarkerStrategy {
    fn from(arg: MarkerStrategyArg) -> Self {
        match arg {
            MarkerStrategyArg::Atomic => rooch_pruner::marker::MarkerStrategy::Atomic,
            MarkerStrategyArg::Locked => rooch_pruner::marker::MarkerStrategy::Locked,
        }
    }
}
