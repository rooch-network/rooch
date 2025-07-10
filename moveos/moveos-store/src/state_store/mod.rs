// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod incremental_sweep;
pub mod metrics;
pub mod pruner;
pub mod reachability;
pub mod statedb;
pub mod sweep_expired;

use crate::PRUNE_META_COLUMN_FAMILY_NAME;
use crate::REACH_SEEN_COLUMN_FAMILY_NAME;
use crate::STATE_NODE_COLUMN_FAMILY_NAME;
use crate::{NODE_REFCOUNT_COLUMN_FAMILY_NAME, SMT_STALE_INDEX_COLUMN_FAMILY_NAME};
use anyhow::Result;
use moveos_types::h256::H256;
use raw_store::rocks::batch::WriteBatch;
use raw_store::CodecKVStore;
use raw_store::SchemaStore;
use raw_store::{derive_store, WriteOp};
use smt::{NodeReader, NodeWriter};
use std::collections::BTreeMap;

derive_store!(NodeDBStore, H256, Vec<u8>, STATE_NODE_COLUMN_FAMILY_NAME);
derive_store!(
    ReachSeenDBStore,
    H256,
    Vec<u8>,
    REACH_SEEN_COLUMN_FAMILY_NAME
);
derive_store!(
    PruneMetaStore,
    String,
    Vec<u8>,
    PRUNE_META_COLUMN_FAMILY_NAME
);
derive_store!(
    StaleIndexStore,
    (H256, H256),
    Vec<u8>,
    SMT_STALE_INDEX_COLUMN_FAMILY_NAME
);
derive_store!(
    NodeRefcountStore,
    H256,
    u32,
    NODE_REFCOUNT_COLUMN_FAMILY_NAME
);

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
        let batch = WriteBatch::new_with_rows(
            keys.into_iter()
                .map(|k| (k.0.to_vec(), WriteOp::Deletion))
                .collect(),
        );
        self.write_batch_raw(batch)
    }

    /// Write stale indices to cf_smt_stale and update refcount in one loop (non-atomic across CFs).
    pub fn write_stale_indices(&self, stale: &[(H256, H256)]) -> anyhow::Result<()> {
        let instance = self.get_store().store().clone();
        let stale_store = StaleIndexStore::new(instance.clone());
        let ref_store = NodeRefcountStore::new(instance);
        for (stale_root, node_hash) in stale {
            stale_store.kv_put((*stale_root, *node_hash), Vec::new())?;
            ref_store.dec(*node_hash)?;
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

impl NodeRefcountStore {
    pub fn inc(&self, key: H256) -> anyhow::Result<()> {
        let current = self.kv_get(key)?.unwrap_or(0);
        self.kv_put(key, current + 1)
    }

    pub fn dec(&self, key: H256) -> anyhow::Result<()> {
        let current = self.kv_get(key)?.unwrap_or(1);
        let new = current.saturating_sub(1);
        if new == 0 {
            self.remove(key)
        } else {
            self.kv_put(key, new)
        }
    }

    /// Get current refcount, 0 if not present.
    pub fn get_ref(&self, key: H256) -> anyhow::Result<u32> {
        Ok(self.kv_get(key)?.unwrap_or(0))
    }
}

impl StaleIndexStore {
    /// List at most `limit` stale indices whose stale_since_root < cutoff_root.
    pub fn list_before(
        &self,
        cutoff_root: H256,
        limit: usize,
    ) -> anyhow::Result<Vec<(H256, H256)>> {
        let mut res = Vec::new();
        for key in self.keys()? {
            if key.0 < cutoff_root {
                res.push(key);
                if res.len() >= limit {
                    break;
                }
            }
        }
        Ok(res)
    }
}
