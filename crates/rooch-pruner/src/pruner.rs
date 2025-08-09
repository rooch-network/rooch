// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::reachability::ReachableBuilder;
use crate::sweep_expired::SweepExpired;
use anyhow::Result;
use moveos_common::bloom_filter::BloomFilter;
use moveos_store::config_store::ConfigStore;
use moveos_store::prune::PruneStore;
use moveos_store::MoveOSStore;
use moveos_types::prune::PrunePhase;
use parking_lot::Mutex;
pub use rooch_config::prune_config::PruneConfig;
use rooch_store::RoochStore;
// Bring trait methods into scope for method-call syntax
use primitive_types::H256;
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

            loop {
                if !thread_running.load(Ordering::Relaxed) {
                    info!("Pruner thread stopping");
                    break;
                }

                // load current phase
                let phase = moveos_store
                    .load_prune_meta_phase()
                    .unwrap_or(PrunePhase::BuildReach);
                info!("Current prune phase: {:?}", phase);

                match phase {
                    PrunePhase::BuildReach => {
                        info!("Starting BuildReach phase");
                        // Determine current live root via StartupInfo
                        let live_roots = moveos_store
                            .get_startup_info()
                            .ok()
                            .and_then(|opt| opt.map(|info| vec![info.state_root]))
                            .unwrap_or_default();
                        info!("Found {} live roots", live_roots.len());

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
                        let latest_order = rooch_store
                            .get_sequencer_info()
                            .ok()
                            .and_then(|opt| opt.map(|info| info.last_order))
                            .unwrap_or(0);
                        info!("Latest sequencer order: {}", latest_order);

                        // Collect at most `scan_batch` roots starting from the oldest end
                        let mut expired_roots: Vec<H256> = Vec::with_capacity(cfg.scan_batch);
                        let mut order_cursor = if latest_order > 30000 {
                            latest_order - 30000
                        } else if latest_order >= 1 {
                            latest_order - 1
                        } else {
                            latest_order
                        };
                        info!("Starting scan from order {}", order_cursor);

                        while expired_roots.len() < cfg.scan_batch && order_cursor > 0 {
                            if let Some(scs) = rooch_store
                                .get_state_change_set(order_cursor)
                                .ok()
                                .flatten()
                            {
                                expired_roots.push(scs.state_change_set.state_root);
                            }
                            order_cursor -= 1;
                        }
                        info!("Found {} expired roots to sweep", expired_roots.len());

                        let sweeper = SweepExpired::new(moveos_store.clone(), bloom.clone());
                        let _ = sweeper.sweep(expired_roots, num_cpus::get());
                        info!("Completed expired roots sweep");

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
                            .save_prune_meta_phase(PrunePhase::BuildReach)
                            .ok();
                        info!("Transitioning back to BuildReach phase");
                    }
                    PrunePhase::Incremental => {
                        info!("Incremental phase disabled, transitioning to BuildReach");
                        moveos_store
                            .save_prune_meta_phase(PrunePhase::BuildReach)
                            .ok();
                    }
                }

                info!("Sleeping for {} seconds", cfg.interval_s);
                thread::sleep(Duration::from_secs(cfg.interval_s));
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
