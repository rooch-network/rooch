// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use crate::utils::open_rooch_db;
use crate::utils::open_rooch_db_readonly;
use async_trait::async_trait;
use clap::Parser;
use rooch_pruner::{GCConfig, GarbageCollector, MarkerStrategy};
use rooch_types::error::RoochResult;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

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
    /// Default: true
    #[clap(long = "recycle-bin", default_value_t = true)]
    pub use_recycle_bin: bool,

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

    /// Force execution to bypass user confirmation prompts
    ///
    /// Confirm that you understand the risks and have stopped the blockchain service
    /// Technical safety verification (database lock check) will still be performed automatically
    /// WARNING: Use only when the service is stopped and no other processes are accessing the database
    #[clap(long)]
    pub force: bool,

    /// Verbose output with detailed progress information
    #[clap(long, short = 'v')]
    pub verbose: bool,
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

        // For write operations, safety verification is mandatory
        if !self.dry_run && !self.force {
            return Err(
                "GC modifies database state. Use --force to confirm you understand the risks and have stopped the blockchain service, or use --dry-run to preview changes.".to_string()
            );
        }

        Ok(())
    }

    /// Create GC configuration from command parameters
    fn create_gc_config(&self) -> Result<GCConfig, String> {
        let marker_strategy = self.parse_marker_strategy()?;

        let config = GCConfig {
            dry_run: self.dry_run,
            batch_size: self.batch_size,
            workers: self.workers,
            use_recycle_bin: self.use_recycle_bin,
            force_compaction: self.force_compaction,
            marker_strategy,
            force_execution: self.force,
            protected_roots_count: self.protected_roots_count,
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
                "   Use --force flag to execute the actual garbage collection"
            )
            .ok();
        }

        output
    }
}

#[async_trait]
impl CommandAction<String> for GCCommand {
    async fn execute(self) -> RoochResult<String> {
        // Validate command parameters
        if let Err(e) = self.validate() {
            return Err(rooch_types::error::RoochError::CommandArgumentError(e));
        }

        // Setup logging based on verbose flag
        if self.verbose {
            match tracing_subscriber::fmt()
                .with_max_level(tracing::Level::INFO)
                .try_init()
            {
                Ok(_) => info!("Verbose logging initialized"),
                Err(_) => info!("Logging already initialized, using existing configuration"),
            }
        }

        // Create GC configuration
        let config = match self.create_gc_config() {
            Ok(config) => config,
            Err(e) => return Err(rooch_types::error::RoochError::CommandArgumentError(e)),
        };

        info!("Starting garbage collection with config: {:?}", config);

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
        let output = self.format_report(&report);

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
            force: false,
            verbose: false,
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
            force: false,
            verbose: false,
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
            force: false,
            verbose: false,
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
            force: false,
            verbose: false,
            protected_roots_count: 1,
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
            force: false,
            verbose: false,
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
            force: false,
            verbose: false,
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
            force: false,
            verbose: false,
        };
        assert!(invalid_command.validate().is_err());

        // Test missing force flag for write operations (should fail without --force)
        let invalid_command = GCCommand {
            protected_roots_count: 1,
            base_data_dir: None,
            chain_id: rooch_types::rooch_network::BuiltinChainID::Dev,
            dry_run: false,
            batch_size: 1000,
            workers: 4,
            use_recycle_bin: true,
            force_compaction: false,
            marker_strategy: "auto".to_string(),
            force: false,
            verbose: false,
        };
        assert!(invalid_command.validate().is_err());

        // Test valid case with --force flag for write operations
        let valid_write_command = GCCommand {
            protected_roots_count: 1,
            base_data_dir: None,
            chain_id: rooch_types::rooch_network::BuiltinChainID::Dev,
            dry_run: false,
            batch_size: 1000,
            workers: 4,
            use_recycle_bin: true,
            force_compaction: false,
            marker_strategy: "auto".to_string(),
            force: true,
            verbose: false,
        };
        assert!(valid_write_command.validate().is_ok());
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
            force: true,
            verbose: false,
            protected_roots_count: 1,
        };

        let config = command.create_gc_config().unwrap();
        assert!(config.dry_run);
        assert_eq!(config.batch_size, 5000);
        assert_eq!(config.workers, 8);
        assert!(!config.use_recycle_bin);
        assert!(config.force_compaction);
        assert_eq!(config.marker_strategy, MarkerStrategy::InMemory);
        assert!(config.force_execution);
    }
}
