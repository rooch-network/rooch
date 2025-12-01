// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use crate::utils::open_rooch_db;
use crate::utils::open_rooch_db_readonly;
use async_trait::async_trait;
use clap::Parser;
use rooch_pruner::{GCConfig, GarbageCollector, MarkerStrategy};
use rooch_pruner::recycle_bin::RecycleBinConfig;
use rooch_types::error::RoochResult;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
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

    /// Marker strategy for tracking reachable nodes
    ///
    /// Available options:
    /// - auto: Automatically select based on available memory
    /// - memory: Use in-memory HashSet (faster, uses more RAM)
    /// - persistent: Use temporary RocksDB column family (slower, less RAM)
    #[clap(long = "marker-strategy", default_value = "auto")]
    pub marker_strategy: String,

    /// Number of recent state roots to protect from garbage collection
    ///
    /// Higher values provide better historical data protection but may use more memory
    /// Default: 1 (backward compatible with existing behavior)
    #[clap(long = "protected-roots-count", default_value_t = 1)]
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
}

impl GCCommand {
    /// Parse marker strategy from string input
    fn parse_marker_strategy(&self) -> Result<MarkerStrategy, String> {
        match self.marker_strategy.as_str() {
            "auto" => Ok(MarkerStrategy::Auto),
            "memory" => Ok(MarkerStrategy::InMemory),
            "persistent" => Ok(MarkerStrategy::Persistent),
            _ => Err("Invalid marker strategy. Use: auto, memory, or persistent".to_string()),
        }
    }

    /// Validate command parameters
    fn validate(&self) -> Result<(), String> {
        // Validate marker strategy
        self.parse_marker_strategy()?;

        // Validate batch size
        if self.batch_size == 0 {
            return Err("Batch size must be greater than 0".to_string());
        }

        // Validate worker count
        if self.workers == 0 {
            return Err("Worker count must be greater than 0".to_string());
        }

        // Validate protected roots count
        if self.protected_roots_count == 0 {
            return Err("Protected roots count must be greater than 0".to_string());
        }

        Ok(())
    }

