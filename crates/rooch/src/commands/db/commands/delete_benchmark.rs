// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use crate::commands::db::commands::expand_tilde;
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use clap::Parser;
use moveos_config::store_config::RocksdbConfig;
use moveos_store::STATE_NODE_COLUMN_FAMILY_NAME;
use rooch_types::error::RoochResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Benchmark and compare RocksDB state before/after deletion operations
#[derive(Parser, Debug)]
pub struct DeleteBenchmarkCommand {
    /// Path to RocksDB directory
    #[clap(long, default_value = "~/.rooch/local/roochdb/store")]
    pub db_path: String,

    /// Action: snapshot (save state) or compare (compare with saved snapshot)
    #[clap(subcommand)]
    pub action: BenchmarkAction,
}

#[derive(Parser, Debug)]
pub enum BenchmarkAction {
    /// Take a snapshot of current DB state
    Snapshot {
        /// Name for this snapshot
        #[clap(long, default_value = "before")]
        name: String,
    },
    /// Compare current state with a saved snapshot
    Compare {
        /// Name of the snapshot to compare with
        #[clap(long, default_value = "before")]
        name: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct DbSnapshot {
    timestamp: String,
    total_sst_size: u64,
    live_sst_size: u64,
    est_live_data: u64,
    l0_files: u64,
    pending_compact: u64,
    blob_files: u64,
    total_blob_size: u64,
    live_blob_size: u64,
    garbage_blob_size: u64,
}

#[async_trait]
impl CommandAction<String> for DeleteBenchmarkCommand {
    async fn execute(self) -> RoochResult<String> {
        let result = self
            .execute_impl()
            .map_err(rooch_types::error::RoochError::from)?;
        Ok(result)
    }
}

impl DeleteBenchmarkCommand {
    fn execute_impl(self) -> Result<String> {
        let path_str = expand_tilde(&self.db_path)?;
        let db_path = PathBuf::from(&path_str);

        if !db_path.exists() {
            return Err(anyhow!("DB path not found: {:?}", db_path));
        }

        match self.action {
            BenchmarkAction::Snapshot { ref name } => self.take_snapshot(&db_path, name),
            BenchmarkAction::Compare { ref name } => self.compare_snapshot(&db_path, name),
        }
    }

    fn take_snapshot(&self, db_path: &PathBuf, name: &str) -> Result<String> {
        let snapshot = self.collect_stats(db_path)?;

        // Save snapshot to file
        let snapshot_path = db_path.join(format!("snapshot_{}.json", name));
        let json = serde_json::to_string_pretty(&snapshot)?;
        std::fs::write(&snapshot_path, json)?;

        let mut out = String::new();
        out.push_str(&format!("=== Snapshot '{}' Saved ===\n", name));
        out.push_str(&format!("Path: {:?}\n\n", snapshot_path));
        out.push_str(&self.format_snapshot(&snapshot));
        out.push_str(&format!("\n💾 Snapshot saved to: {:?}\n", snapshot_path));
        out.push_str("\nNext steps:\n");
        out.push_str("  1. Run deletion operations (pruner, manual delete, etc.)\n");
        out.push_str(&format!(
            "  2. Run: rooch db delete-benchmark compare --name {}\n",
            name
        ));

        Ok(out)
    }

    fn compare_snapshot(&self, db_path: &PathBuf, name: &str) -> Result<String> {
        let snapshot_path = db_path.join(format!("snapshot_{}.json", name));

        if !snapshot_path.exists() {
            return Err(anyhow!(
                "Snapshot '{}' not found. Run 'snapshot' first.",
                name
            ));
        }

        let before_json = std::fs::read_to_string(&snapshot_path)?;
        let before: DbSnapshot = serde_json::from_str(&before_json)?;
        let after = self.collect_stats(db_path)?;

        let mut out = String::new();
        out.push_str("=== Deletion Benchmark Results ===\n\n");

        // Before state
        out.push_str(&format!(
            "📷 Snapshot '{}' (taken at {})\n",
            name, before.timestamp
        ));
        out.push_str(&self.format_snapshot(&before));

        // After state
        out.push_str(&format!("\n📷 Current State ({})\n", after.timestamp));
        out.push_str(&self.format_snapshot(&after));

        // Comparison
        out.push_str("\n=== Changes ===\n");
        out.push_str(&self.format_diff("Total SST", before.total_sst_size, after.total_sst_size));
        out.push_str(&self.format_diff("Live SST", before.live_sst_size, after.live_sst_size));
        out.push_str(&self.format_diff(
            "Est. Live Data",
            before.est_live_data,
            after.est_live_data,
        ));
        out.push_str(&self.format_diff(
            "Pending Compact",
            before.pending_compact,
            after.pending_compact,
        ));
        out.push_str(&self.format_diff(
            "Total Blob",
            before.total_blob_size,
            after.total_blob_size,
        ));
        out.push_str(&self.format_diff("Live Blob", before.live_blob_size, after.live_blob_size));

        // Summary
        let total_before = before.total_sst_size + before.total_blob_size;
        let total_after = after.total_sst_size + after.total_blob_size;
        let reclaimed = total_before.saturating_sub(total_after);
        let percent = if total_before > 0 {
            (reclaimed as f64 / total_before as f64) * 100.0
        } else {
            0.0
        };

        out.push_str("\n=== Summary ===\n");
        out.push_str(&format!(
            "Total space (SST+Blob): {:.2} GB → {:.2} GB\n",
            total_before as f64 / 1e9,
            total_after as f64 / 1e9
        ));
        out.push_str(&format!(
            "Reclaimed: {:.2} GB ({:.2}%)\n",
            reclaimed as f64 / 1e9,
            percent
        ));

        if reclaimed == 0 {
            out.push_str("\n⚠️  No space reclaimed. Possible reasons:\n");
            out.push_str("  1. Deleted nodes are still referenced by other roots\n");
            out.push_str("  2. Tombstones not yet compacted (run: rooch db rocksdb-gc)\n");
            out.push_str("  3. Deletion amount too small to measure\n");
        } else if percent < 1.0 {
            out.push_str(&format!(
                "\n✓ Minor reclaim ({:.2}%). Consider more deletions or check for shared nodes.\n",
                percent
            ));
        } else if percent < 10.0 {
            out.push_str(&format!(
                "\n✓ Moderate reclaim ({:.2}%). Deletion effective.\n",
                percent
            ));
        } else {
            out.push_str(&format!(
                "\n✅ Significant reclaim ({:.2}%)! Deletion very effective.\n",
                percent
            ));
        }

        Ok(out)
    }

    fn collect_stats(&self, db_path: &PathBuf) -> Result<DbSnapshot> {
        let existing = raw_store::rocks::RocksDB::list_cf(db_path)?;
        // Build a Vec<&'static str> from known CF lists, filtered by existence on disk
        let mut open_cfs: Vec<&'static str> = Vec::new();
        if existing.iter().any(|s| s == "default") {
            open_cfs.push("default");
        }
        for cf in moveos_store::StoreMeta::get_column_family_names() {
            if existing.iter().any(|s| s == cf) {
                open_cfs.push(cf);
            }
        }
        for cf in rooch_store::StoreMeta::get_column_family_names() {
            if existing.iter().any(|s| s == cf) {
                open_cfs.push(cf);
            }
        }

        let config = RocksdbConfig::default();
        let db = raw_store::rocks::RocksDB::new_readonly(db_path, open_cfs, config)
            .context("Failed to open DB")?;
        let raw = db.inner();
        let cf = raw
            .cf_handle(STATE_NODE_COLUMN_FAMILY_NAME)
            .context("state_node CF not found")?;

        let get_prop = |name: &str| -> u64 {
            raw.property_int_value_cf(&cf, name)
                .ok()
                .flatten()
                .unwrap_or(0)
        };

        Ok(DbSnapshot {
            timestamp: format!(
                "{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ),
            total_sst_size: get_prop("rocksdb.total-sst-files-size"),
            live_sst_size: get_prop("rocksdb.live-sst-files-size"),
            est_live_data: get_prop("rocksdb.estimate-live-data-size"),
            l0_files: get_prop("rocksdb.num-files-at-level0"),
            pending_compact: get_prop("rocksdb.estimate-pending-compaction-bytes"),
            blob_files: get_prop("rocksdb.num-blob-files"),
            total_blob_size: get_prop("rocksdb.total-blob-file-size"),
            live_blob_size: get_prop("rocksdb.live-blob-file-size"),
            garbage_blob_size: get_prop("rocksdb.garbage-blob-file-size"),
        })
    }

    fn format_snapshot(&self, s: &DbSnapshot) -> String {
        format!(
            "  Total SST:      {:.2} GB\n\
             Live SST:       {:.2} GB\n\
             Est. Live Data: {:.2} GB\n\
             L0 Files:       {}\n\
             Pending Compact:{:.2} GB\n\
             Blob Files:     {}\n\
             Total Blob:     {:.2} MB\n\
             Live Blob:      {:.2} MB\n\
             Garbage Blob:   {:.2} MB\n",
            s.total_sst_size as f64 / 1e9,
            s.live_sst_size as f64 / 1e9,
            s.est_live_data as f64 / 1e9,
            s.l0_files,
            s.pending_compact as f64 / 1e9,
            s.blob_files,
            s.total_blob_size as f64 / 1e6,
            s.live_blob_size as f64 / 1e6,
            s.garbage_blob_size as f64 / 1e6
        )
    }

    fn format_diff(&self, label: &str, before: u64, after: u64) -> String {
        let diff = after as i64 - before as i64;
        let percent = if before > 0 {
            (diff as f64 / before as f64) * 100.0
        } else if after > 0 {
            100.0
        } else {
            0.0
        };

        let (indicator, arrow) = match diff.cmp(&0) {
            std::cmp::Ordering::Less => ("OK", "v"),
            std::cmp::Ordering::Greater => ("WARN", "^"),
            std::cmp::Ordering::Equal => ("  ", "="),
        };

        format!(
            "{} {:<16} {:.2} GB {} {:.2} GB ({:+.2}%)\n",
            indicator,
            label,
            before as f64 / 1e9,
            arrow,
            after as f64 / 1e9,
            percent
        )
    }
}
