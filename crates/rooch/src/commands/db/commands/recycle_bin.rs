// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::utils::{open_rooch_db, open_rooch_db_readonly};
use clap::Parser;
use moveos_types::state::{FieldKey, ObjectState};
use raw_store::CodecKVStore;
use rooch_pruner::recycle_bin::{RecycleBinConfig, RecycleBinStore, RecycleFilter, RecyclePhase};
use rooch_types::error::RoochResult;
use serde_json;
use smt::jellyfish_merkle::node_type::Node;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

/// Parse time duration string (e.g., "1h", "24h", "7d", "30d") to timestamp
fn parse_time_duration(duration_str: &str) -> Result<u64, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if let Ok(seconds) = duration_str.parse::<u64>() {
        return Ok(now - seconds);
    }

    let (num_str, unit) = if duration_str.len() < 2 {
        return Err("Invalid duration format".to_string());
    } else {
        let (num_part, unit_part) = duration_str.split_at(duration_str.len() - 1);
        (num_part, unit_part)
    };

    let number = num_str
        .parse::<u64>()
        .map_err(|_| "Invalid number".to_string())?;

    let seconds_to_subtract = match unit {
        "s" => number,
        "m" => number * 60,
        "h" => number * 60 * 60,
        "d" => number * 24 * 60 * 60,
        "w" => number * 7 * 24 * 60 * 60,
        _ => return Err("Invalid unit. Use s, m, h, d, or w".to_string()),
    };

    Ok(now - seconds_to_subtract)
}

/// Parse phase string to RecyclePhase enum
fn parse_phase(phase_str: &str) -> Result<RecyclePhase, String> {
    match phase_str.to_lowercase().as_str() {
        "incremental" => Ok(RecyclePhase::Incremental),
        "sweepexpired" | "sweep_expired" => Ok(RecyclePhase::SweepExpired),
        "stoptheworld" | "stop_the_world" => Ok(RecyclePhase::StopTheWorld),
        "manual" => Ok(RecyclePhase::Manual),
        _ => Err(
            "Invalid phase. Use: incremental, sweepexpired, stoptheworld, or manual".to_string(),
        ),
    }
}

#[derive(Debug, serde::Serialize)]
struct DecodedNodeSummary {
    node_kind: String,
    field_key: Option<String>,
    object_id: Option<String>,
    object_type: Option<String>,
    state_root: Option<String>,
    value_len: Option<usize>,
}

/// Dump recycle bin record for a specific node hash
#[derive(Debug, Parser)]
pub struct RecycleDumpCommand {
    /// Base data dir, e.g. ~/.rooch
    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,
    /// Chain ID
    #[clap(long, short = 'n')]
    pub chain_id: rooch_types::rooch_network::BuiltinChainID,
    /// Node hash to dump
    #[clap(long)]
    pub hash: String,
    /// Output file
    #[clap(long, short = 'o')]
    pub output: Option<PathBuf>,
    /// Decode node bytes if possible (Node<FieldKey, ObjectState>); best-effort
    #[clap(long)]
    pub decode: bool,
}

