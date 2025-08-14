// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::reachability::ReachableBuilder;
use crate::sweep_expired::SweepExpired;
use anyhow::Result;
use moveos_common::bloom_filter::BloomFilter;
use moveos_store::config_store::ConfigStore;
use moveos_store::prune::PruneStore;
use moveos_store::MoveOSStore;
use moveos_types::prune::{PrunePhase, PruneSnapshot};
use parking_lot::Mutex;
pub use rooch_config::prune_config::PruneConfig;
use rooch_store::RoochStore;
// Bring trait methods into scope for method-call syntax

use rooch_store::meta_store::MetaStore;
use rooch_store::state_store::StateStore;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;
// incremental_sweep not currently used but may be enabled later
use tracing::{info, warn};

pub struct StatePruner {
    handle: Option<JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

impl StatePruner {
    pub fn start(
        cfg: Arc<PruneConfig>,
        moveos_store: Arc<MoveOSStore>,
        rooch_store: Arc<RoochStore>,
        // metrics: Arc<StateDBMetrics>,
    ) -> Result<Self> {
        info!("Starting pruner");
        if !cfg.enable {
            return Ok(Self {
                handle: None,
                running: Arc::new(AtomicBool::new(false)),
            });
        }

        info!("Starting pruner with config: {:?}", cfg);
        let running = Arc::new(AtomicBool::new(true));
        let thread_running = running.clone();

        let handle = thread::spawn(move || {
            info!("Pruner thread started");
            let bloom = moveos_store
                .load_prune_meta_bloom()
                .ok()
                .and_then(|opt| opt.map(|bf| Arc::new(Mutex::new(bf))))
                .unwrap_or(Arc::new(Mutex::new(BloomFilter::new(cfg.bloom_bits, 4))));
            info!("Loaded bloom filter with {} bits", cfg.bloom_bits);

            let mut phase = PrunePhase::BuildReach;
            loop {
                if !thread_running.load(Ordering::Relaxed) {
                    info!("Pruner thread stopping");
                    break;
                }

                // load current phase
                // let phase = moveos_store
                //     .load_prune_meta_phase()
                //     .unwrap_or(PrunePhase::BuildReach);
                // let phase = PrunePhase::BuildReach;
                info!("Current prune phase: {:?}", phase);

                match phase {
                    PrunePhase::BuildReach => {
                        info!("Starting BuildReach phase");
                        // Determine current live root via StartupInfo
                        let startup_info_state_root = moveos_store
                            .get_startup_info()
                            .ok()
                            .and_then(|opt| opt.map(|info| info.state_root))
                            .unwrap_or_default();
                        info!("Current startup state root: {}", startup_info_state_root);
                        let latest_order = rooch_store
                            .get_sequencer_info()
                            .ok()
                            .and_then(|opt| opt.map(|info| info.last_order))
                            .unwrap_or(0);
                        info!("Current latest_order: {}", latest_order);
                        let live_roots = vec![startup_info_state_root];
                        info!("Found {} live roots", live_roots.len());

                        // after startup_info_state_root & latest_order are known
                        let snap = PruneSnapshot {
                            state_root: startup_info_state_root,
                            latest_order,
                        };
                        let _ = moveos_store.save_prune_meta_snapshot(snap);
                        moveos_store
                            .save_prune_meta_phase(PrunePhase::SweepExpired)
                            .ok();

                        let builder = ReachableBuilder::new(moveos_store.clone(), bloom.clone());
                        // if let Ok(scanned_size) =
                        match builder.build(live_roots, num_cpus::get()) {
                            Ok(scanned_size) => info!(
                                "Completed reachability build, scanned size {}",
                                scanned_size
                            ),
                            Err(e) => warn!("Failed to reachability build: {}", e),
                        }

                        // Persist bloom snapshot after reachability phase
                        {
                            if let Err(e) = moveos_store.save_prune_meta_bloom(bloom.lock().clone())
                            {
                                warn!("Failed to save bloom snapshot: {}", e);
                            } else {
                                info!("Saved bloom snapshot");
                            }
                        }
                        moveos_store
                            .save_prune_meta_phase(PrunePhase::SweepExpired)
                            .ok();
                        info!("Transitioning to SweepExpired phase");
                    }
                    PrunePhase::SweepExpired => {
                        info!("Starting SweepExpired phase");
                        // load snapshot taken during previous BuildReach
                        let snapshot = moveos_store
                            .load_prune_meta_snapshot()
                            .ok()
                            .flatten()
                            .unwrap_or_default();
                        info!("Latest prune snapshot: {:?}", snapshot);

                        // Stream process expired roots
                        let latest_order = snapshot.latest_order;
                        let mut order_cursor = if latest_order > 30000 {
                            latest_order - 30000
                        } else if latest_order >= 1 {
                            latest_order - 1
                        } else {
                            latest_order
                        };
                        info!("Starting scan from order {}", order_cursor);

                        let sweeper = SweepExpired::new(moveos_store.clone(), bloom.clone());
                        let mut processed_count = 0;
                        let mut batch_roots = Vec::with_capacity(1000); // Process in smaller batches

                        // while processed_count < cfg.scan_batch && order_cursor > 0 {
                        while order_cursor > 0 {
                            // Check exit signal frequently
                            if !thread_running.load(Ordering::Relaxed) {
                                info!("Pruner thread stopping during sweep");
                                return;
                            }

                            if let Some(scs) = rooch_store
                                .get_state_change_set(order_cursor)
                                .ok()
                                .flatten()
                            {
                                batch_roots.push(scs.state_change_set.state_root);

                                // Process in smaller batches to avoid memory pressure
                                if batch_roots.len() >= 1000 {
                                    if !thread_running.load(Ordering::Relaxed) {
                                        info!("Pruner thread stopping before batch sweep");
                                        return;
                                    }
                                    if let Ok(deleted) = sweeper.sweep(batch_roots, num_cpus::get())
                                    {
                                        info!("Swept batch of roots, this loop deleted {} nodes, total deleted {} nodes", deleted, processed_count);
                                    }
                                    processed_count += 1000;
                                    batch_roots = Vec::with_capacity(1000);
                                }
                            }
                            order_cursor -= 1;
                        }

                        // Process any remaining roots
                        if !batch_roots.is_empty() {
                            if let Ok(deleted) = sweeper.sweep(batch_roots, num_cpus::get()) {
                                info!(
                                    "Swept final batch of roots, total deleted {} nodes",
                                    deleted
                                );
                            }
                        }

                        info!(
                            "Completed expired roots sweep, processed {} roots",
                            processed_count
                        );

                        // Persist bloom snapshot after sweep phase
                        {
                            if let Err(e) = moveos_store.save_prune_meta_bloom(bloom.lock().clone())
                            {
                                warn!("Failed to save bloom snapshot after sweep: {}", e);
                            } else {
                                info!("Saved bloom snapshot after sweep");
                            }
                        }

                        moveos_store
                            .save_prune_meta_phase(PrunePhase::Incremental)
                            .ok();
                        info!("Transitioning back to Incremental phase");
                    }
                    PrunePhase::Incremental => {
                        info!("Incremental phase disabled, do Nothing");
                        // moveos_store
                        //     .save_prune_meta_phase(PrunePhase::BuildReach)
                        //     .ok();
                        // Check exit signal frequently
                        if !thread_running.load(Ordering::Relaxed) {
                            info!("Pruner thread stopping during sweep");
                            return;
                        }
                    }
                }
                // reload current phase
                phase = moveos_store
                    .load_prune_meta_phase()
                    .unwrap_or(PrunePhase::BuildReach);

                info!("Sleeping for {} seconds", cfg.interval_s);
                // Sleep in small intervals to respond to exit signal quickly
                let mut slept = 0;
                while slept < cfg.interval_s {
                    if !thread_running.load(Ordering::Relaxed) {
                        info!("Pruner thread stopping during sleep");
                        return;
                    }
                    thread::sleep(Duration::from_secs(1));
                    slept += 1;
                }
            }
        });

        Ok(Self {
            handle: Some(handle),
            running,
        })
    }

    pub fn stop(self) {
        if let Some(h) = self.handle {
            info!("Stopping pruner thread");
            self.running.store(false, Ordering::Relaxed);
            let _ = h.join();
            info!("Pruner thread stopped");
        }
    }
}
