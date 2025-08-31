// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::util::try_extract_child_root;
use anyhow::Result;
use moveos_common::bloom_filter::BloomFilter;
use moveos_store::MoveOSStore;
use parking_lot::Mutex;
use primitive_types::H256;
use rayon::prelude::*;
use smt::jellyfish_merkle::node_type::Node;
use smt::NodeReader;
use std::sync::Arc;
use tracing::info;
// no extra high-level imports

/// SweepExpired traverses expired roots (< cutoff) and deletes any node hash not present in ReachableSet.
/// ReachableSet is represented by an in-memory Bloom filter plus optional `reach_seen` CF.
pub struct SweepExpired {
    moveos_store: Arc<MoveOSStore>,
    bloom: Arc<Mutex<BloomFilter>>, // same Bloom used by ReachableBuilder
                                    // metrics: Arc<StateDBMetrics>,
}

impl SweepExpired {
    pub fn new(
        moveos_store: Arc<MoveOSStore>,
        bloom: Arc<Mutex<BloomFilter>>, // pass the same bloom instance
                                        // metrics: Arc<StateDBMetrics>,
    ) -> Self {
        Self {
            moveos_store,
            bloom,
        }
    }

    /// Sweep nodes belonging to `expired_roots`. Returns number of deleted nodes.
    pub fn sweep(&self, expired_roots: Vec<H256>, workers: usize) -> Result<u64> {
        let deleted = Arc::new(std::sync::atomic::AtomicU64::new(0));

        expired_roots
            .into_par_iter()
            .with_max_len(workers)
            .for_each(|root| {
                if let Err(e) = self.sweep_root(root, &deleted) {
                    tracing::error!("sweep error: {}", e);
                }
            });

        let deleted_count = deleted.load(std::sync::atomic::Ordering::Relaxed);
        // self.metrics
        //     .pruner_sweep_nodes_deleted
        //     .with_label_values(&["sweep"])
        //     .observe(deleted_count as f64);

        Ok(deleted_count)
    }

    fn is_reachable(&self, hash: &H256) -> bool {
        // first check bloom
        if self.bloom.lock().contains(hash) {
            return true;
        }
        // The probability of bloomfitler misjudgment will only lead to fewer deletions, not wrong deletions

        // // fallback to reach_seen CF if enabled
        // if let Some(store) = &self.reach_seen {
        //     if let Ok(Some(_)) = store.kv_get(*hash) {
        //         return true;
        //     }
        // }
        false
    }

    fn sweep_root(
        &self,
        root_hash: H256,
        deleted: &Arc<std::sync::atomic::AtomicU64>,
    ) -> Result<()> {
        use std::collections::VecDeque;
        let mut stack = VecDeque::new();
        stack.push_back(root_hash);
        let mut batch = Vec::with_capacity(1000); // Process deletion in small batches
        let mut total_deleted: u64 = 0;

        while let Some(node_hash) = stack.pop_back() {
            // if node is reachable by other live roots, keep it
            if self.is_reachable(&node_hash) {
                continue;
            }

            // Add to current batch for deletion
            batch.push(node_hash);
            total_deleted += 1;

            // Process batch if it reaches the threshold
            if batch.len() >= 1000 {
                // verify first, then delete
                self.moveos_store.node_store.delete_nodes(batch.clone())?;
                info!(
                    "Sweep expired this loop, state root {:?}, deletes batch size {}, total delete size {}",
                    root_hash,
                    batch.len(),
                    total_deleted
                );
                if total_deleted <= 1000 || total_deleted % 10000 == 0 {
                    info!("Sweep expired this loop delete batch {:?}", batch);
                }
                deleted.fetch_add(batch.len() as u64, std::sync::atomic::Ordering::Relaxed);
                batch.clear();
            }

            // Inside dfs loop, traverse children and leaf add to stack
            // Include both Global‐State and Table-State JMT
            if let Some(bytes) = self.moveos_store.node_store.get(&node_hash)? {
                // If this leaf embeds another table root, push it to the stack for further traversal.
                if let Some(child_root) = try_extract_child_root(&bytes) {
                    stack.push_back(child_root);
                } else if let Ok(Node::Internal(internal)) = Node::<H256, Vec<u8>>::decode(&bytes) {
                    for child_hash in internal.all_child() {
                        stack.push_back(child_hash.into());
                    }
                }
            }
        }

        // Process any remaining nodes in the final batch
        if !batch.is_empty() {
            // verify first, then delete
            self.moveos_store.node_store.delete_nodes(batch.clone())?;
            info!(
                "Sweep expired delete final loop, state root {:?}, final batch size {}, total delete size {}",
                root_hash,
                batch.len(),
                total_deleted
            );
            if total_deleted < 1000 || total_deleted % 10000 == 0 {
                info!("Sweep expired delete final batch {:?}", batch);
            }
            deleted.fetch_add(batch.len() as u64, std::sync::atomic::Ordering::Relaxed);
        }

        info!(
            "Completed sweeping root, total deleted nodes: {}",
            total_deleted
        );
        Ok(())
    }
}