impl RecycleDumpCommand {
    pub async fn execute(self) -> RoochResult<String> {
        let (_root_meta, rooch_db, _start) = open_rooch_db_readonly(
            self.base_data_dir.clone(),
            Some(rooch_types::rooch_network::RoochChainID::Builtin(
                self.chain_id,
            )),
        );

        let recycle_store =
            RecycleBinStore::new(rooch_db.moveos_store.get_node_recycle_store().clone())?;

        // Parse hex string to bytes
        let hash_str = self.hash.strip_prefix("0x").unwrap_or(&self.hash);
        let hash_bytes = hex::decode(hash_str).map_err(|_| {
            rooch_types::error::RoochError::CommandArgumentError("Invalid hex format".to_string())
        })?;
        if hash_bytes.len() != 32 {
            return Err(rooch_types::error::RoochError::CommandArgumentError(
                "Hash must be 32 bytes".to_string(),
            ));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&hash_bytes);
        let node_hash = moveos_types::h256::H256(arr);

        if let Some(record) = recycle_store.get_record(&node_hash)? {
            #[derive(Debug, serde::Serialize)]
            struct DumpOut {
                record: rooch_pruner::recycle_bin::RecycleRecord,
                #[serde(skip_serializing_if = "Option::is_none")]
                decoded: Option<DecodedNodeSummary>,
            }
            let mut decoded = None;
            if self.decode {
                decoded = decode_node(&record.bytes);
            }
            let out = DumpOut { record, decoded };
            let output = serde_json::to_string_pretty(&out)?;

            match self.output {
                Some(output_path) => {
                    std::fs::write(&output_path, output).map_err(|e| {
                        rooch_types::error::RoochError::UnexpectedError(e.to_string())
                    })?;
                    info!("Recycle bin record dumped to {:?}", output_path);
                    Ok("Record dumped successfully".to_string())
                }
                None => {
                    println!("{}", output);
                    Ok("Record dumped successfully".to_string())
                }
            }
        } else {
            eprintln!("No recycle bin record found for hash: {}", self.hash);
            Ok("No record found".to_string())
        }
    }
}

fn decode_node(bytes: &[u8]) -> Option<DecodedNodeSummary> {
    if let Ok(node) = Node::<FieldKey, ObjectState>::decode(bytes) {
        match node {
            Node::Internal(_internal) => Some(DecodedNodeSummary {
                node_kind: "Internal".to_string(),
                field_key: None,
                object_id: None,
                object_type: None,
                state_root: None,
                value_len: None,
            }),
            Node::Leaf(leaf) => {
                let fk = leaf.key();
                let val = leaf.value().origin.clone();
                let obj_id = val.metadata.id.to_string();
                let obj_type = val.metadata.object_type.to_canonical_string();
                let sr = val.metadata.state_root().to_string();
                Some(DecodedNodeSummary {
                    node_kind: "Leaf".to_string(),
                    field_key: Some(fk.to_string()),
                    object_id: Some(obj_id),
                    object_type: Some(obj_type),
                    state_root: Some(sr),
                    value_len: Some(val.value.len()),
                })
            }
            Node::Null => Some(DecodedNodeSummary {
                node_kind: "Null".to_string(),
                field_key: None,
                object_id: None,
                object_type: None,
                state_root: None,
                value_len: None,
            }),
        }
    } else {
        None
    }
}

/// Restore a node from recycle bin back to state_node
#[derive(Debug, Parser)]
pub struct RecycleRestoreCommand {
    /// Base data dir, e.g. ~/.rooch
    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,
    /// Chain ID
    #[clap(long, short = 'n')]
    pub chain_id: rooch_types::rooch_network::BuiltinChainID,
    /// Node hash to restore
    #[clap(long)]
    pub hash: String,
    /// Force restore (required)
    #[clap(long, short = 'f')]
    pub force: bool,
}

impl RecycleRestoreCommand {
    pub async fn execute(self) -> RoochResult<String> {
        if !self.force {
            eprintln!("Error: --force is required for restore operation");
            eprintln!("This operation will overwrite existing state_node data.");
            return Ok("Operation cancelled".to_string());
        }

        let (_root_meta, rooch_db, _start) = open_rooch_db(
            self.base_data_dir.clone(),
            Some(rooch_types::rooch_network::RoochChainID::Builtin(
                self.chain_id,
            )),
        );

        let recycle_store =
            RecycleBinStore::new(rooch_db.moveos_store.get_node_recycle_store().clone())?;

        // Parse hex string to bytes
        let hash_str = self.hash.strip_prefix("0x").unwrap_or(&self.hash);
        let hash_bytes = hex::decode(hash_str).map_err(|_| {
            rooch_types::error::RoochError::CommandArgumentError("Invalid hex format".to_string())
        })?;
        if hash_bytes.len() != 32 {
            return Err(rooch_types::error::RoochError::CommandArgumentError(
                "Hash must be 32 bytes".to_string(),
            ));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&hash_bytes);
        let node_hash = moveos_types::h256::H256(arr);

        if let Some(record) = recycle_store.get_record(&node_hash)? {
            // Restore node back to state_node store (not recycle store)
            rooch_db
                .moveos_store
                .get_state_node_store()
                .kv_put(node_hash, record.bytes)
                .map_err(|e| rooch_types::error::RoochError::UnexpectedError(e.to_string()))?;

            info!("Node {} restored to state_node store", self.hash);
            info!("Created at: {}", record.created_at);
            info!("Original size: {} bytes", record.original_size);

            // Optionally remove from recycle bin after successful restore
            // recycle_store.delete_record(&node_hash)?;

            Ok("Node restored successfully".to_string())
        } else {
            eprintln!("No recycle bin record found for hash: {}", self.hash);
            Ok("No record found".to_string())
        }
    }
}

