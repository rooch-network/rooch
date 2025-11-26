// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use moveos_store::MoveOSStore;
use std::sync::Arc;
use tracing::info;

/// IncrementalSweep scans cf_smt_stale and deletes nodes whose refcount==0
/// for stale_since_order < cutoff_order.
pub struct IncrementalSweep {
    moveos_store: Arc<MoveOSStore>,
}

impl IncrementalSweep {
    pub fn new(moveos_store: Arc<MoveOSStore>) -> Self {
        Self { moveos_store }
    }

    /// Sweep at most `batch` indices per call.
    pub fn sweep(&self, cutoff_order: u64, batch: usize) -> Result<usize> {
        let indices = self
            .moveos_store
            .prune_store
            .list_before(cutoff_order, batch)?;
        if indices.is_empty() {
            return Ok(0);
        }

        let mut to_delete_nodes = Vec::new();
        let mut to_delete_indices = Vec::new();

        for (stale_root, node_hash) in indices {
            match self.moveos_store.prune_store.get_node_refcount(node_hash)? {
                Some(0) => {
                    to_delete_nodes.push(node_hash);
                    to_delete_indices.push((stale_root, node_hash));
                }
                Some(_) => {
                    // still referenced
                }
                None => {
                    tracing::warn!(
                        ?node_hash,
                        ?stale_root,
                        "IncrementalSweep: refcount missing for stale node, skipping delete"
                    );
                }
            }
        }

        if !to_delete_nodes.is_empty() {
            // Log delete batch for traceability
            let sample: Vec<String> = to_delete_nodes
                .iter()
                .take(20)
                .map(|h| format!("{:#x}", h))
                .collect();
            info!(
                cutoff_order,
                delete_count = to_delete_nodes.len(),
                sample = ?sample,
                "IncrementalSweep deleting nodes with refcount==0"
            );
            // delete nodes
            self.moveos_store
                .node_store
                .delete_nodes(to_delete_nodes.clone())?;
            // delete indices and refcount
            for (_root, node_hash) in to_delete_indices {
                let _ = self
                    .moveos_store
                    .prune_store
                    .remove_stale_indice((_root, node_hash));
                let _ = self
                    .moveos_store
                    .prune_store
                    .remove_node_refcount(node_hash);
            }
        }
        Ok(to_delete_nodes.len())
    }
}
