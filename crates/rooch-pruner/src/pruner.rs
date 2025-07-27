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
use rooch_config::prune_config::PruneConfig;
use rooch_store::RoochStore;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;
// incremental_sweep not currently used but may be enabled later

// #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
// pub enum PrunePhase {
//     BuildReach,
//     SweepExpired,
//     Incremental,
// }

// const META_KEY_PHASE: &str = "phase";
// const META_KEY_CURSOR: &str = "cursor"; // placeholder for future use
// const META_KEY_BLOOM: &str = "bloom_snapshot";

// fn load_phase(meta: &PruneMetaStore) -> Result<PrunePhase> {
//     if let Some(bytes) = meta.kv_get(META_KEY_PHASE.to_string())? {
//         Ok(bcs::from_bytes::<PrunePhase>(&bytes)?)
//     } else {
//         Ok(PrunePhase::BuildReach)
//     }
// }
//
// fn save_phase(meta: &PruneMetaStore, phase: PrunePhase) -> Result<()> {
//     meta.kv_put(META_KEY_PHASE.to_string(), bcs::to_bytes(&phase)?)
// }

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
            // let reach_seen_store = Some(Arc::new(moveos_store.get_prune_store());
            // let prune_meta_store = Arc::new(moveos_store.get_prune_meta_store());
            // let node_store = Arc::new(moveos_store.get_state_node_store().clone());
            // Load bloom snapshot if exists
            // let bloom = if let Ok(Some(bytes)) = meta_store.kv_get(META_KEY_BLOOM.to_string()) {
            // let bloom = if let Ok(bloom_opt) = moveos_store.load_prune_meta_bloom() {
            //     match bloom_opt {
            //         Some(bf) => Arc::new(Mutex::new(bf)),
            //         None => Arc::new(Mutex::new(BloomFilter::new(cfg.bloom_bits as usize, 4))),
            //     }
            // } else {
            //     Arc::new(Mutex::new(BloomFilter::new(cfg.bloom_bits as usize, 4)))
            // };

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
                        // let config_store =
                        //     ConfigDBStore::new(node_store.get_store().store().clone());
                        let live_roots = moveos_store
                            .get_startup_info()
                            .ok()
                            .and_then(|opt| opt.map(|info| vec![info.state_root]))
                            .unwrap_or_default();
                        let builder = ReachableBuilder::new(
                            moveos_store.clone(),
                            // reach_seen.clone(),
                            bloom.clone(),
                            // metrics.clone(),
                        );
                        let _ = builder.build(live_roots, num_cpus::get());
                        // Persist bloom snapshot after reachability phase
                        {
                            // let bytes = bloom.lock().to_bytes();
                            // let _ = meta_store.kv_put(META_KEY_BLOOM.to_string(), bytes);
                            let _ = moveos_store.save_prune_meta_bloom(bloom.lock().clone());
                        }
                        // save_phase(&meta_store, PrunePhase::SweepExpired).ok();
                        moveos_store
                            .save_prune_meta_phase(PrunePhase::SweepExpired)
                            .ok();
                    }
                    PrunePhase::SweepExpired => {
                        // let stale_store =
                        //     StaleIndexStore::new(node_store.get_store().store().clone());

                        // NOTE: Temporarily disable time-window logic, fall back to zero root; will be restored later
                        // let config_store = ConfigDBStore::new(node_store.as_ref().get_store().store().clone());
                        // let cutoff_root = config_store
                        //     .get_startup_info()
                        //     .ok()
                        //     .and_then(|opt| opt.map(|info| info.state_root))
                        //     .unwrap_or_else(H256::zero);

                        // let cutoff_order = latest_order.saturating_sub(cfg.keep_tx); // 或按时间换算
                        let expired_roots =
                            scs_store.list_roots_before(cutoff_order, cfg.scan_batch)?;

                        let sweeper = SweepExpired::new(
                            // node_store.clone(),
                            // reach_seen.clone(),
                            moveos_store.clone(),
                            bloom.clone(),
                            // metrics.clone(),
                        );
                        let _ = sweeper.sweep(expired_roots, num_cpus::get());
                        // Persist bloom snapshot after sweep phase (in case items added)
                        {
                            let bytes = bloom.lock().to_bytes();
                            // let _ = meta_store.kv_put(META_KEY_BLOOM.to_string(), bytes);
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
                        // save_phase(&meta_store, PrunePhase::BuildReach).ok();
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