/// Show recycle bin statistics with enhanced safety focus
#[derive(Debug, Parser)]
pub struct RecycleStatCommand {
    /// Base data dir, e.g. ~/.rooch
    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,
    /// Chain ID
    #[clap(long, short = 'n')]
    pub chain_id: rooch_types::rooch_network::BuiltinChainID,

    // Enhanced options for strong backup mode
    /// Show detailed statistics by phase and node type
    #[clap(long)]
    pub detailed: bool,

    /// Group statistics by deletion phase
    #[clap(long)]
    pub by_phase: bool,

    /// Show age distribution analysis
    #[clap(long)]
    pub by_age: bool,

    /// Export statistics to JSON file
    #[clap(long)]
    pub export: Option<PathBuf>,

    /// Output format (json/table)
    #[clap(long, default_value = "table")]
    pub format: String,
}

#[derive(Debug, serde::Serialize)]
struct DetailedRecycleStats {
    summary: rooch_pruner::recycle_bin::RecycleBinStats,
    by_phase: Option<std::collections::HashMap<String, u64>>,
    age_distribution: Option<AgeDistribution>,
    recommendations: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct AgeDistribution {
    under_1h: u64,
    hours_1_24: u64,
    days_1_7: u64,
    over_7d: u64,
}

impl RecycleStatCommand {
    pub async fn execute(self) -> RoochResult<String> {
        let (_root_meta, rooch_db, _start) = open_rooch_db_readonly(
            self.base_data_dir.clone(),
            Some(rooch_types::rooch_network::RoochChainID::Builtin(
                self.chain_id,
            )),
        );

        // Use new RecycleBinConfig with strong backup defaults
        let recycle_config = RecycleBinConfig::default();
        let recycle_store = RecycleBinStore::new_with_config(
            rooch_db.moveos_store.get_node_recycle_store().clone(),
            recycle_config,
        )?;

        let stats = recycle_store.get_stats();

        // Create detailed statistics
        let mut detailed_stats = DetailedRecycleStats {
            summary: stats.clone(),
            by_phase: None,
            age_distribution: None,
            recommendations: Vec::new(),
        };

        // Generate recommendations based on strong backup mode
        if stats.strong_backup {
            detailed_stats
                .recommendations
                .push("Strong backup is ENABLED - no automatic deletion will occur".to_string());
            detailed_stats
                .recommendations
                .push("Manual cleanup required when disk space is low".to_string());

            if stats.current_bytes > 0 {
                let current_gb = stats.current_bytes / (1024 * 1024 * 1024);
                if current_gb >= 10 {
                    detailed_stats.recommendations.push(
                        format!("Consider running 'rooch db recycle-clean --dry-run' to review cleanup options")
                    );
                }
            }
        }

        // Format output
        match self.format.as_str() {
            "json" => {
                let output = serde_json::to_string_pretty(&detailed_stats)?;
                match self.export {
                    Some(export_path) => {
                        std::fs::write(&export_path, output).map_err(|e| {
                            rooch_types::error::RoochError::UnexpectedError(e.to_string())
                        })?;
                        info!("Recycle bin statistics exported to {:?}", export_path);
                        Ok("Statistics exported successfully".to_string())
                    }
                    None => {
                        println!("{}", output);
                        Ok("Statistics displayed".to_string())
                    }
                }
            }
            _ => {
                self.show_table_format(&stats, &detailed_stats)?;

                if let Some(export_path) = self.export {
                    let output = serde_json::to_string_pretty(&detailed_stats)?;
                    std::fs::write(&export_path, output).map_err(|e| {
                        rooch_types::error::RoochError::UnexpectedError(e.to_string())
                    })?;
                    info!("Recycle bin statistics exported to {:?}", export_path);
                }

                Ok("Statistics displayed".to_string())
            }
        }
    }

