// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::marker::NodeMarker;
use crate::util::try_extract_child_root;
use anyhow::Result;
use moveos_common::bloom_filter::BloomFilter;
use moveos_store::MoveOSStore;
use parking_lot::Mutex;
use primitive_types::H256;
use rayon::prelude::*;
use smt::jellyfish_merkle::node_type::Node;
use smt::NodeReader;
use std::collections::VecDeque;
use std::sync::Arc;
use tracing::info;

/// Build reachable set by parallel DFS over live roots.
/// Writes seen node hashes into optional ReachSeenDBStore to avoid re-traversal on crash resume.
///
/// Design notes:
/// * A global BloomFilter is used for hot-path deduplication.
/// * When `bloom` is saturated, overflow hashes are flushed into RocksDB CF `reach_seen`.
/// * Builder can be resumed by seeding Bloom with hashes already in `reach_seen`.
pub struct ReachableBuilder {
    moveos_store: Arc<MoveOSStore>,
    bloom: Arc<Mutex<BloomFilter>>,
    // metrics: Arc<StateDBMetrics>,
}

impl ReachableBuilder {
    pub fn new(moveos_store: Arc<MoveOSStore>, bloom: Arc<Mutex<BloomFilter>>) -> Self {
        Self {
            moveos_store,
            bloom,
        }
    }

    /// Build reachable set starting from `live_roots`.
    /// Returns number of unique reachable nodes scanned.
    pub fn build(&self, live_roots: Vec<H256>, workers: usize) -> Result<u64> {
        // // Seed bloom with previously seen hashes if reach_seen cf is enabled
        // if let Some(reach_store) = &self.reach_seen {
        //     // iterate keys (may be large). For first version we skip preloading to keep simple.
        //     // TODO: preload if necessary.
        //     let _ = reach_store; // silence unused
        // }

        let counter = Arc::new(std::sync::atomic::AtomicU64::new(0));

        live_roots
            .into_par_iter()
            .with_max_len(workers)
            .for_each(|root| {
                if let Err(e) = self.dfs_from_root(root, &counter) {
                    tracing::error!("DFS error: {}", e);
                }
            });

        let scanned = counter.load(std::sync::atomic::Ordering::Relaxed);
        // record metrics
        // self.metrics
        //     .pruner_reachable_nodes_scanned
        //     .with_label_values(&["build"])
        //     .observe(scanned as f64);

        Ok(scanned)
    }

    fn dfs_from_root(
        &self,
        root_hash: H256,
        counter: &Arc<std::sync::atomic::AtomicU64>,
    ) -> Result<()> {
        use std::sync::atomic;
        let mut stack = VecDeque::new();
        stack.push_back(root_hash);
        let mut curr_count: u32 = 0;

        while let Some(node_hash) = stack.pop_back() {
            // bloom check
            {
                let mut bloom_guard = self.bloom.lock();
                if bloom_guard.contains(&node_hash) {
                    continue;
                }
                bloom_guard.insert(&node_hash);
            }

            // // optional spill to reach_seen
            // if let Some(reach_store) = &self.reach_seen {
            //     // ignore errors
            //     let _ = reach_store.kv_put(node_hash, Vec::new());
            // }

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

            curr_count += 1;
            counter.fetch_add(1, atomic::Ordering::Relaxed);

            if curr_count % 10000 == 0 {
                info!(
                    "ReachableBuilder dfs_from_root looping, curr_count {}",
                    curr_count
                );
            }
        }
        info!(
            "ReachableBuilder dfs_from_root done, root_hash {:?} recursive child size {}",
            root_hash, curr_count
        );
        Ok(())
    }

    /// Build reachable set using a custom NodeMarker instead of BloomFilter
    /// This is designed for garbage collection scenarios where we need precise marking
    /// Returns number of unique reachable nodes scanned.
    pub fn build_with_marker(
        &self,
        live_roots: Vec<H256>,
        workers: usize,
        marker: Box<dyn NodeMarker>,
    ) -> Result<u64> {
        let counter = Arc::new(std::sync::atomic::AtomicU64::new(0));

        // Use Rayon for parallel processing with maximum parallelism
        live_roots
            .into_par_iter()
            .with_max_len(workers)
            .for_each(|root| {
                if let Err(e) = self.dfs_from_root_with_marker(root, &counter, marker.as_ref()) {
                    tracing::error!("DFS error with marker: {}", e);
                }
            });

        let scanned = counter.load(std::sync::atomic::Ordering::Relaxed);
        tracing::info!(
            "ReachableBuilder build_with_marker completed: {} nodes scanned, marker type: {}",
            scanned,
            marker.marker_type()
        );

        Ok(scanned)
    }

