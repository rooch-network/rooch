// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::utils::{open_rooch_db, open_rooch_db_readonly, read_line};
use clap::Parser;
use moveos_types::state::{FieldKey, ObjectState};
use raw_store::CodecKVStore;
use rooch_pruner::recycle_bin::{RecycleBinConfig, RecycleBinStore, RecycleFilter};
use rooch_types::error::RoochResult;
use serde_json;
use smt::jellyfish_merkle::node_type::Node;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

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

// Phase parsing removed - no longer needed with simplified RecycleRecord structure

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
                #[serde(flatten)]
                record_fields: RecycleRecordFields,
                #[serde(skip_serializing_if = "Option::is_none")]
                decoded: Option<DecodedNodeSummary>,
            }

            #[derive(Debug, serde::Serialize)]
            struct RecycleRecordFields {
                original_size: usize,
                created_at: u64,
                #[serde(with = "hex_bytes")]
                bytes: Vec<u8>,
            }

            // Module for hex serialization of Vec<u8>
            mod hex_bytes {
                use hex;
                use serde::Serializer;

                pub fn serialize<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    let hex_string = format!("0x{}", hex::encode(bytes));
                    serializer.serialize_str(&hex_string)
                }
            }

            let mut decoded = None;
            if self.decode {
                decoded = decode_node(&record.bytes);
            }

            let out = DumpOut {
                record_fields: RecycleRecordFields {
                    original_size: record.original_size,
                    created_at: record.created_at,
                    bytes: record.bytes,
                },
                decoded,
            };
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
            tracing::warn!("No recycle bin record found for hash: {}", self.hash);
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
            tracing::error!("Error: --force is required for restore operation");
            tracing::error!("This operation will overwrite existing state_node data.");
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
            tracing::warn!("No recycle bin record found for hash: {}", self.hash);
            Ok("No record found".to_string())
        }
    }
}

// RecycleStatCommand removed - output unreliable data from memory counters instead of actual database
// Use 'rooch db recycle-list' to see actual entries

/// List recycle bin entries with filtering options
#[derive(Debug, Parser)]
pub struct RecycleListCommand {
    /// Base data dir, e.g. ~/.rooch
    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,
    /// Chain ID
    #[clap(long, short = 'n')]
    pub chain_id: rooch_types::rooch_network::BuiltinChainID,

    /// Filter by age (e.g., "1h", "24h", "7d")
    #[clap(long)]
    pub older_than: Option<String>,

    /// Page size for cursor-based pagination (default: 20)
    #[clap(long, default_value_t = 20)]
    pub page_size: usize,