    /// Create GC configuration from command parameters
    fn create_gc_config(&self) -> Result<GCConfig, String> {
        let marker_strategy = self.parse_marker_strategy()?;

        // Create recycle bin configuration with strong backup defaults
        let recycle_bin_config = RecycleBinConfig {
            strong_backup: true, // Always enabled - immutable default
            disk_space_warning_threshold: self.recycle_space_warning_threshold,
            space_check_enabled: !self.force_recycle_despite_space_warning,
        };

        let config = GCConfig {
            // Runtime Configuration
            dry_run: self.dry_run,
            workers: self.workers,
            use_recycle_bin: self.use_recycle_bin,
            force_compaction: self.force_compaction,
            skip_confirm: self.skip_confirm,

            // Core GC Configuration
            scan_batch: 10000, // Default scan batch size
            batch_size: self.batch_size,
            bloom_bits: 8589934592,   // 2^33 bits (1GB)
            protection_orders: 30000, // Default protection orders
            protected_roots_count: self.protected_roots_count,

            // Marker Strategy Configuration
            marker_strategy,
            marker_batch_size: 10000,
            marker_bloom_bits: 1048576, // 2^20 bits (1MB)
            marker_bloom_hash_fns: 4,
            marker_memory_threshold_mb: 1024, // 1GB
            marker_auto_strategy: true,
            marker_force_persistent: false,
            marker_temp_cf_name: "gc_marker_temp".to_string(),
            marker_error_recovery: true,

            // Recycle Bin Configuration
            recycle_bin: recycle_bin_config,
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
                "  Use 'rooch db recycle-stat' to view recycle bin statistics"
            )
            .ok();
            writeln!(
                output,
                "  Use 'rooch db recycle-dump <hash>' to view specific deleted nodes"
            )
            .ok();
            writeln!(
                output,
                "  Use 'rooch db recycle-restore <hash>' to recover specific nodes"
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

        // Note: Logging is now handled globally in main.rs with ERROR level by default
        // This prevents info! statements from contaminating JSON output
        // Set GC_VERBOSE_MODE=1 if INFO level logging is needed for debugging

        // Create GC configuration
        let config = match self.create_gc_config() {
            Ok(config) => config,
            Err(e) => return Err(rooch_types::error::RoochError::CommandArgumentError(e)),
        };

        // Note: Removed debug logging for cleaner JSON output when --json flag is used
        // info!("Starting garbage collection with config: {:?}", config);

        // Create config options to get database path
        let opt = rooch_config::RoochOpt::new_with_default(
            self.base_data_dir.clone(),
            Some(rooch_types::rooch_network::RoochChainID::Builtin(
                self.chain_id,
            )),
            None,
        )
        .map_err(|e| rooch_types::error::RoochError::UnexpectedError(e.to_string()))?;

        // Get database path from config
        let db_path = opt.store_config().get_store_dir();

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

        // Create garbage collector with database path
        let gc = GarbageCollector::new(Arc::new(rooch_db.moveos_store.clone()), config, db_path)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_marker_strategy() {
        let command = GCCommand {
            base_data_dir: None,
            chain_id: rooch_types::rooch_network::BuiltinChainID::Dev,
            dry_run: false,
            batch_size: 1000,
            workers: 4,
            use_recycle_bin: true,
            force_compaction: false,
            marker_strategy: "auto".to_string(),
            protected_roots_count: 1,
            skip_confirm: false,
            verbose: false,
            json: false,
        };

        assert!(matches!(
            command.parse_marker_strategy(),
            Ok(MarkerStrategy::Auto)
        ));

        let command = GCCommand {
            base_data_dir: None,
            chain_id: rooch_types::rooch_network::BuiltinChainID::Dev,
            dry_run: false,
            batch_size: 1000,
            workers: 4,
            use_recycle_bin: true,
            force_compaction: false,
            marker_strategy: "memory".to_string(),
            protected_roots_count: 1,
            skip_confirm: false,
            verbose: false,
            json: false,
        };
        assert!(matches!(
            command.parse_marker_strategy(),
            Ok(MarkerStrategy::InMemory)
        ));

        let command = GCCommand {
            base_data_dir: None,
            chain_id: rooch_types::rooch_network::BuiltinChainID::Dev,
            dry_run: false,
            batch_size: 1000,
            workers: 4,
            use_recycle_bin: true,
            force_compaction: false,
            marker_strategy: "persistent".to_string(),
            protected_roots_count: 1,
            skip_confirm: false,
            verbose: false,
            json: false,
        };
        assert!(matches!(
            command.parse_marker_strategy(),
            Ok(MarkerStrategy::Persistent)
        ));

        let command = GCCommand {
            base_data_dir: None,
            chain_id: rooch_types::rooch_network::BuiltinChainID::Dev,
            dry_run: false,
            batch_size: 1000,
            workers: 4,
            use_recycle_bin: true,
            force_compaction: false,
            marker_strategy: "invalid".to_string(),
            skip_confirm: false,
            verbose: false,
            protected_roots_count: 1,
            json: false,
        };
        assert!(command.parse_marker_strategy().is_err());
    }

    #[test]
    fn test_validation() {
        let valid_command = GCCommand {
            protected_roots_count: 1,
            base_data_dir: None,
            chain_id: rooch_types::rooch_network::BuiltinChainID::Dev,
            dry_run: true,
            batch_size: 1000,
            workers: 4,
            use_recycle_bin: true,
            force_compaction: false,
            marker_strategy: "auto".to_string(),
            skip_confirm: false,
            verbose: false,
            json: false,
        };
        assert!(valid_command.validate().is_ok());

        // Test invalid batch size
        let invalid_command = GCCommand {
            protected_roots_count: 1,
            base_data_dir: None,
            chain_id: rooch_types::rooch_network::BuiltinChainID::Dev,
            dry_run: true,
            batch_size: 0,
            workers: 4,
            use_recycle_bin: true,
            force_compaction: false,
            marker_strategy: "auto".to_string(),
            skip_confirm: false,
            verbose: false,
            json: false,
        };
        assert!(invalid_command.validate().is_err());

        // Test invalid worker count
        let invalid_command = GCCommand {
            protected_roots_count: 1,
            base_data_dir: None,
            chain_id: rooch_types::rooch_network::BuiltinChainID::Dev,
            dry_run: true,
            batch_size: 1000,
            workers: 0,
            use_recycle_bin: true,
            force_compaction: false,
            marker_strategy: "auto".to_string(),
            skip_confirm: false,
            verbose: false,
            json: false,
        };
        assert!(invalid_command.validate().is_err());

        // Test validation for write operations with new simplified logic
        let valid_write_command_skip_confirm = GCCommand {
            protected_roots_count: 1,
            base_data_dir: None,
            chain_id: rooch_types::rooch_network::BuiltinChainID::Dev,
            dry_run: false,
            batch_size: 1000,
            workers: 4,
            use_recycle_bin: true,
            force_compaction: false,
            marker_strategy: "auto".to_string(),
            skip_confirm: true, // Skip confirmation for automation
            verbose: false,
            json: false,
        };
        assert!(valid_write_command_skip_confirm.validate().is_ok());

        let valid_write_command_with_confirm = GCCommand {
            protected_roots_count: 1,
            base_data_dir: None,
            chain_id: rooch_types::rooch_network::BuiltinChainID::Dev,
            dry_run: false,
            batch_size: 1000,
            workers: 4,
            use_recycle_bin: true,
            force_compaction: false,
            marker_strategy: "auto".to_string(),
            skip_confirm: false, // Will require user confirmation
            verbose: false,
            json: false,
        };
        assert!(valid_write_command_with_confirm.validate().is_ok());
    }

    #[test]
    fn test_config_creation() {
        let command = GCCommand {
            base_data_dir: None,
            chain_id: rooch_types::rooch_network::BuiltinChainID::Dev,
            dry_run: true,
            batch_size: 5000,
            workers: 8,
            use_recycle_bin: false,
            force_compaction: true,
            marker_strategy: "memory".to_string(),
            skip_confirm: true,
            verbose: false,
            protected_roots_count: 1,
            json: false,
        };

        let config = command.create_gc_config().unwrap();
        assert!(config.dry_run);
        assert_eq!(config.batch_size, 5000);
        assert_eq!(config.workers, 8);
        assert!(!config.use_recycle_bin);
        assert!(config.force_compaction);
        assert_eq!(config.marker_strategy, MarkerStrategy::InMemory);
        assert!(config.skip_confirm);
    }
}
