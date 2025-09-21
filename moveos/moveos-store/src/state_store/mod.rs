// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod metrics;
pub mod statedb;

use crate::STATE_NODE_COLUMN_FAMILY_NAME;
use anyhow::Result;
use moveos_types::h256::H256;
use raw_store::rocks::batch::WriteBatch;
use raw_store::WriteOp;
use raw_store::{derive_store, CodecKVStore};
use smt::{NodeReader, NodeWriter};
use std::collections::BTreeMap;

derive_store!(NodeDBStore, H256, Vec<u8>, STATE_NODE_COLUMN_FAMILY_NAME);

impl NodeDBStore {
    pub fn put(&self, key: H256, node: Vec<u8>) -> Result<()> {
        self.put_raw(key.as_bytes().to_vec(), node)
    }

    pub fn write_nodes(&self, nodes: BTreeMap<H256, Vec<u8>>) -> Result<()> {
        let batch = WriteBatch::new_with_rows(
            nodes
                .into_iter()
                .map(|(k, v)| (k.0.to_vec(), WriteOp::Value(v)))
                .collect(),
        );
        self.write_batch_raw(batch)
    }

    /// Delete `keys` from the node column family.
    ///
    /// * `flush` – if true, the column family is flushed immediately after the deletions are
    ///   written.  For large-scale bulk deletions (e.g. pruner sweep) passing `false` allows the
    ///   caller to defer flushing and compaction until the very end, which avoids creating a large
    ///   number of tiny SST files and the accompanying temporary disk usage spike.
    pub fn delete_nodes_with_flush(&self, keys: Vec<H256>, flush: bool) -> Result<()> {
        if let Some(wrapper) = self.store.store().db() {
            use rocksdb::{WriteBatch as RawBatch, WriteOptions};
            let raw_db = wrapper.inner();
            let cf = raw_db
                .cf_handle(STATE_NODE_COLUMN_FAMILY_NAME)
                .expect("state node cf");

            // Build per-key delete batch; keeps exact control of which hashes are removed.
            let mut wb = RawBatch::default();
            for h in keys {
                wb.delete_cf(&cf, h.0);
            }

            // Disable WAL for performance – crash consistency is not critical for pruning.
            let mut opts = WriteOptions::default();
            opts.disable_wal(true);
            raw_db.write_opt(wb, &opts)?;

            if flush {
                raw_db.flush_cf(&cf)?;
            }
            Ok(())
        } else {
            // Fallback path (e.g. in-memory DB during tests)
            let batch = WriteBatch::new_with_rows(
                keys.into_iter()
                    .map(|k| (k.0.to_vec(), WriteOp::Deletion))
                    .collect(),
            );
            self.write_batch_raw(batch)
        }
    }

    /// Delete a continuous range of node hashes `[start, end)` using RocksDB DeleteRange.
    /// `start` inclusive, `end` exclusive. Caller guarantees the range keys share the same prefix
    /// and are fully unreachable. When `flush` is true the column family is flushed immediately
    /// after the delete-range tombstone is written.
    pub fn delete_range_nodes(&self, start: H256, end: H256, flush: bool) -> Result<()> {
        if let Some(wrapper) = self.store.store().db() {
            let raw_db = wrapper.inner();
            let cf = raw_db
                .cf_handle(STATE_NODE_COLUMN_FAMILY_NAME)
                .expect("state node cf");

            raw_db.delete_range_cf(cf, start.0, end.0)?;

            if flush {
                raw_db.flush_cf(&cf)?;
            }
        }
        Ok(())
    }

    /// Backward-compat helper that preserves the old behaviour (delete then flush)
    pub fn delete_nodes(&self, keys: Vec<H256>) -> Result<()> {
        self.delete_nodes_with_flush(keys, /*flush*/ true)
    }

    /// Flush the state-node column family and trigger a manual compaction.  Call this once after a
    /// bulk pruning run to quickly reclaim space without generating a huge number of temporary
    /// files during the run.
    pub fn flush_and_compact(&self) -> Result<()> {
        if let Some(wrapper) = self.store.store().db() {
            let raw_db = wrapper.inner();
            let cf = raw_db
                .cf_handle(STATE_NODE_COLUMN_FAMILY_NAME)
                .expect("state node cf");

            raw_db.flush_cf(&cf)?;
            raw_db.compact_range_cf(&cf, None::<&[u8]>, None::<&[u8]>);
        }
        Ok(())
    }

    /// Flush the state node column family without triggering compaction.
    pub fn flush_only(&self) -> Result<()> {
        if let Some(wrapper) = self.store.store().db() {
            let raw_db = wrapper.inner();
            let cf = raw_db
                .cf_handle(STATE_NODE_COLUMN_FAMILY_NAME)
                .expect("state node cf");
            raw_db.flush_cf(&cf)?;
        }
        Ok(())
    }

    /// Perform an aggressive compaction that forces all levels to be merged down to the bottommost
    /// level, then immediately purge obsolete files. This effectively removes tombstoned keys and
    /// reclaims disk space.
    pub fn aggressive_compact(&self) -> Result<()> {
        if let Some(wrapper) = self.store.store().db() {
            let raw_db = wrapper.inner();
            let cf = raw_db
                .cf_handle(STATE_NODE_COLUMN_FAMILY_NAME)
                .expect("state node cf");

            // Disable auto compactions during the manual compaction window to reduce write stall
            raw_db.set_options_cf(&cf, &[("disable_auto_compactions", "true")])?;

            // Compact the whole key range and force bottommost level rewrite.
            use rocksdb::{BottommostLevelCompaction, CompactOptions};
            let mut copt = CompactOptions::default();
            copt.set_bottommost_level_compaction(BottommostLevelCompaction::Force);
            raw_db.compact_range_cf_opt(&cf, None::<&[u8]>, None::<&[u8]>, &copt);
            // raw_db.compact_range_cf(&cf, None::<&[u8]>, None::<&[u8]>);

            // Re-enable automatic compactions.
            raw_db.set_options_cf(&cf, &[("disable_auto_compactions", "false")])?;
        }
        Ok(())
    }
}

pub fn nodes_to_write_batch(nodes: BTreeMap<H256, Vec<u8>>) -> WriteBatch {
    WriteBatch::new_with_rows(
        nodes
            .into_iter()
            .map(|(k, v)| (k.0.to_vec(), WriteOp::Value(v)))
            .collect(),
    )
}

impl NodeReader for NodeDBStore {
    fn get(&self, hash: &H256) -> Result<Option<Vec<u8>>> {
        self.get_raw(hash.as_bytes())
    }
}

impl NodeWriter for NodeDBStore {
    fn write_nodes(&self, nodes: BTreeMap<H256, Vec<u8>>) -> Result<()> {
        NodeDBStore::write_nodes(self, nodes)
    }
}
