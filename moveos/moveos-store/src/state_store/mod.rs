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

    pub fn delete_nodes(&self, keys: Vec<H256>) -> Result<()> {
        if let Some(wrapper) = self.store.store().db() {
            use rocksdb::{WriteBatch as RawBatch, WriteOptions};
            let raw_db = wrapper.inner();
            let cf = raw_db
                .cf_handle(STATE_NODE_COLUMN_FAMILY_NAME)
                .expect("state node cf");

            // Build per-key delete batch instead of DeleteRange to prevent accidental
            // deletion of keys that are not explicitly listed. Although this may generate
            // more individual delete operations, they are still written atomically in one
            // WriteBatch, so the performance impact is limited while ensuring correctness.

            let mut wb = RawBatch::default();
            for h in keys {
                wb.delete_cf(&cf, &h.0);
            }

            // Write with WAL disabled
            let mut opts = WriteOptions::default();
            opts.disable_wal(true);
            raw_db.write_opt(wb, &opts)?;

            // Flush memtable so tombstone file is generated quickly
            raw_db.flush_cf(&cf)?;
            Ok(())
        } else {
            // fallback (should not happen in production)
            let batch = WriteBatch::new_with_rows(
                keys.into_iter()
                    .map(|k| (k.0.to_vec(), WriteOp::Deletion))
                    .collect(),
            );
            self.write_batch_raw(batch)
        }
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
