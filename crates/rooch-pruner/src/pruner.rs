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
        if !cfg.enable {
            return Ok(Self {
                handle: None,
                running: Arc::new(AtomicBool::new(false)),
            });
        }

        let running = Arc::new(AtomicBool::new(true));
        let thread_running = running.clone();

        let handle = thread::spawn(move || {
            let bloom = moveos_store
                .load_prune_meta_bloom()
                .ok()
                .and_then(|opt| opt.map(|bf| Arc::new(Mutex::new(bf))))
                .unwrap_or(Arc::new(Mutex::new(BloomFilter::new(
                    cfg.bloom_bits as usize,
                    4,
                ))));

            loop {
                if !thread_running.load(Ordering::Relaxed) {
                    break;
                }

                // load current phase
                let phase = moveos_store
                    .load_prune_meta_phase()
                    .unwrap_or(PrunePhase::BuildReach);

                match phase {
                    PrunePhase::BuildReach => {
                        // Determine current live root via StartupInfo
                        let live_roots = moveos_store
                            .get_startup_info()
                            .ok()
                            .and_then(|opt| opt.map(|info| vec![info.state_root]))
                            .unwrap_or_default();
                        let builder = ReachableBuilder::new(moveos_store.clone(), bloom.clone());
                        let _ = builder.build(live_roots, num_cpus::get());
                        // Persist bloom snapshot after reachability phase
                        {
                            let _ = moveos_store.save_prune_meta_bloom(bloom.lock().clone());
                        }
                        moveos_store
                            .save_prune_meta_phase(PrunePhase::SweepExpired)
                            .ok();
                    }
                    PrunePhase::SweepExpired => {
                        // NOTE: Temporarily disable time-window logic, fall back to zero root; will be restored later
                        // let config_store = ConfigDBStore::new(node_store.as_ref().get_store().store().clone());
                        // let cutoff_root = config_store
                        //     .get_startup_info()
                        //     .ok()
                        //     .and_then(|opt| opt.map(|info| info.state_root))
                        //     .unw rap_or_else(H256::zero);

                        // Calculate cutoff order based on latest sequenced tx order and `keep_tx` window
                        let latest_order = rooch_store
                            .get_sequencer_info()
                            .ok()
                            .and_then(|opt| opt.map(|info| info.last_order))
                            .unwrap_or(0);

                        // Collect at most `scan_batch` roots starting from the oldest end (latest_order descending)
                        let mut expired_roots: Vec<H256> = Vec::with_capacity(cfg.scan_batch);
                        // Start from latest order and skip the first 30k orders to avoid scanning too many roots
                        let mut order_cursor = if latest_order > 30000 {
                            latest_order - 30000
                        } else if latest_order >= 1 {
                            latest_order - 1
                        } else {
                            latest_order
                        };
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

                        let sweeper = SweepExpired::new(moveos_store.clone(), bloom.clone());
                        let _ = sweeper.sweep(expired_roots, num_cpus::get());
                        // Persist bloom snapshot after sweep phase (in case items added)
                        {
                            let _bytes = bloom.lock().to_bytes();
                            let _ = moveos_store.save_prune_meta_bloom(bloom.lock().clone());
                        }
                        // Instead of entering Incremental phase, jump back to BuildReach so the sweep can repeat and free disk continuously
                        // save_phase(&meta_store, PrunePhase::BuildReach).ok();
                        moveos_store
                            .save_prune_meta_phase(PrunePhase::BuildReach)
                            .ok();
                    }
                    PrunePhase::Incremental => {
                        // Incremental phase temporarily disabled; just store BuildReach to be scheduled in next loop
                        moveos_store
                            .save_prune_meta_phase(PrunePhase::BuildReach)
                            .ok();
                    }
                }

                thread::sleep(Duration::from_secs(cfg.interval_s as u64));
            }
        });

        Ok(Self {
            handle: Some(handle),
            running,
        })
    }

    pub fn stop(self) {
        if let Some(h) = self.handle {
            self.running.store(false, Ordering::Relaxed);
            let _ = h.join();
        }
    }
}
