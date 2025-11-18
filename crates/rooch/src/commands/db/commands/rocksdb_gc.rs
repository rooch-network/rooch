// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use crate::commands::db::commands::expand_tilde;
use anyhow::{Context, Result};
use async_trait::async_trait;
use clap::Parser;
use moveos_config::store_config::RocksdbConfig;
use rooch_types::error::RoochResult;
use std::path::PathBuf;

/// Force flush + bottommost compaction on state_node CF and print before/after stats
#[derive(Parser, Debug)]
pub struct RocksDBGcCommand {
    /// Path to RocksDB directory
    #[clap(long, default_value = "~/.rooch/local/roochdb/store")]
    pub db_path: String,
}

#[async_trait]
impl CommandAction<String> for RocksDBGcCommand {
    async fn execute(self) -> RoochResult<String> {
        let result = self
            .execute_impl()
            .map_err(rooch_types::error::RoochError::from)?;
        Ok(result)
    }
}

impl RocksDBGcCommand {
    fn execute_impl(self) -> Result<String> {
        let path_str = expand_tilde(&self.db_path)?;
        let db_path = PathBuf::from(&path_str);
        if !db_path.exists() {
            return Err(anyhow::anyhow!(
                "Database path does not exist: {:?}",
                db_path
            ));
        }

        // Only open existing CFs and ensure state_node exists
        let existing =
            raw_store::rocks::RocksDB::list_cf(&db_path).context("List column families failed")?;
        if !existing.iter().any(|s| s == "state_node") {
            return Err(anyhow::anyhow!(
                "state_node column family not found in {:?}",
                db_path
            ));
        }
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
        let db = raw_store::rocks::RocksDB::new_readonly(&db_path, open_cfs.clone(), config)
            .context("Failed to open RocksDB read-only; stop the node before running GC")?;
        let raw = db.inner();
        let cf = raw.cf_handle("state_node").context("state_node handle")?;

        let before_sst = raw
            .property_int_value_cf(&cf, "rocksdb.total-sst-files-size")
            .ok()
            .flatten()
            .unwrap_or(0);
        let before_blob = raw
            .property_int_value_cf(&cf, "rocksdb.total-blob-file-size")
            .ok()
            .flatten()
            .unwrap_or(0);

        // Try to switch to full access to run compaction
        drop(db);
        let config = RocksdbConfig::default();
        let db_rw = raw_store::rocks::RocksDB::new(&db_path, open_cfs, config)
            .context("Failed to open RocksDB read-write; ensure node is stopped")?;
        let raw_rw = db_rw.inner();
        let cf_rw = raw_rw
            .cf_handle("state_node")
            .context("state_node handle")?;

        // Flush first then bottommost forced compaction
        raw_rw.flush_wal(true)?;
        raw_rw.flush_cf(&cf_rw)?;
        use rocksdb::{BottommostLevelCompaction, CompactOptions};
        let mut copt = CompactOptions::default();
        copt.set_bottommost_level_compaction(BottommostLevelCompaction::Force);
        copt.set_exclusive_manual_compaction(true);
        let start = std::time::Instant::now();
        raw_rw.compact_range_cf_opt(&cf_rw, None::<&[u8]>, None::<&[u8]>, &copt);
        let elapsed = start.elapsed();

        let after_sst = raw_rw
            .property_int_value_cf(&cf_rw, "rocksdb.total-sst-files-size")
            .ok()
            .flatten()
            .unwrap_or(0);
        let after_blob = raw_rw
            .property_int_value_cf(&cf_rw, "rocksdb.total-blob-file-size")
            .ok()
            .flatten()
            .unwrap_or(0);

        let mut out = String::new();
        out.push_str("=== RocksDB GC Result (state_node) ===\n");
        out.push_str(&format!(
            "SST: before {:.2} GB -> after {:.2} GB, reclaimed {:.2} GB\n",
            before_sst as f64 / 1e9,
            after_sst as f64 / 1e9,
            (before_sst.saturating_sub(after_sst)) as f64 / 1e9
        ));
        out.push_str(&format!(
            "Blob: before {:.2} GB -> after {:.2} GB, reclaimed {:.2} GB\n",
            before_blob as f64 / 1e9,
            after_blob as f64 / 1e9,
            (before_blob.saturating_sub(after_blob)) as f64 / 1e9
        ));
        out.push_str(&format!("Duration: {:?}\n", elapsed));
        Ok(out)
    }
}