    /// Cursor for pagination (returned from previous page)
    #[clap(long)]
    pub cursor: Option<String>,

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
        // Display structure for table format
        #[derive(Debug)]
        struct TableDisplayEntry<'a> {
            hash: String,
            record: &'a rooch_pruner::recycle_bin::RecycleRecord,
        }

        // JSON output structure
        #[derive(Debug, serde::Serialize)]
        struct JsonOutputEntry {
            hash: String,
            record: rooch_pruner::recycle_bin::RecycleRecord,
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
                .map_err(rooch_types::error::RoochError::CommandArgumentError)?;
            filter.older_than = Some(cutoff_time);
        }

        // Phase filtering removed - RecycleRecord no longer has phase field

        // Validate page size
        if self.page_size == 0 || self.page_size > 1000 {
            return Err(rooch_types::error::RoochError::CommandArgumentError(
                "Page size must be between 1 and 1000".to_string(),
            ));
        }

        // Parse cursor if provided
        let cursor = if let Some(cursor_str) = &self.cursor {
            let clean_str = cursor_str.strip_prefix("0x").unwrap_or(cursor_str);
            let bytes = hex::decode(clean_str).map_err(|_| {
                rooch_types::error::RoochError::CommandArgumentError(
                    "Invalid cursor format".to_string(),
                )
            })?;
            if bytes.len() != 32 {
                return Err(rooch_types::error::RoochError::CommandArgumentError(
                    "Cursor must be 32 bytes".to_string(),
                ));
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            Some(moveos_types::h256::H256(arr))
        } else {
            None
        };

        // Use cursor-based pagination if page_size is specified
        let paginated_result =
            recycle_store.list_entries_cursor(Some(filter), cursor, self.page_size)?;

        if paginated_result.entries.is_empty() {
            println!("No entries found matching the specified criteria.");
            return Ok("No entries found".to_string());
        }

        // Prepare display data with hash information
        let display_entries: Vec<TableDisplayEntry> = paginated_result
            .entries
            .iter()
            .map(|(hash, record)| TableDisplayEntry {
                hash: format!("0x{}", hex::encode(hash.as_bytes())),
                record,
            })
            .collect();

        // Output based on format
        match self.format.as_str() {
            "json" => {
                let json_entries: Vec<JsonOutputEntry> = paginated_result
                    .entries
                    .iter()
                    .map(|(hash, record)| JsonOutputEntry {
                        hash: format!("0x{}", hex::encode(hash.as_bytes())),
                        record: record.clone(),
                    })
                    .collect();

                let json_output = serde_json::json!({
                    "entries": json_entries,
                    "pagination": {
                        "next_cursor": paginated_result.next_cursor,
                        "has_more": paginated_result.has_more,
                        "page_size": paginated_result.page_size,
                        "entries_on_page": paginated_result.entries.len()
                    }
                });

                let json_string = serde_json::to_string_pretty(&json_output)?;
                match self.export {
                    Some(ref path) => {
                        std::fs::write(path, json_string).map_err(|e| {
                            rooch_types::error::RoochError::UnexpectedError(e.to_string())
                        })?;
                        println!(
                            "Exported {} entries to {:?}",
                            paginated_result.entries.len(),
                            path
                        );
                    }
                    None => {
                        println!("{}", json_string);
                    }
                }
            }
            _ => {
                // table format
                println!("=== Recycle Bin Entries ===");
                println!();

                if self.hashes_only {
                    for entry in &display_entries {
                        // Show the real hash value
                        println!("{}", entry.hash);
                    }
                } else {
                    // New table format with Hash column (66 chars for hash + other columns)
                    println!("{:<66} {:<10} {:<6} {:<10}", "Hash", "Size", "Age", "Type");
                    println!("{}", "-".repeat(94));

                    for entry in &display_entries {
                        let age_seconds = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                            - entry.record.created_at; // Use created_at instead of deleted_at
                        let age_str = if age_seconds < 3600 {
                            format!("{}m", age_seconds / 60)
                        } else if age_seconds < 86400 {
                            format!("{}h", age_seconds / 3600)
                        } else {
                            format!("{}d", age_seconds / 86400)
                        };

                        // Node type can be derived from bytes if needed
                        let node_type = if entry.record.bytes.is_empty() {
                            "Null"
                        } else {
                            match entry.record.bytes[0] {
                                0 => "Null",
                                1 => "Internal",
                                2 => "Leaf",
                                _ => "Unknown",
                            }
                        };

                        println!(
                            "{:<66} {:<10} {:<6} {:<10}",
                            entry.hash, entry.record.original_size, age_str, node_type
                        );
                    }
                }
            }
        }

        // Add pagination information at the bottom
        self.display_pagination_info(&paginated_result)?;

        Ok(format!("Listed {} entries", paginated_result.entries.len()))
    }

    /// Display pagination information for cursor-based pagination
    fn display_pagination_info(
        &self,
        paginated_result: &rooch_pruner::recycle_bin::PaginatedListResult,
    ) -> RoochResult<()> {
        println!();
        println!("--- Pagination Info ---");
        println!("Page size: {}", paginated_result.page_size);
        println!("Entries returned: {}", paginated_result.entries.len());
        println!("Has more: {}", paginated_result.has_more);

        if let Some(next_cursor) = &paginated_result.next_cursor {
            println!("Next cursor: {}", next_cursor);
        }

        Ok(())
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

    // Filtering options
    /// Delete records older than specified time (e.g., "1h", "24h", "7d")
    #[clap(long)]
    pub older_than: Option<String>,

    // Performance options
    /// Batch size for deletion operations
    #[clap(long, default_value_t = 1000)]
    pub batch_size: usize,
}