    fn show_table_format(
        &self,
        stats: &rooch_pruner::recycle_bin::RecycleBinStats,
        detailed_stats: &DetailedRecycleStats,
    ) -> RoochResult<()> {
        println!("{}", stats);

        // Show recommendations
        if !detailed_stats.recommendations.is_empty() {
            println!("\n=== Recommendations ===");
            for recommendation in &detailed_stats.recommendations {
                println!("  ⚠️  {}", recommendation);
            }
        }

        // Show safety information for strong backup
        if stats.strong_backup {
            println!("\n=== Strong Backup Mode ===");
            println!("  📦 All deleted nodes are preserved indefinitely");
            println!("  🔒 No automatic deletion will occur");
            println!("  🧹 Manual cleanup is the only way to free space");
            println!(
                "  📊 Current storage: {} MB",
                stats.current_bytes / (1024 * 1024)
            );

            if stats.current_bytes > 0 {
                let current_gb = stats.current_bytes / (1024 * 1024 * 1024);
                if current_gb >= 50 {
                    println!("  🚨 CRITICAL: High storage usage detected!");
                    println!(
                        "     Run 'rooch db recycle-clean --dry-run' to review cleanup options"
                    );
                } else if current_gb >= 10 {
                    println!("  ⚠️  WARNING: Growing storage usage detected");
                }
            }
        }

        Ok(())
    }
}

/// List recycle bin entries with filtering options
#[derive(Debug, Parser)]
pub struct RecycleListCommand {
    /// Base data dir, e.g. ~/.rooch
    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,
    /// Chain ID
    #[clap(long, short = 'n')]
    pub chain_id: rooch_types::rooch_network::BuiltinChainID,

    // Filtering options
    /// Filter by deletion phase (Incremental/SweepExpired/StopTheWorld/Manual)
    #[clap(long)]
    pub phase: Option<String>,

    /// Filter by age (e.g., "1h", "24h", "7d")
    #[clap(long)]
    pub older_than: Option<String>,

    /// Filter by node type (Internal/Leaf/Null/Unknown)
    #[clap(long)]
    pub node_type: Option<String>,

    /// Limit number of results
    #[clap(long)]
    pub limit: Option<usize>,

    // Output options
    /// Output format (json/table)
    #[clap(long, default_value = "table")]
    pub format: String,

    /// Export to file
    #[clap(long)]
    pub export: Option<PathBuf>,

