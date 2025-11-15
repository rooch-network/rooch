// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use anyhow::{Context, Result};
use async_trait::async_trait;
use clap::Parser;
use rooch_types::error::RoochResult;
use std::path::PathBuf;

/// Analyze RocksDB statistics and disk usage
#[derive(Parser, Debug)]
pub struct RocksDBStatsCommand {
    /// Path to RocksDB directory
    #[clap(long, default_value = "~/.rooch/local/roochdb/store")]
    pub db_path: String,

    /// List all column families
    #[clap(long)]
    pub list_cf: bool,
}

#[async_trait]
impl CommandAction<String> for RocksDBStatsCommand {
    async fn execute(self) -> RoochResult<String> {
        let result = self
            .execute_impl()
            .map_err(rooch_types::error::RoochError::from)?;
        Ok(result)
    }
}

impl RocksDBStatsCommand {
    fn execute_impl(self) -> Result<String> {
        // Expand tilde manually
        let path_str = if self.db_path.starts_with("~") {
            let home = std::env::var("HOME").context("HOME environment variable not set")?;
            self.db_path.replacen("~", &home, 1)
        } else {
            self.db_path.clone()
        };

        let db_path = PathBuf::from(&path_str);

        if !db_path.exists() {
            return Err(anyhow::anyhow!(
                "Database path does not exist: {:?}",
                db_path
            ));
        }

        println!("=== RocksDB Statistics ===");
        println!("Path: {}\n", db_path.display());

        // Get directory size
        let total_size = get_dir_size(&db_path)?;
        println!(
            "Total directory size: {:.2} GB ({} bytes)\n",
            total_size as f64 / 1e9,
            total_size
        );

        // Discover existing column families from the DB on disk to avoid errors on unknown CFs
        let existing_names: Vec<String> = raw_store::rocks::RocksDB::list_cf(&db_path)
            .unwrap_or_else(|_| vec!["default".to_string()]);
        let existing_set: std::collections::HashSet<&str> =
            existing_names.iter().map(|s| s.as_str()).collect();
        // Build open CF list from known CFs that actually exist on disk
        let mut open_cfs: Vec<&'static str> = Vec::new();
        if existing_set.contains("default") {
            open_cfs.push("default");
        }
        let mut known_cfs = moveos_store::StoreMeta::get_column_family_names().to_vec();
        known_cfs.append(&mut rooch_store::StoreMeta::get_column_family_names().to_vec());
        for cf in known_cfs {
            if existing_set.contains(cf) {
                open_cfs.push(cf);
            }
        }

        // Also collect known CFs from StoreMeta (for reference only)
        let mut known_cfs = moveos_store::StoreMeta::get_column_family_names().to_vec();
        known_cfs.append(&mut rooch_store::StoreMeta::get_column_family_names().to_vec());

        if self.list_cf {
            println!("Existing Column Families:");
            for cf in &existing_names {
                println!("  - {}", cf);
            }
            return Ok(format!(
                "Found {} existing column families",
                existing_names.len()
            ));
        }

        println!("Existing Column Families: {}\n", existing_names.len());
        for cf_name in &existing_names {
            println!("  - {}", cf_name);
        }

        // Open DB in read-only mode
        let config = moveos_config::store_config::RocksdbConfig::default();
        // Open DB in read-only mode with only the existing CFs to avoid "CF not found" errors
        // Convert Vec<String> -> Vec<&str> for API
        let db = raw_store::rocks::RocksDB::new_readonly(&db_path, open_cfs.clone(), config)
            .context("Failed to open RocksDB. Make sure the rooch service is stopped.")?;

        println!("\n=== Column Family Statistics ===\n");

        // Get properties for each column family via internal db access
        let raw_db = db.inner();

        for cf_name in &open_cfs {
            if let Some(cf) = raw_db.cf_handle(cf_name) {
                println!("--- {} ---", cf_name);

                // Integer properties
                let props = vec![
                    ("rocksdb.total-sst-files-size", "Total SST size"),
                    ("rocksdb.live-sst-files-size", "Live SST size"),
                    ("rocksdb.estimate-live-data-size", "Est. live data"),
                    ("rocksdb.num-files-at-level0", "L0 files"),
                    ("rocksdb.num-snapshots", "Snapshots"),
                    (
                        "rocksdb.estimate-pending-compaction-bytes",
                        "Pending compact",
                    ),
                    // BlobDB integer metrics (may be None if BlobDB disabled)
                    ("rocksdb.num-blob-files", "Blob files"),
                    ("rocksdb.total-blob-file-size", "Total blob size"),
                    ("rocksdb.live-blob-file-size", "Live blob size"),
                    ("rocksdb.garbage-blob-file-size", "Garbage blob size"),
                ];

                for (prop, label) in props {
                    if let Ok(Some(value)) = raw_db.property_int_value_cf(&cf, prop) {
                        if prop.contains("size") || prop.contains("bytes") {
                            println!(
                                "  {}: {:.2} GB ({} bytes)",
                                label,
                                value as f64 / 1e9,
                                value
                            );
                        } else {
                            println!("  {}: {}", label, value);
                        }
                    }
                }

                // String properties for BlobDB
                let str_props = vec![
                    (
                        "rocksdb.blobdb.is-garbage-collection-enabled",
                        "Blob GC enabled",
                    ),
                    ("rocksdb.blobdb.stats", "BlobDB stats"),
                ];

                for (prop, label) in str_props {
                    if let Ok(Some(value)) = raw_db.property_value_cf(&cf, prop) {
                        let v = value.trim();
                        if !v.is_empty() {
                            println!("  {}: {}", label, v);
                        }
                    }
                }

                println!();
            }
        }

        Ok("Statistics complete".to_string())
    }
}

fn get_dir_size(path: &PathBuf) -> Result<u64> {
    let mut total = 0u64;

    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;

            if metadata.is_file() {
                total += metadata.len();
            } else if metadata.is_dir() {
                total += get_dir_size(&entry.path())?;
            }
        }
    }

    Ok(total)
}