    /// DFS traversal using NodeMarker for precise marking instead of BloomFilter
    /// This method is designed for garbage collection where we need exact tracking
    fn dfs_from_root_with_marker(
        &self,
        root_hash: H256,
        counter: &Arc<std::sync::atomic::AtomicU64>,
        marker: &dyn NodeMarker,
    ) -> Result<()> {
        use std::sync::atomic;
        let mut stack = VecDeque::new();
        stack.push_back(root_hash);
        let mut curr_count: u32 = 0;

        while let Some(node_hash) = stack.pop_back() {
            // Check if node is already marked using NodeMarker
            if marker.is_marked(&node_hash) {
                continue;
            }

            // Mark the node as reachable
            let newly_marked = marker.mark(node_hash)?;
            if !newly_marked {
                // Already marked by another thread (race condition)
                continue;
            }

            // Traverse the node to find children
            // Include both Global‐State and Table-State JMT nodes
            if let Some(bytes) = self.moveos_store.node_store.get(&node_hash)? {
                // If this leaf embeds another table root, push it to the stack for further traversal.
                if let Some(child_root) = try_extract_child_root(&bytes) {
                    stack.push_back(child_root);
                } else if let Ok(Node::Internal(internal)) = Node::<H256, Vec<u8>>::decode(&bytes) {
                    // For internal nodes, add all children to the stack
                    for child_hash in internal.all_child() {
                        stack.push_back(child_hash.into());
                    }
                }
            }

            curr_count += 1;
            counter.fetch_add(1, atomic::Ordering::Relaxed);

            // Log progress every 10,000 nodes
            if curr_count % 10000 == 0 {
                info!(
                    "ReachableBuilder dfs_from_root_with_marker looping, curr_count {}, marker: {}",
                    curr_count,
                    marker.marker_type()
                );
            }
        }

        info!(
            "ReachableBuilder dfs_from_root_with_marker done, root_hash {:?}, recursive child size {}, total marked: {}",
            root_hash,
            curr_count,
            marker.marked_count()
        );
        Ok(())
    }

    /// Hybrid build method that uses both BloomFilter and NodeMarker
    /// BloomFilter is used for initial fast filtering, NodeMarker for precise tracking
    /// This is useful for scenarios where we want both performance and precision
    pub fn build_hybrid(
        &self,
        live_roots: Vec<H256>,
        workers: usize,
        marker: Box<dyn NodeMarker>,
    ) -> Result<u64> {
        let counter = Arc::new(std::sync::atomic::AtomicU64::new(0));

        live_roots
            .into_par_iter()
            .with_max_len(workers)
            .for_each(|root| {
                if let Err(e) = self.dfs_from_root_hybrid(root, &counter, marker.as_ref()) {
                    tracing::error!("DFS error with hybrid approach: {}", e);
                }
            });

        let scanned = counter.load(std::sync::atomic::Ordering::Relaxed);
        tracing::info!(
            "ReachableBuilder build_hybrid completed: {} nodes scanned, bloom + {} marker",
            scanned,
            marker.marker_type()
        );

        Ok(scanned)
    }

    /// Hybrid DFS using BloomFilter for initial filtering and NodeMarker for precise tracking
    fn dfs_from_root_hybrid(
        &self,
        root_hash: H256,
        counter: &Arc<std::sync::atomic::AtomicU64>,
        marker: &dyn NodeMarker,
    ) -> Result<()> {
        use std::sync::atomic;
        let mut stack = VecDeque::new();
        stack.push_back(root_hash);
        let mut curr_count: u32 = 0;

        while let Some(node_hash) = stack.pop_back() {
            // First check bloom filter for fast deduplication
            {
                let mut bloom_guard = self.bloom.lock();
                if bloom_guard.contains(&node_hash) {
                    continue;
                }
                bloom_guard.insert(&node_hash);
            }

            // Then mark using NodeMarker for precise tracking
            let newly_marked = marker.mark(node_hash)?;
            if !newly_marked {
                // This can happen if the node was already marked by another thread
                continue;
            }

            // Traverse the node to find children
            if let Some(bytes) = self.moveos_store.node_store.get(&node_hash)? {
                // If this leaf embeds another table root, push it to the stack for further traversal.
                if let Some(child_root) = try_extract_child_root(&bytes) {
                    stack.push_back(child_root);
                } else if let Ok(Node::Internal(internal)) = Node::<H256, Vec<u8>>::decode(&bytes) {
                    // For internal nodes, add all children to the stack
                    for child_hash in internal.all_child() {
                        stack.push_back(child_hash.into());
                    }
                }
            }

            curr_count += 1;
            counter.fetch_add(1, atomic::Ordering::Relaxed);

            // Log progress every 10,000 nodes
            if curr_count % 10000 == 0 {
                info!(
                    "ReachableBuilder dfs_from_root_hybrid looping, curr_count {}",
                    curr_count
                );
            }
        }

        info!(
            "ReachableBuilder dfs_from_root_hybrid done, root_hash {:?}, recursive child size {}, total marked: {}",
            root_hash,
            curr_count,
            marker.marked_count()
        );
        Ok(())
    }
}
