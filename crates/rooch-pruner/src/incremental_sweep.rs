// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use moveos_store::MoveOSStore;
use primitive_types::H256;
use std::sync::Arc;

/// IncrementalSweep scans cf_smt_stale and deletes nodes whose refcount==0
/// for stale_since_root < cutoff_root.
pub struct IncrementalSweep {
    moveos_store: Arc<MoveOSStore>,
}

impl IncrementalSweep {
    pub fn new(moveos_store: Arc<MoveOSStore>) -> Self {
        Self { moveos_store }
    }

    /// Sweep at most `batch` indices per call.
    pub fn sweep(&self, cutoff_root: H256, batch: usize) -> Result<usize> {
        let indices = self
            .moveos_store
            .prune_store
            .list_before(cutoff_root, batch)?;
        if indices.is_empty() {
            return Ok(0);
        }

        let mut to_delete_nodes = Vec::new();
        let mut to_delete_indices = Vec::new();

        for (stale_root, node_hash) in indices {
            if self.moveos_store.prune_store.get_node_refcount(node_hash)? == 0 {
                to_delete_nodes.push(node_hash);
                to_delete_indices.push((stale_root, node_hash));
            }
        }

        if !to_delete_nodes.is_empty() {
            // delete nodes
            self.moveos_store
                .node_store
                .delete_nodes(to_delete_nodes.clone())?;
            // delete indices and refcount
            for (_root, node_hash) in to_delete_indices {
                let _ = self
                    .moveos_store
                    .prune_store
                    .stale_index_store
                    .remove((_root, node_hash));
                let _ = self
                    .moveos_store
                    .prune_store
                    .node_refcount_store
                    .remove(node_hash);
            }
        }
        Ok(to_delete_nodes.len())
    }
}