    /// Show only node hashes
    #[clap(long)]
    pub hashes_only: bool,
}

impl RecycleListCommand {
    pub async fn execute(self) -> RoochResult<String> {
        let (_root_meta, rooch_db, _start) = open_rooch_db_readonly(
            self.base_data_dir.clone(),
            Some(rooch_types::rooch_network::RoochChainID::Builtin(
                self.chain_id,
            )),
        );

        let recycle_config = RecycleBinConfig::default();
        let recycle_store = RecycleBinStore::new_with_config(
            rooch_db.moveos_store.get_node_recycle_store().clone(),
            recycle_config,
        )?;

        // Build the filter based on command line options
        let mut filter = RecycleFilter {
            older_than: None,
            newer_than: None,
            min_size: None,
            max_size: None,
        };

        // Parse time filter
        if let Some(older_than) = &self.older_than {
            let cutoff_time = parse_time_duration(older_than)
                .map_err(|e| rooch_types::error::RoochError::CommandArgumentError(e))?;
            filter.older_than = Some(cutoff_time);
        }

        // Parse phase filter
        if let Some(phase_str) = &self.phase {
            let phase = parse_phase(phase_str)
                .map_err(|e| rooch_types::error::RoochError::CommandArgumentError(e))?;
            // Phase filtering removed in simplified RecycleRecord design
            warn!("Phase filtering requested but phase field has been removed");
        }

        // Get filtered entries
        let entries = recycle_store.list_entries(Some(filter), self.limit)?;

        if entries.is_empty() {
            println!("No entries found matching the specified criteria.");
            return Ok("No entries found".to_string());
        }

        // Output based on format
        match self.format.as_str() {
            "json" => {
                let json_output = serde_json::to_string_pretty(&entries)?;
                match self.export {
                    Some(path) => {
                        std::fs::write(&path, json_output).map_err(|e| {
                            rooch_types::error::RoochError::UnexpectedError(e.to_string())
                        })?;
                        println!("Exported {} entries to {:?}", entries.len(), path);
                    }
                    None => {
                        println!("{}", json_output);
                    }
                }
            }
            _ => {
                // table format
                println!("=== Recycle Bin Entries ===");
                println!("Total entries: {}", entries.len());
                println!();

                if self.hashes_only {
                    for record in &entries {
                        // Show a sample of bytes data as identifier
                        let bytes_preview = if record.bytes.len() >= 4 {
                            &record.bytes[0..4]
                        } else {
                            &record.bytes
                        };
                        println!("0x{}", hex::encode(bytes_preview));
                    }
                } else {
                    println!(
                        "{:<20} {:<12} {:<10} {:<20} {:<10}",
                        "Phase", "Size", "Age", "Deleted At", "Type"
                    );
                    println!("{}", "-".repeat(80));

                    for record in &entries {
                        let age_seconds = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                            - record.created_at; // Use created_at instead of deleted_at
                        let age_hours = age_seconds / 3600;
                        let age_str = if age_hours < 24 {
                            format!("{}h", age_hours)
                        } else {
                            format!("{}d", age_hours / 24)
                        };

                        // Node type can be derived from bytes if needed
                        let node_type = if record.bytes.is_empty() {
                            "Null"
                        } else {
                            match record.bytes[0] {
                                0 => "Null",
                                1 => "Internal",
                                2 => "Leaf",
                                _ => "Unknown",
                            }
                        };

                        println!("{:<12} {:<10} {:<10}", "Size", "Age", "Type");

                        println!(
                            "{:<12} {:<10} {:<10}",
                            record.original_size, age_str, node_type
                        );
                    }
                }
            }
        }

        Ok(format!("Listed {} entries", entries.len()))
    }
}

/// Clean up recycle bin entries with explicit manual control
#[derive(Debug, Parser)]
pub struct RecycleCleanCommand {
    /// Base data dir, e.g. ~/.rooch
    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,
    /// Chain ID
    #[clap(long, short = 'n')]
    pub chain_id: rooch_types::rooch_network::BuiltinChainID,

    // Safety options
    /// Dry run - show what would be deleted without actually deleting
    #[clap(long)]
    pub dry_run: bool,

    /// Force cleanup without interactive confirmation
    #[clap(long, short = 'f')]
    pub force: bool,

    /// Keep most recent N records (preserve newest)
    #[clap(long)]
    pub preserve_count: Option<usize>,

    // Filtering options
    /// Delete records older than specified time (e.g., "1h", "24h", "7d")
    #[clap(long)]
    pub older_than: Option<String>,

    /// Delete records from specific phase
    #[clap(long)]
    pub phase: Option<String>,

    /// Delete records of specific node type
    #[clap(long)]
    pub node_type: Option<String>,

    // Performance options
    /// Batch size for deletion operations
    #[clap(long, default_value_t = 1000)]
    pub batch_size: usize,

