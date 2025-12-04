// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::marker::NodeMarker;
use crate::util::extract_child_nodes;
use anyhow::Result;
use crossbeam_deque::{Injector, Steal, Stealer, Worker};
use moveos_common::bloom_filter::BloomFilter;
use moveos_store::MoveOSStore;
use parking_lot::Mutex;
use primitive_types::H256;
use rayon::prelude::*;
use smt::NodeReader;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use tracing::info;

/// Build reachable set by parallel DFS over live roots.
///
/// Design notes:
/// * A global BloomFilter is used for hot-path deduplication in the legacy `build()` method.
/// * For GC operations, use `build_with_marker()` or `build_with_marker_parallel()` with `NodeMarker` for precise tracking.
pub struct ReachableBuilder {
    moveos_store: MoveOSStore,
    bloom: Arc<Mutex<BloomFilter>>,
    // metrics: Arc<StateDBMetrics>,
}

impl ReachableBuilder {
    pub fn new(moveos_store: MoveOSStore, bloom: Arc<Mutex<BloomFilter>>) -> Self {
        Self {
            moveos_store,
            bloom,
        }
    }

    /// Build reachable set starting from `live_roots`.
    /// Returns number of unique reachable nodes scanned.
    pub fn build(&self, live_roots: Vec<H256>, workers: usize) -> Result<u64> {
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

            // Traverse children and leaf add to stack
            // Include both Global‐State and Table-State JMT
            if let Some(bytes) = self.moveos_store.node_store.get(&node_hash)? {
                for child in extract_child_nodes(&bytes) {
                    stack.push_back(child);
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
    /// Uses single-threaded batch processing for optimal I/O performance.
    /// For multi-threaded execution, use `build_with_marker_parallel` instead.
    /// Returns number of unique reachable nodes scanned.
    pub fn build_with_marker(
        &self,
        live_roots: Vec<H256>,
        marker: &dyn NodeMarker,
        batch_size: usize,
    ) -> Result<u64> {
        // Validate batch_size to prevent infinite loop or excessive memory allocation
        const MAX_BATCH_SIZE: usize = 100_000;
        anyhow::ensure!(batch_size > 0, "batch_size must be positive");
        anyhow::ensure!(
            batch_size <= MAX_BATCH_SIZE,
            "batch_size {} exceeds maximum allowed size {}",
            batch_size,
            MAX_BATCH_SIZE
        );

        let counter = Arc::new(std::sync::atomic::AtomicU64::new(0));

        // For now, use single-threaded batch processing with optimized I/O
        // Future enhancement: implement full work-stealing parallelism
        for root in live_roots {
            if let Err(e) = self.dfs_from_root_with_marker_batch(root, &counter, marker, batch_size)
            {
                tracing::error!("DFS error with marker: {}", e);
            }
        }

        let scanned = counter.load(std::sync::atomic::Ordering::Relaxed);
        tracing::info!(
            "ReachableBuilder build_with_marker completed: {} nodes scanned, marker type: {}",
            scanned,
            marker.marker_type()
        );

        Ok(scanned)
    }

    /// Optimized single-threaded DFS with batch I/O processing
    fn dfs_from_root_with_marker_batch(
        &self,
        root_hash: H256,
        counter: &Arc<std::sync::atomic::AtomicU64>,
        marker: &dyn NodeMarker,
        batch_size: usize,
    ) -> Result<()> {
        let mut stack = VecDeque::new();
        stack.push_back(root_hash);
        let mut curr_count: u32 = 0;

        while !stack.is_empty() {
            // Collect up to batch_size nodes for batch processing
            let mut pending_nodes = Vec::with_capacity(batch_size);
            while pending_nodes.len() < batch_size && !stack.is_empty() {
                let node_hash = stack.pop_back().unwrap();

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

                pending_nodes.push(node_hash);
            }

            if pending_nodes.is_empty() {
                continue;
            }

            // Batch read all pending nodes
            let bytes_results = self.moveos_store.node_store.multi_get(&pending_nodes)?;

            // Process each node and extract children
            for bytes_option in bytes_results.into_iter() {
                // Traverse the node to find children
                // Include both Global‐State and Table-State JMT nodes
                if let Some(bytes) = bytes_option {
                    for child in extract_child_nodes(&bytes) {
                        stack.push_back(child);
                    }
                }

                curr_count += 1;
                counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }

            // Log progress every 10,000 nodes
            if curr_count % 10000 == 0 {
                info!(
                    "ReachableBuilder dfs_from_root_with_marker_batch looping, curr_count {}, marker: {}",
                    curr_count,
                    marker.marker_type()
                );
            }
        }

        info!(
            "ReachableBuilder dfs_from_root_with_marker_batch done, root_hash {:?}, recursive child size {}, total marked: {}",
            root_hash, curr_count, marker.marked_count()
        );
        Ok(())
    }

    /// DFS traversal using NodeMarker for precise marking instead of BloomFilter
    /// This method is designed for garbage collection where we need exact tracking

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
                for child in extract_child_nodes(&bytes) {
                    stack.push_back(child);
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

    /// Parallel build with work-stealing for better performance on multi-core systems
    /// Optimized for both single-root and multi-root scenarios
    pub fn build_with_marker_parallel(
        &self,
        live_roots: Vec<H256>,
        workers: usize,
        marker: &dyn NodeMarker,
        batch_size: usize,
    ) -> Result<u64> {
        // Validate batch_size to prevent infinite loop or excessive memory allocation
        const MAX_BATCH_SIZE: usize = 100_000;
        anyhow::ensure!(batch_size > 0, "batch_size must be positive");
        anyhow::ensure!(
            batch_size <= MAX_BATCH_SIZE,
            "batch_size {} exceeds maximum allowed size {}",
            batch_size,
            MAX_BATCH_SIZE
        );

        // Fallback to single-threaded for small worker counts
        if workers <= 1 {
            return self.build_with_marker(live_roots, marker, batch_size);
        }

        info!(
            "Starting parallel reachable build with {} workers, batch_size={}",
            workers, batch_size
        );

        // Global work injector for initial distribution and overflow
        let global_injector = Arc::new(Injector::new());
        for root in live_roots {
            global_injector.push(root);
        }

        // Create local worker queues and stealers
        let local_queues: Vec<Worker<H256>> = (0..workers).map(|_| Worker::new_fifo()).collect();
        let stealers: Vec<Stealer<H256>> = local_queues.iter().map(|w| w.stealer()).collect();

        // Termination detection
        let active_workers = Arc::new(AtomicUsize::new(workers));
        let termination_flag = Arc::new(AtomicBool::new(false));

        // Launch worker threads using scoped threads for lifetime safety
        thread::scope(|s| {
            for (worker_id, local_queue) in local_queues.into_iter().enumerate() {
                let global = Arc::clone(&global_injector);
                let stealers_ref = &stealers;
                let active = Arc::clone(&active_workers);
                let term_flag = Arc::clone(&termination_flag);

                s.spawn(move || {
                    if let Err(e) = self.worker_loop(
                        worker_id,
                        local_queue,
                        &global,
                        stealers_ref,
                        marker,
                        batch_size,
                        &active,
                        &term_flag,
                    ) {
                        tracing::error!("Worker {} error: {}", worker_id, e);
                    }
                });
            }
        });

        let total_marked = marker.marked_count();
        info!(
            "Parallel reachable build completed: {} nodes marked",
            total_marked
        );
        Ok(total_marked)
    }

    /// Worker loop for parallel traversal with work stealing
    #[allow(clippy::too_many_arguments)]
    fn worker_loop(
        &self,
        worker_id: usize,
        local: Worker<H256>,
        global: &Injector<H256>,
        stealers: &[Stealer<H256>],
        marker: &dyn NodeMarker,
        batch_size: usize,
        active_workers: &AtomicUsize,
        termination_flag: &AtomicBool,
    ) -> Result<()> {
        let mut pending = Vec::with_capacity(batch_size);
        // Overflow threshold: push to global when local queue gets too large
        // This is critical for single-root scenarios to enable work sharing
        // Smaller threshold to encourage earlier sharing; keep a floor to avoid thrashing.
        let local_overflow_threshold = std::cmp::max(256, batch_size / 2);
        let mut local_queue_estimate = 0usize;
        let mut processed_count = 0u64;
        let mut idle_cycles = 0u64;

        info!("Worker {} started", worker_id);

        loop {
            // Check termination flag
            if termination_flag.load(Ordering::Relaxed) {
                break;
            }

            // 1. Try to fill pending batch from local queue
            while pending.len() < batch_size {
                match local.pop() {
                    Some(hash) => {
                        if !marker.is_marked(&hash) {
                            pending.push(hash);
                        }
                        local_queue_estimate = local_queue_estimate.saturating_sub(1);
                    }
                    None => break,
                }
            }

            // 2. If local queue is empty, try to steal work
            if pending.is_empty() {
                let stolen = self.steal_batch(global, stealers, worker_id, batch_size / 2);
                if stolen.is_empty() {
                    // No work available, enter idle state
                    idle_cycles += 1;

                    // Mark as idle
                    let prev_active = active_workers.fetch_sub(1, Ordering::SeqCst);

                    // Wait for work with timeout-based termination detection
                    let mut found_work = false;
                    let start_wait = std::time::Instant::now();
                    let max_wait = std::time::Duration::from_secs(5); // Wait up to 5 seconds

                    while start_wait.elapsed() < max_wait {
                        if termination_flag.load(Ordering::Relaxed) {
                            break;
                        }

                        // Try to steal work
                        let retry_stolen = self.steal_batch(global, stealers, worker_id, 1);
                        if !retry_stolen.is_empty() {
                            // Found work, reactivate
                            active_workers.fetch_add(1, Ordering::SeqCst);
                            for hash in retry_stolen {
                                if !marker.is_marked(&hash) {
                                    pending.push(hash);
                                }
                            }
                            found_work = true;
                            break;
                        }

                        // Check termination condition: all workers idle and no work in global queue
                        let current_active = active_workers.load(Ordering::SeqCst);
                        if current_active == 0 {
                            // All workers are idle, check if global queue is empty
                            match global.steal() {
                                Steal::Empty => {
                                    // Double check - give other workers a chance to add work
                                    std::thread::sleep(std::time::Duration::from_millis(10));
                                    if active_workers.load(Ordering::SeqCst) == 0 {
                                        match global.steal() {
                                            Steal::Empty => {
                                                // Truly empty, signal termination
                                                termination_flag.store(true, Ordering::SeqCst);
                                                break;
                                            }
                                            Steal::Success(hash) => {
                                                active_workers.fetch_add(1, Ordering::SeqCst);
                                                if !marker.is_marked(&hash) {
                                                    pending.push(hash);
                                                }
                                                found_work = true;
                                                break;
                                            }
                                            Steal::Retry => continue,
                                        }
                                    }
                                }
                                Steal::Success(hash) => {
                                    active_workers.fetch_add(1, Ordering::SeqCst);
                                    if !marker.is_marked(&hash) {
                                        pending.push(hash);
                                    }
                                    found_work = true;
                                    break;
                                }
                                Steal::Retry => {}
                            }
                        }

                        // Brief sleep to avoid busy-waiting
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }

                    if !found_work {
                        // Timeout reached without finding work
                        if prev_active == 1 || active_workers.load(Ordering::SeqCst) == 0 {
                            // We might be the last worker, signal termination
                            termination_flag.store(true, Ordering::SeqCst);
                        }
                        break;
                    }
                } else {
                    for hash in stolen {
                        if !marker.is_marked(&hash) {
                            pending.push(hash);
                        }
                    }
                }

                if pending.is_empty() {
                    continue;
                }
            }

            // Reset idle cycles when we have work
            idle_cycles = 0;

            // 3. Process batch and distribute children
            self.process_batch_parallel(
                &pending,
                &local,
                global,
                marker,
                local_overflow_threshold,
                &mut local_queue_estimate,
            )?;

            processed_count += pending.len() as u64;
            pending.clear();

            // Log progress periodically (every 100K nodes for less noise)
            if processed_count % 100000 == 0 && processed_count > 0 {
                info!(
                    "Worker {}: processed {} nodes, local_queue_estimate={}",
                    worker_id, processed_count, local_queue_estimate
                );
            }
        }

        info!(
            "Worker {} completed: processed {} nodes total, idle_cycles={}",
            worker_id, processed_count, idle_cycles
        );
        Ok(())
    }

    /// Batch processing with smart child distribution for work sharing
    fn process_batch_parallel(
        &self,
        pending: &[H256],
        local: &Worker<H256>,
        global: &Injector<H256>,
        marker: &dyn NodeMarker,
        overflow_threshold: usize,
        local_queue_estimate: &mut usize,
    ) -> Result<()> {
        // Batch read all pending nodes
        let bytes_results = self.moveos_store.node_store.multi_get(pending)?;

        // Process each node and extract children
        for (i, bytes_option) in bytes_results.into_iter().enumerate() {
            let node_hash = pending[i];

            // Mark the node
            let newly_marked = marker.mark(node_hash)?;
            if !newly_marked {
                continue;
            }

            // Extract and distribute children
            if let Some(bytes) = bytes_option {
                let children = self.extract_children(&bytes);

                for child in children {
                    // Smart distribution: overflow to global queue when local is full
                    // This enables other workers to steal work, crucial for single-root scenarios
                    if *local_queue_estimate >= overflow_threshold {
                        global.push(child);
                    } else {
                        local.push(child);
                        *local_queue_estimate += 1;
                    }
                }
            }
        }
        Ok(())
    }

    /// Extract child nodes from a serialized SMT node
    fn extract_children(&self, bytes: &[u8]) -> Vec<H256> {
        extract_child_nodes(bytes)
    }

    /// Batch stealing from global and other workers
    fn steal_batch(
        &self,
        global: &Injector<H256>,
        stealers: &[Stealer<H256>],
        self_id: usize,
        max_steal: usize,
    ) -> Vec<H256> {
        let mut stolen = Vec::with_capacity(max_steal);

        // First, try to steal from global injector (higher priority)
        for _ in 0..max_steal {
            match global.steal() {
                Steal::Success(hash) => stolen.push(hash),
                Steal::Empty => break,
                Steal::Retry => continue,
            }
        }

        if !stolen.is_empty() {
            return stolen;
        }

        // Then try to steal from other workers
        for (i, stealer) in stealers.iter().enumerate() {
            if i == self_id {
                continue; // Skip self
            }

            for _ in 0..(max_steal - stolen.len()) {
                match stealer.steal() {
                    Steal::Success(hash) => stolen.push(hash),
                    Steal::Empty => break,
                    Steal::Retry => continue,
                }
            }

            if stolen.len() >= max_steal {
                break;
            }
        }

        stolen
    }
}
