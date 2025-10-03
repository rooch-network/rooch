// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::util::try_extract_child_root;
use anyhow::Result;
use moveos_common::bloom_filter::BloomFilter;
use moveos_store::prune::PruneStore;
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
    deleted_state_root_bloom: Arc<Mutex<BloomFilter>>, // BloomFilter tracking deleted state roots
    processed_roots_count: Arc<std::sync::atomic::AtomicU64>, // Counter for aggressive compaction
                                    // metrics: Arc<StateDBMetrics>,
}

impl SweepExpired {
    pub fn new(
        moveos_store: Arc<MoveOSStore>,
        bloom: Arc<Mutex<BloomFilter>>, // pass the same bloom instance
                                        // metrics: Arc<StateDBMetrics>,
    ) -> Self {
        // Load or create deleted roots bloom filter
        let deleted_state_root_bloom = moveos_store
            .load_deleted_state_root_bloom()
            .ok()
            .flatten()
            .unwrap_or_else(|| {
                // Create a new BloomFilter: 2^38 bits, 4 hash functions
                // Can track ~137438M state roots with <1% false positive rate
                BloomFilter::new(1 << 38, 4)
            });

        Self {
            moveos_store,
            bloom,
            deleted_state_root_bloom: Arc::new(Mutex::new(deleted_state_root_bloom)),
            processed_roots_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Sweep nodes belonging to `expired_roots`. Returns number of deleted nodes.
    ///
    /// # Arguments
    /// * `expired_roots` - Vec of (state_root, tx_order) pairs for expired roots
    /// * `workers` - Number of parallel workers
    ///
    /// # Ordering Guarantee
    /// This function ensures that roots are processed in descending tx_order.
    /// Roots are divided into mini-batches, where each batch is processed in parallel,
    /// but batches themselves are processed sequentially from highest to lowest tx_order.
    /// This guarantees that if the process crashes, we maintain the oldest continuous
    /// history segment intact.
    pub fn sweep(&self, mut expired_roots: Vec<(H256, u64)>, workers: usize) -> Result<u64> {
        // ✅ Step 1: Sort by tx_order in descending order (largest first)
        // This ensures we delete from newest to oldest, preserving oldest history on crash
        expired_roots.sort_by(|a, b| b.1.cmp(&a.1));

        // Filter out already-deleted state roots
        let roots_to_process: Vec<(H256, u64)> = {
            let deleted_bloom = self.deleted_state_root_bloom.lock();
            expired_roots
                .into_iter()
                .filter(|(root, _)| !deleted_bloom.contains(root))
                .collect()
        };

        info!(
            "Sweep: {} roots to process (after filtering deleted roots)",
            roots_to_process.len()
        );

        if roots_to_process.is_empty() {
            return Ok(0);
        }

        // Log the tx_order range for verification
        let max_order = roots_to_process.first().map(|(_, o)| *o).unwrap_or(0);
        let min_order = roots_to_process.last().map(|(_, o)| *o).unwrap_or(0);
        info!(
            "Processing tx_order range: {} (newest) -> {} (oldest)",
            max_order, min_order
        );

        let deleted = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let mut all_processed_roots = Vec::new();

        // ✅ Step 2: Divide into mini-batches
        // Each mini-batch is processed in parallel, but batches are sequential
        let mini_batch_size = workers * 10; // e.g., if workers=8, mini_batch=80
        let total_batches = (roots_to_process.len() + mini_batch_size - 1) / mini_batch_size;

        info!(
            "Processing {} roots in {} mini-batches (size: {} each)",
            roots_to_process.len(),
            total_batches,
            mini_batch_size
        );

        // ✅ Step 3: Process batches sequentially (largest tx_order first)
        for (batch_idx, mini_batch) in roots_to_process.chunks(mini_batch_size).enumerate() {
            let batch_max_order = mini_batch.first().map(|(_, o)| *o).unwrap_or(0);
            let batch_min_order = mini_batch.last().map(|(_, o)| *o).unwrap_or(0);

            info!(
                "Processing mini-batch {}/{}: {} roots (tx_order {} -> {})",
                batch_idx + 1,
                total_batches,
                mini_batch.len(),
                batch_max_order,
                batch_min_order
            );

            let processed_roots = Arc::new(Mutex::new(Vec::new()));

            // ✅ Step 4: Within each mini-batch, process in parallel
            mini_batch.par_iter().for_each(|&(root, tx_order)| {
                match self.sweep_root(root, tx_order, &deleted) {
                    Ok(()) => {
                        processed_roots.lock().push(root);
                    }
                    Err(e) => {
                        tracing::error!(
                            "sweep error for root {:?} at tx_order {}: {}",
                            root,
                            tx_order,
                            e
                        );
                    }
                }
            });

            // ✅ Step 5: After each mini-batch, immediately persist progress
            {
                let mut deleted_bloom = self.deleted_state_root_bloom.lock();
                for root in processed_roots.lock().iter() {
                    deleted_bloom.insert(root);
                }
                all_processed_roots.extend_from_slice(&processed_roots.lock());
            }

            // Persist the bloom filter after each batch (crash recovery checkpoint)
            if let Err(e) = self
                .moveos_store
                .save_deleted_state_root_bloom(self.deleted_state_root_bloom.lock().clone())
            {
                tracing::warn!(
                    "Failed to save deleted roots bloom after batch {}: {}",
                    batch_idx + 1,
                    e
                );
            } else {
                info!(
                    "Saved deleted roots bloom after batch {}/{}, total processed: {}",
                    batch_idx + 1,
                    total_batches,
                    all_processed_roots.len()
                );
            }
        }

        let deleted_count = deleted.load(std::sync::atomic::Ordering::Relaxed);
        info!(
            "Sweep completed: processed {} roots, deleted {} nodes",
            all_processed_roots.len(),
            deleted_count
        );

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
        tx_order: u64,
        deleted: &Arc<std::sync::atomic::AtomicU64>,
    ) -> Result<()> {
        use std::collections::VecDeque;
        let mut stack = VecDeque::new();
        stack.push_back(root_hash);
        let mut batch = Vec::with_capacity(1000); // Process deletion in small batches
        let mut total_deleted: u64 = 0;
        let mut since_last_flush: u64 = 0;
        let flush_interval: u64 = 100000; // flush every 100k deletions

        while let Some(node_hash) = stack.pop_back() {
            // Step 1: Read node content and extract children FIRST (before any checks)
            // This is critical: we must read before checking reachability or deletion
            // to ensure we can traverse the entire subtree even in concurrent scenarios
            let mut children_to_traverse = Vec::new();
            if let Some(bytes) = self.moveos_store.node_store.get(&node_hash)? {
                // If this leaf embeds another table root, collect it for traversal
                if let Some(child_root) = try_extract_child_root(&bytes) {
                    children_to_traverse.push(child_root);
                } else if let Ok(Node::Internal(internal)) = Node::<H256, Vec<u8>>::decode(&bytes) {
                    // Collect all children of internal node
                    for child_hash in internal.all_child() {
                        children_to_traverse.push(child_hash.into());
                    }
                }
            }

            // Step 2: Check if node is reachable by other live roots
            // If reachable, skip deletion but we've already extracted children (safe to ignore them)
            if self.is_reachable(&node_hash) {
                continue;
            }

            // Step 3: Node is not reachable, mark for deletion
            batch.push(node_hash);
            total_deleted += 1;

            // Step 4: Add children to stack for further traversal
            for child in children_to_traverse {
                stack.push_back(child);
            }

            // Step 5: Process batch if it reaches the threshold
            if batch.len() >= 1000 {
                self.moveos_store
                    .node_store
                    .delete_nodes_with_flush(batch.clone(), /*flush*/ false)?;
                info!(
                    "Sweep expired this loop, tx_order {}, state root {:?}, deletes batch size {}, total delete size {}",
                    tx_order,
                    root_hash,
                    batch.len(),
                    total_deleted
                );

                if total_deleted <= 1000 || total_deleted % 10000 == 0 {
                    info!("Sweep expired this loop delete batch {:?}", batch);
                }
                deleted.fetch_add(batch.len() as u64, std::sync::atomic::Ordering::Relaxed);
                batch.clear();

                since_last_flush += 1000;
                if since_last_flush >= flush_interval {
                    // Layer 1: Periodic lightweight flush only (avoid huge memtables)
                    // Don't compact here to avoid performance overhead during traversal
                    self.moveos_store.node_store.flush_only()?;
                    since_last_flush = 0;
                }
            }
        }

        // Process any remaining nodes in the final batch
        if !batch.is_empty() {
            // verify first, then delete
            self.moveos_store
                .node_store
                .delete_nodes_with_flush(batch.clone(), /*flush*/ true)?;
            info!(
                "Sweep expired delete final loop, tx_order {}, state root {:?}, final batch size {}, total delete size {}",
                tx_order,
                root_hash,
                batch.len(),
                total_deleted
            );
            if total_deleted < 1000 || total_deleted % 10000 == 0 {
                info!("Sweep expired delete final batch {:?}", batch);
            }
            deleted.fetch_add(batch.len() as u64, std::sync::atomic::Ordering::Relaxed);
        }

        // Layer 2/3: Compact after finishing this root
        // Use aggressive compaction every N roots, otherwise standard compaction
        let count = self
            .processed_roots_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        // let aggressive_compact_interval = 5000;
        let aggressive_compact_interval = 50;

        if count % aggressive_compact_interval == 0 {
            // Layer 3: Aggressive compaction to fully reclaim disk space
            info!(
                "Triggering aggressive compaction after {} roots (tx_order {}, root {:?})",
                count, tx_order, root_hash
            );
            self.moveos_store.node_store.aggressive_compact()?;
            info!("Aggressive compaction completed successfully");
        } else {
            // Layer 2: Standard compaction to merge SST files
            self.moveos_store.node_store.flush_and_compact()?;
        }

        info!(
            "Completed sweeping tx_order {}, state root {:?}, total deleted nodes: {}",
            tx_order, root_hash, total_deleted
        );
        Ok(())
    }
}