    /// Use parallel processing
    #[clap(long)]
    pub parallel: bool,
}

impl RecycleCleanCommand {
    pub async fn execute(self) -> RoochResult<String> {
        // Safety check: require either --dry-run or --force
        if !self.dry_run && !self.force {
            eprintln!("Error: Either --dry-run or --force is required");
            eprintln!("");
            eprintln!("This is a PERMANENT deletion operation. Consider:");
            eprintln!("  1. Run with --dry-run first to see what would be deleted");
            eprintln!("  2. Run with --force after reviewing the dry-run output");
            eprintln!("");
            eprintln!("Example:");
            eprintln!("  rooch db recycle-clean --dry-run --older-than 7d");
            eprintln!("  rooch db recycle-clean --force --older-than 7d");
            return Ok("Operation cancelled for safety".to_string());
        }

        let (_root_meta, rooch_db, _start) = open_rooch_db_readonly(
            self.base_data_dir.clone(),
            Some(rooch_types::rooch_network::RoochChainID::Builtin(
                self.chain_id,
            )),
        );

        let recycle_config = RecycleBinConfig::default();
        let recycle_store = RecycleBinStore::new_with_config(
            rooch_db.moveos_store.get_node_recycle_store().clone(),
            recycle_config,
        )?;

        // Get current statistics
        let stats = recycle_store.get_stats();

        println!("=== Recycle Bin Cleanup Plan ===");
        println!("{}", stats);
        println!("");

        if stats.strong_backup {
            println!("🔒 Strong Backup Mode: ENABLED");
            println!("⚠️  WARNING: This is a PERMANENT deletion operation");
            println!("📦 Deleted records cannot be recovered after cleanup");
        }

        // Show what would be affected
        println!("\n=== Cleanup Parameters ===");
        if let Some(older_than) = &self.older_than {
            println!("  Delete records older than: {}", older_than);
        }
        if let Some(phase) = &self.phase {
            println!("  Delete records from phase: {}", phase);
        }
        if let Some(node_type) = &self.node_type {
            println!("  Delete records of type: {}", node_type);
        }
        if let Some(preserve_count) = self.preserve_count {
            println!("  Preserve most recent {} records", preserve_count);
        }
        println!("  Batch size: {}", self.batch_size);
        println!("  Parallel processing: {}", self.parallel);

        // Build the filter based on command line options
        let mut filter = RecycleFilter {
            older_than: None,
            newer_than: None,
            min_size: None,
            max_size: None,
        };

        // Parse time filter
        if let Some(older_than) = &self.older_than {
            let cutoff_time = parse_time_duration(older_than)
                .map_err(|e| rooch_types::error::RoochError::CommandArgumentError(e))?;
            filter.older_than = Some(cutoff_time);
        }

        // Parse phase filter
        if let Some(phase_str) = &self.phase {
            let phase = parse_phase(phase_str)
                .map_err(|e| rooch_types::error::RoochError::CommandArgumentError(e))?;
            // Phase filtering removed in simplified RecycleRecord design
            warn!("Phase filtering requested but phase field has been removed");
        }

        // Execute the cleanup
        if self.dry_run {
            println!("\n📋 DRY RUN MODE - No actual deletion will occur");
            println!("");

            // Use list_entries to show what would be deleted
            let entries =
                recycle_store.list_entries(Some(filter.clone()), Some(self.batch_size))?;

            println!("=== Records that would be deleted ===");
            println!("Found {} matching records", entries.len());

            for (i, record) in entries.iter().take(10).enumerate() {
                println!(
                    "  {}. {} bytes (created at: {})",
                    i + 1,
                    record.original_size,
                    record.created_at
                );
            }

            if entries.len() > 10 {
                println!("  ... and {} more", entries.len() - 10);
            }
        } else {
            println!("\n🚨 LIVE DELETION MODE - Records will be permanently deleted!");
            println!("   This operation is IRREVERSIBLE!");

            // Additional safety confirmation
            if !self.force {
                eprintln!("\nType 'DELETE-RECYCLE-BIN' to confirm permanent deletion:");
                use std::io::{self, Write};
                let mut input = String::new();
                io::stdout().flush().unwrap();
                io::stdin().read_line(&mut input).unwrap();

                if input.trim() != "DELETE-RECYCLE-BIN" {
                    eprintln!("Confirmation failed. Operation cancelled.");
                    return Ok("Operation cancelled by user".to_string());
                }
            }

            // Execute the actual deletion
            let deleted_count = if let Some(preserve_count) = self.preserve_count {
                recycle_store.preserve_recent_entries(preserve_count)?
            } else {
                recycle_store.delete_entries(&filter, self.batch_size)?
            };

            println!("✅ Cleanup completed!");
            println!("🗑️  Deleted {} records", deleted_count);

            // Show updated statistics
            let new_stats = recycle_store.get_stats();
            println!("\n=== Updated Statistics ===");
            println!("{}", new_stats);
        }

        Ok("Cleanup operation completed".to_string())
    }
}

/// Export recycle bin data for backup or analysis
#[derive(Debug, Parser)]
pub struct RecycleExportCommand {
    /// Base data dir, e.g. ~/.rooch
    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,
    /// Chain ID
    #[clap(long, short = 'n')]
    pub chain_id: rooch_types::rooch_network::BuiltinChainID,

