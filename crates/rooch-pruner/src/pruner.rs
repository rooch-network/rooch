// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::reachability::ReachableBuilder;
use crate::sweep_expired::SweepExpired;
use anyhow::Result;
use moveos_common::bloom_filter::BloomFilter;
use moveos_store::config_store::ConfigStore;
use moveos_store::prune::PruneStore;
use moveos_store::state_store::metrics::StateDBMetrics;
use moveos_store::MoveOSStore;
use moveos_types::prune::{PrunePhase, PruneSnapshot};
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
use tokio::sync::broadcast::Receiver;
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
        mut shutdown_rx: Receiver<()>,
        metrics: Option<Arc<StateDBMetrics>>,
    ) -> Result<Self> {
        if !cfg.enable {
            return Ok(Self {
                handle: None,
                running: Arc::new(AtomicBool::new(false)),
            });
        }
        info!("Starting pruner");

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

            // Record bloom filter size metric
            if let Some(ref metrics) = metrics {
                let bloom_size_bytes = cfg.bloom_bits / 8 + (cfg.bloom_bits % 8 != 0) as usize;
                metrics.pruner_bloom_filter_size_bytes
                    .with_label_values(&[])
                    .set(bloom_size_bytes as f64);
            }
            thread::sleep(Duration::from_secs(60));
            // only for test
            // let mut phase = PrunePhase::BuildReach;
            loop {
                if !thread_running.load(Ordering::Relaxed) || shutdown_rx.try_recv().is_ok() {
                    info!("Pruner thread stopping");
                    break;
                }

                // load current phase
                let phase = moveos_store
                    .load_prune_meta_phase()
                    .unwrap_or(PrunePhase::BuildReach);
                // only for test
                // let phase = PrunePhase::BuildReach;
                info!("Current prune phase: {:?}", phase);

                // Record current phase metric
                if let Some(ref metrics) = metrics {
                    let phase_value = match phase {
                        PrunePhase::BuildReach => 0.0,
                        PrunePhase::SweepExpired => 1.0,
                        PrunePhase::Incremental => 2.0,
                    };
                    metrics.pruner_current_phase
                        .with_label_values(&[&format!("{:?}", phase)])
                        .set(phase_value);
                }

                match phase {
                    PrunePhase::BuildReach => {
                        info!("Starting BuildReach phase");
                        // Determine current live root via StartupInfo
                        let startup_info_state_root = moveos_store
                            .get_startup_info()
                            .ok()
                            .and_then(|opt| opt.map(|info| info.state_root))
                            .unwrap_or_default();
                        info!("Current startup state root: {:?}", startup_info_state_root);
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

                        let builder = ReachableBuilder::new(moveos_store.clone(), bloom.clone());
                        // if let Ok(scanned_size) =
                        let start_time = std::time::Instant::now();
                        match builder.build(live_roots, num_cpus::get()) {
                            Ok(scanned_size) => {
                                let duration = start_time.elapsed();
                                let nodes_per_sec = scanned_size as f64 / duration.as_secs_f64();

                                info!(
                                    "Completed reachability build, scanned size {} in {:?} ({:.2} nodes/sec)",
                                    scanned_size, duration, nodes_per_sec
                                );

                                // Record metrics
                                if let Some(ref metrics) = metrics {
                                    metrics.pruner_reachable_nodes_scanned
                                        .with_label_values(&["BuildReach"])
                                        .observe(scanned_size as f64);

                                    metrics.pruner_processing_speed_nodes_per_sec
                                        .with_label_values(&["reachability_build"])
                                        .observe(nodes_per_sec);
                                }
                            },
                            Err(e) => {
                                warn!("Failed to reachability build: {}", e);
                                if let Some(ref metrics) = metrics {
                                    metrics.pruner_error_count
                                        .with_label_values(&["reachability_build", "BuildReach"])
                                        .inc();
                                }
                            },
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
                        // only for test
                        // phase = PrunePhase::SweepExpired;
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

                      let sweeper = SweepExpired::new(
                            moveos_store.clone(),
                            bloom.clone(),
                            cfg.bloom_bits,
                            thread_running.clone(),
                        );
                        let sweep_start_time = std::time::Instant::now();
                        let mut processed_count = 0;
                        let mut total_deleted = 0;
                        let mut batch_roots = Vec::with_capacity(1000); // Process in smaller batches

                        // while processed_count < cfg.scan_batch && order_cursor > 0 {
                        while order_cursor > 0 {
                            // Check exit signal frequently
                            if !thread_running.load(Ordering::Relaxed)
                                || shutdown_rx.try_recv().is_ok()
                            {
                                info!("Pruner thread stopping during sweep");
                                return;
                            }

                            if let Some(scs) = rooch_store
                                .get_state_change_set(order_cursor)
                                .ok()
                                .flatten()
                            {
                                // Store both state_root and tx_order for traceability
                                batch_roots.push((scs.state_change_set.state_root, order_cursor));

                                // Process in smaller batches to avoid memory pressure
                                if batch_roots.len() >= 1000 {
                                    if !thread_running.load(Ordering::Relaxed)
                                        || shutdown_rx.try_recv().is_ok()
                                    {
                                        info!("Pruner thread stopping before batch sweep");
                                        return;
                                    }
                                    if let Ok(deleted) =
                                        sweeper.sweep(batch_roots.clone(), num_cpus::get())
                                    {
                                    total_deleted += deleted;
                                        let (from_state_root, from_tx_order) = batch_roots
                                            .first()
                                            .map(|(root, order)| (*root, *order))
                                            .unwrap_or((H256::zero(), 0));
                                        let (to_state_root, to_tx_order) = batch_roots
                                            .last()
                                            .map(|(root, order)| (*root, *order))
                                            .unwrap_or((H256::zero(), 0));
                                        info!("Pruner swept batch from tx_order {} (root {:?}) to tx_order {} (root {:?}), deleted {} nodes, total processed {} batches",
                                            from_tx_order, from_state_root, to_tx_order, to_state_root, deleted, processed_count);

                                        // Record metrics for this batch
                                        if let Some(ref metrics) = metrics {
                                            metrics.pruner_sweep_nodes_deleted
                                                .with_label_values(&["SweepExpired"])
                                                .observe(deleted as f64);

                                            // Estimate disk space reclaimed (assuming average node size of 32 bytes)
                                            let estimated_bytes_reclaimed = deleted * 32;
                                            metrics.pruner_disk_space_reclaimed_bytes
                                                .with_label_values(&["SweepExpired"])
                                                .inc_by(estimated_bytes_reclaimed as f64);
                                        }
                                    } else {
                                        if let Some(ref metrics) = metrics {
                                            metrics.pruner_error_count
                                                .with_label_values(&["sweep_batch", "SweepExpired"])
                                                .inc();
                                        }
                                    }
                                    processed_count += 1000;
                                    batch_roots = Vec::with_capacity(1000);
                                }
                            }
                            order_cursor -= 1;
                        }

                        // Process any remaining roots
                        if !batch_roots.is_empty() {
                            if let Ok(deleted) = sweeper.sweep(batch_roots.clone(), num_cpus::get())
                            {
                              total_deleted += deleted;
                                let (from_state_root, from_tx_order) = batch_roots
                                    .first()
                                    .map(|(root, order)| (*root, *order))
                                    .unwrap_or((H256::zero(), 0));
                                let (to_state_root, to_tx_order) = batch_roots
                                    .last()
                                    .map(|(root, order)| (*root, *order))
                                    .unwrap_or((H256::zero(), 0));
                                info!(
                                    "Pruner swept final batch from tx_order {} (root {:?}) to tx_order {} (root {:?}), deleted {} nodes",
                                    from_tx_order, from_state_root, to_tx_order, to_state_root, deleted
                                );

                                // Record metrics for final batch
                                if let Some(ref metrics) = metrics {
                                    metrics.pruner_sweep_nodes_deleted
                                        .with_label_values(&["SweepExpired"])
                                        .observe(deleted as f64);

                                    let estimated_bytes_reclaimed = deleted * 32;
                                    metrics.pruner_disk_space_reclaimed_bytes
                                        .with_label_values(&["SweepExpired"])
                                        .inc_by(estimated_bytes_reclaimed as f64);
                                }
                            } else {
                                if let Some(ref metrics) = metrics {
                                    metrics.pruner_error_count
                                        .with_label_values(&["sweep_final", "SweepExpired"])
                                        .inc();
                                }
                            }
                        }

                        let sweep_duration = sweep_start_time.elapsed();
                        let total_nodes_per_sec = if sweep_duration.as_secs() > 0 {
                            total_deleted as f64 / sweep_duration.as_secs_f64()
                        } else {
                            0.0
                        };

                        info!(
                            "Completed expired roots sweep, processed {} roots, deleted {} nodes in {:?} ({:.2} nodes/sec)",
                            processed_count, total_deleted, sweep_duration, total_nodes_per_sec
                        );

                        // Record final sweep metrics
                        if let Some(ref metrics) = metrics {
                            metrics.pruner_sweep_nodes_deleted
                                .with_label_values(&["SweepExpired_Total"])
                                .observe(total_deleted as f64);

                            metrics.pruner_processing_speed_nodes_per_sec
                                .with_label_values(&["sweep_expired_total"])
                                .observe(total_nodes_per_sec);
                        }

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
                        // only for test
                        // phase = PrunePhase::Incremental;
                        info!("Transitioning back to Incremental phase");
                    }
                    PrunePhase::Incremental => {
                        info!("Incremental phase disabled, do Nothing");
                        // moveos_store
                        //     .save_prune_meta_phase(PrunePhase::BuildReach)
                        //     .ok();
                        // Check exit signal frequently
                        if !thread_running.load(Ordering::Relaxed) || shutdown_rx.try_recv().is_ok()
                        {
                            info!("Pruner thread stopping during sweep");
                            return;
                        }
                    }
                }

                info!("Sleeping for {} seconds", cfg.interval_s);
                // Sleep in small intervals to respond to exit signal quickly
                let mut slept = 0;
                while slept < cfg.interval_s {
                    if !thread_running.load(Ordering::Relaxed) || shutdown_rx.try_recv().is_ok() {
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