impl RecycleCleanCommand {
    pub async fn execute(self) -> RoochResult<String> {
        // Safety check: require either --dry-run or --force
        if !self.dry_run && !self.force {
            tracing::error!("Error: Either --dry-run or --force is required");
            tracing::error!("This is a PERMANENT deletion operation. Consider:");
            tracing::error!("  1. Run with --dry-run first to see what would be deleted");
            tracing::error!("  2. Run with --force after reviewing the dry-run output");
            tracing::error!("Example:");
            tracing::error!("  rooch db recycle-clean --dry-run --older-than 7d");
            tracing::error!("  rooch db recycle-clean --force --older-than 7d");
            return Ok("Operation cancelled for safety".to_string());
        }

        let (_root_meta, rooch_db, _start) = open_rooch_db(
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

        tracing::info!("=== Recycle Bin Cleanup Plan ===");
        tracing::info!("{}", stats);

        if stats.strong_backup {
            tracing::info!("🔒 Strong Backup Mode: ENABLED");
            tracing::warn!("⚠️  WARNING: This is a PERMANENT deletion operation");
            tracing::warn!("📦 Deleted records cannot be recovered after cleanup");
        }

        // Show what would be affected
        tracing::info!("=== Cleanup Parameters ===");
        if let Some(older_than) = &self.older_than {
            tracing::info!("  Delete records older than: {}", older_than);
        }
        // phase and node_type filtering removed - RecycleRecord simplified
        // preserve_count functionality removed - not implemented
        tracing::info!("  Batch size: {}", self.batch_size);

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
                .map_err(rooch_types::error::RoochError::CommandArgumentError)?;
            filter.older_than = Some(cutoff_time);
        }

        // Phase filtering removed - RecycleRecord no longer has phase field

        // Execute the cleanup
        if self.dry_run {
            tracing::info!("📋 DRY RUN MODE - No actual deletion will occur");

            // Use list_entries to show what would be deleted
            let entries =
                recycle_store.list_entries(Some(filter.clone()), Some(self.batch_size))?;

            tracing::info!("=== Records that would be deleted ===");
            tracing::info!("Found {} matching records", entries.len());

            for (i, record) in entries.iter().take(10).enumerate() {
                tracing::info!(
                    "  {}. {} bytes (created at: {})",
                    i + 1,
                    record.original_size,
                    record.created_at
                );
            }

            if entries.len() > 10 {
                tracing::info!("  ... and {} more", entries.len() - 10);
            }
        } else {
            tracing::warn!("🚨 LIVE DELETION MODE - Records will be permanently deleted!");
            tracing::warn!("   This operation is IRREVERSIBLE!");

            // Additional safety confirmation
            if !self.force {
                tracing::warn!("This operation will PERMANENTLY delete all matching recycle bin entries.");
                tracing::warn!("Type 'DELETE-RECYCLE-BIN' to confirm permanent deletion:");
                
                match read_line() {
                    Ok(input) if input.trim() == "DELETE-RECYCLE-BIN" => {
                        tracing::info!("Confirmation received, proceeding with deletion.");
                    }
                    _ => {
                        tracing::info!("Confirmation failed. Operation cancelled by user.");
                        return Ok("Operation cancelled by user".to_string());
                    }
                }
            }

            // Execute the actual deletion
            let deleted_count = recycle_store.delete_entries(&filter, self.batch_size)?;

            tracing::info!("✅ Cleanup completed!");
            tracing::info!("🗑️  Deleted {} records", deleted_count);

            // Show updated statistics
            let new_stats = recycle_store.get_stats();
            tracing::info!("=== Updated Statistics ===");
            tracing::info!("{}", new_stats);
        }

        Ok("Cleanup operation completed".to_string())
    }
}