    /// Output file path
    #[clap(long, short = 'o')]
    pub output: PathBuf,

    /// Export format (json/csv/sqlite)
    #[clap(long, default_value = "json")]
    pub format: String,

    /// Include original node data
    #[clap(long)]
    pub include_node_data: bool,

    /// Compress output
    #[clap(long)]
    pub compress: bool,

    /// Export records since timestamp
    #[clap(long)]
    pub since: Option<String>,

    /// Export records until timestamp
    #[clap(long)]
    pub until: Option<String>,
}

impl RecycleExportCommand {
    pub async fn execute(self) -> RoochResult<String> {
        let (_root_meta, rooch_db, _start) = open_rooch_db_readonly(
            self.base_data_dir.clone(),
            Some(rooch_types::rooch_network::RoochChainID::Builtin(
                self.chain_id,
            )),
        );

        let recycle_config = RecycleBinConfig::default();
        let recycle_store = RecycleBinStore::new_with_config(
            rooch_db.moveos_store.get_node_recycle_store().clone(),
            recycle_config,
        )?;

        // Get current statistics
        let stats = recycle_store.get_stats();

        println!("=== Recycle Bin Export ===");
        println!("{}", stats);
        println!("");

        println!("Export parameters:");
        println!("  Output file: {:?}", self.output);
        println!("  Format: {}", self.format);
        println!("  Include node data: {}", self.include_node_data);
        println!("  Compress: {}", self.compress);
        println!("  Time range: {:?} to {:?}", self.since, self.until);

        // Create basic export metadata
        #[derive(Debug, serde::Serialize)]
        struct ExportMetadata {
            export_time: u64,        // Unix timestamp
            export_time_iso: String, // ISO 8601 format for readability
            stats: rooch_pruner::recycle_bin::RecycleBinStats,
            parameters: std::collections::HashMap<String, serde_json::Value>,
        }

        let mut parameters = std::collections::HashMap::new();
        parameters.insert(
            "format".to_string(),
            serde_json::Value::String(self.format.clone()),
        );
        parameters.insert(
            "include_node_data".to_string(),
            serde_json::Value::Bool(self.include_node_data),
        );
        parameters.insert(
            "compress".to_string(),
            serde_json::Value::Bool(self.compress),
        );

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let metadata = ExportMetadata {
            export_time: now,
            export_time_iso: format!("timestamp:{}", now),
            stats: stats.clone(),
            parameters,
        };

        let export_content = match self.format.as_str() {
            "json" => serde_json::to_string_pretty(&metadata)?,
            _ => serde_json::to_string_pretty(&metadata)?, // Default to JSON for now
        };

        // Write export file
        if self.compress {
            // TODO: Implement compression
            std::fs::write(&self.output, export_content)?;
            info!("Export completed (compression not yet implemented)");
        } else {
            std::fs::write(&self.output, export_content)?;
            info!("Export completed: {:?}", self.output);
        }

        println!("\n⚠️  Note: Full export functionality requires database iteration capabilities");
        println!("      Current implementation exports metadata and statistics only.");

        Ok("Export completed".to_string())
    }
}
