use crate::state_store::{NodeDBStore, NodeRefcountStore, StaleIndexStore};
use anyhow::Result;
use primitive_types::H256;
use raw_store::CodecKVStore;
use std::sync::Arc;

/// IncrementalSweep scans cf_smt_stale and deletes nodes whose refcount==0
/// for stale_since_root < cutoff_root.
pub struct IncrementalSweep {
    node_store: Arc<NodeDBStore>,
    stale_store: Arc<StaleIndexStore>,
    ref_store: Arc<NodeRefcountStore>,
}

impl IncrementalSweep {
    pub fn new(
        node_store: Arc<NodeDBStore>,
        stale_store: Arc<StaleIndexStore>,
        ref_store: Arc<NodeRefcountStore>,
    ) -> Self {
        Self {
            node_store,
            stale_store,
            ref_store,
        }
    }

    /// Sweep at most `batch` indices per call.
    pub fn sweep(&self, cutoff_root: H256, batch: usize) -> Result<usize> {
        let indices = self.stale_store.list_before(cutoff_root, batch)?;
        if indices.is_empty() {
            return Ok(0);
        }

        let mut to_delete_nodes = Vec::new();
        let mut to_delete_indices = Vec::new();

        for (stale_root, node_hash) in indices {
            if self.ref_store.get_ref(node_hash)? == 0 {
                to_delete_nodes.push(node_hash);
                to_delete_indices.push((stale_root, node_hash));
            }
        }

        if !to_delete_nodes.is_empty() {
            // delete nodes
            self.node_store.delete_nodes(to_delete_nodes.clone())?;
            // delete indices and refcount
            for (_root, node_hash) in to_delete_indices {
                let _ = self.stale_store.as_ref().remove((_root, node_hash));
                let _ = self.ref_store.as_ref().remove(node_hash);
            }
        }
        Ok(to_delete_nodes.len())
    }
}
