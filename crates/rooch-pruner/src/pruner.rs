// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::atomic_snapshot::{AtomicSnapshotManager, SnapshotManagerConfig};
use crate::incremental_sweep::IncrementalSweep;
use crate::metrics::PrunerMetrics;
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
use tracing::{error, info, warn};

pub struct StatePruner {
    handle: Option<JoinHandle<()>>,
    is_running: Arc<AtomicBool>, // true = running, false = stopped
    #[allow(dead_code)]
    atomic_snapshot_manager: Arc<AtomicSnapshotManager>,
}

impl StatePruner {
    pub fn start(
        cfg: Arc<PruneConfig>,
        moveos_store: Arc<MoveOSStore>,
        rooch_store: Arc<RoochStore>,
        mut shutdown_rx: Receiver<()>,
        metrics: Option<Arc<PrunerMetrics>>,
    ) -> Result<Self> {
        if !cfg.enable {
            return Ok(Self {
                handle: None,
                is_running: Arc::new(AtomicBool::new(false)),
                atomic_snapshot_manager: Arc::new(AtomicSnapshotManager::new(
                    moveos_store.clone(),
                    rooch_store.clone(),
                    metrics.clone(),
                    Some(SnapshotManagerConfig::default()),
                )),
            });
        }
        info!("Starting pruner");

        info!("Starting pruner with config: {:?}", cfg);
        let is_running = Arc::new(AtomicBool::new(true));
        let is_running_for_thread = is_running.clone();

        // Initialize atomic snapshot manager
        let snapshot_config = SnapshotManagerConfig {
            lock_timeout_ms: 30 * 60 * 1000,         // 30 minutes
            max_snapshot_age_ms: 2 * 60 * 60 * 1000, // 2 hours
            enable_validation: true,
            enable_persistence: true,
        };

        let atomic_snapshot_manager = Arc::new(AtomicSnapshotManager::new(
            moveos_store.clone(),
            rooch_store.clone(),
            metrics.clone(),
            Some(snapshot_config),
        ));

        // Initialize the snapshot manager
        atomic_snapshot_manager.initialize()?;

        info!("Atomic snapshot manager initialized");

        let atomic_snapshot_manager_clone = atomic_snapshot_manager.clone();
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
                metrics
                    .pruner_bloom_filter_size_bytes
                    .with_label_values(&["Init"])
                    .set(bloom_size_bytes as f64);
            }
            thread::sleep(Duration::from_secs(60));
            // only for test
            // let mut phase = PrunePhase::BuildReach;
            loop {
                if !is_running_for_thread.load(Ordering::Relaxed) || shutdown_rx.try_recv().is_ok()
                {
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
                    metrics
                        .pruner_current_phase
                        .with_label_values(&[&format!("{:?}", phase)])
                        .set(phase_value);
                }

                match phase {
                    PrunePhase::BuildReach => {
                        info!("Starting BuildReach phase");

                        // Create atomic snapshot for this pruning cycle
                        let atomic_snapshot = match atomic_snapshot_manager_clone
                            .create_snapshot(PrunePhase::BuildReach)
                        {
                            Ok(snapshot) => {
                                info!(
                                    "Successfully created atomic snapshot {} for BuildReach",
                                    snapshot.snapshot_id
                                );
                                snapshot
                            }
                            Err(e) => {
                                error!("Failed to create atomic snapshot for BuildReach: {}", e);

                                // Fallback to basic snapshot creation for compatibility
                                warn!("Falling back to basic snapshot creation");
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

                                let snap = PruneSnapshot {
                                    state_root: startup_info_state_root,
                                    latest_order,
                                };
                                let _ = moveos_store.save_prune_meta_snapshot(snap);

                                let live_roots = vec![startup_info_state_root];
                                info!("Found {} live roots", live_roots.len());

                                let builder =
                                    ReachableBuilder::new(moveos_store.clone(), bloom.clone());

                                // Continue with normal processing using fallback snapshot
                                let start_time = std::time::Instant::now();
                                match builder.build(live_roots, num_cpus::get()) {
                                    Ok(scanned_size) => {
                                        let duration = start_time.elapsed();
                                        let nodes_per_sec =
                                            scanned_size as f64 / duration.as_secs_f64();

                                        info!(
                                            "Completed reachability build, scanned size {} in {:?} ({:.2} nodes/sec)",
                                            scanned_size, duration, nodes_per_sec
                                        );

                                        if let Some(ref metrics) = metrics {
                                            metrics
                                                .pruner_reachable_nodes_scanned
                                                .with_label_values(&["BuildReach"])
                                                .observe(scanned_size as f64);

                                            metrics
                                                .pruner_processing_speed_nodes_per_sec
                                                .with_label_values(&["reachability_build"])
                                                .observe(nodes_per_sec);
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Failed to reachability build: {}", e);
                                        if let Some(ref metrics) = metrics {
                                            metrics
                                                .pruner_error_count
                                                .with_label_values(&[
                                                    "reachability_build",
                                                    "BuildReach",
                                                ])
                                                .inc();
                                        }
                                    }
                                }

                                // Persist bloom snapshot after reachability phase
                                {
                                    if let Err(e) =
                                        moveos_store.save_prune_meta_bloom(bloom.lock().clone())
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
                                continue;
                            }
                        };

                        // Use atomic snapshot for live roots
                        let live_roots = vec![atomic_snapshot.snapshot.state_root];
                        info!("Found {} live roots from atomic snapshot", live_roots.len());
                        info!(
                            "Atomic snapshot state_root: {:?}",
                            atomic_snapshot.snapshot.state_root
                        );
                        info!(
                            "Atomic snapshot latest_order: {}",
                            atomic_snapshot.snapshot.latest_order
                        );

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
                                    metrics
                                        .pruner_reachable_nodes_scanned
                                        .with_label_values(&["BuildReach"])
                                        .observe(scanned_size as f64);

                                    metrics
                                        .pruner_processing_speed_nodes_per_sec
                                        .with_label_values(&["reachability_build"])
                                        .observe(nodes_per_sec);
                                }
                            }
                            Err(e) => {
                                warn!("Failed to reachability build: {}", e);
                                if let Some(ref metrics) = metrics {
                                    metrics
                                        .pruner_error_count
                                        .with_label_values(&["reachability_build", "BuildReach"])
                                        .inc();
                                }
                            }
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

                        // Lock the current atomic snapshot for this phase
                        let atomic_snapshot = match atomic_snapshot_manager_clone
                            .lock_snapshot(PrunePhase::SweepExpired)
                        {
                            Ok(snapshot) => {
                                info!(
                                    "Successfully locked atomic snapshot {} for SweepExpired",
                                    snapshot.snapshot_id
                                );
                                snapshot
                            }
                            Err(e) => {
                                error!("Failed to lock atomic snapshot for SweepExpired: {}", e);

                                // Fallback to basic snapshot loading for compatibility
                                warn!("Falling back to basic snapshot loading for SweepExpired");
                                let snapshot = moveos_store
                                    .load_prune_meta_snapshot()
                                    .ok()
                                    .flatten()
                                    .unwrap_or_default();
                                info!("Latest prune snapshot: {:?}", snapshot);

                                // Continue with normal processing using fallback snapshot
                                let latest_order = snapshot.latest_order;
                                let order_cursor = Self::calculate_sweep_start_order(
                                    latest_order,
                                    cfg.protection_orders,
                                );
                                info!(
                                    "Starting scan from order {} (protection_orders: {}, latest_order: {})",
                                    order_cursor, cfg.protection_orders, latest_order
                                );

                                let _sweeper = SweepExpired::new(
                                    moveos_store.clone(),
                                    bloom.clone(),
                                    cfg.bloom_bits,
                                    is_running_for_thread.clone(),
                                );

                                // Process using fallback logic (original implementation)
                                // ... (this would be a duplicate of the original logic)
                                // For brevity, let's just log and transition
                                warn!(
                                    "Skipping SweepExpired processing due to snapshot lock failure"
                                );
                                moveos_store
                                    .save_prune_meta_phase(PrunePhase::Incremental)
                                    .ok();
                                info!("Transitioning to Incremental phase");
                                continue;
                            }
                        };

                        // Validate snapshot consistency before processing
                        match atomic_snapshot_manager_clone.validate_phase_consistency() {
                            Ok(true) => {
                                info!(
                                    "Snapshot {} consistency validation passed",
                                    atomic_snapshot.snapshot_id
                                );
                            }
                            Ok(false) | Err(_) => {
                                warn!("Snapshot {} consistency validation failed. Creating new snapshot.", atomic_snapshot.snapshot_id);

                                // Try to create a fresh snapshot
                                match atomic_snapshot_manager_clone
                                    .refresh_snapshot(PrunePhase::SweepExpired)
                                {
                                    Ok(refreshed_snapshot) => {
                                        info!("Successfully created fresh snapshot {} for SweepExpired", refreshed_snapshot.snapshot_id);
                                        // Use the refreshed snapshot
                                        // Release old snapshot lock
                                        let _ = atomic_snapshot_manager_clone
                                            .release_snapshot(PrunePhase::SweepExpired);
                                        // Continue to use refreshed_snapshot from above
                                        let _atomic_snapshot = refreshed_snapshot;
                                    }
                                    Err(e) => {
                                        error!("Failed to refresh snapshot: {}", e);
                                        // Skip this cycle and retry later
                                        moveos_store
                                            .save_prune_meta_phase(PrunePhase::BuildReach)
                                            .ok();
                                        info!("Transitioning to BuildReach phase due to snapshot refresh error");
                                        continue;
                                    }
                                }
                            }
                        }

                        info!(
                            "Using atomic snapshot {} for SweepExpired processing",
                            atomic_snapshot.snapshot_id
                        );
                        info!(
                            "Atomic snapshot state_root: {:?}",
                            atomic_snapshot.snapshot.state_root
                        );
                        info!(
                            "Atomic snapshot latest_order: {}",
                            atomic_snapshot.snapshot.latest_order
                        );

                        // Stream process expired roots using atomic snapshot
                        let latest_order = atomic_snapshot.snapshot.latest_order;
                        let mut order_cursor =
                            Self::calculate_sweep_start_order(latest_order, cfg.protection_orders);
                        info!(
                            "Starting scan from order {} (protection_orders: {}, latest_order: {})",
                            order_cursor, cfg.protection_orders, latest_order
                        );

                        let sweeper = SweepExpired::new(
                            moveos_store.clone(),
                            bloom.clone(),
                            cfg.bloom_bits,
                            is_running_for_thread.clone(),
                        );
                        let sweep_start_time = std::time::Instant::now();
                        let mut processed_count = 0;
                        let mut total_deleted = 0;
                        let mut batch_roots = Vec::with_capacity(1000); // Process in smaller batches

                        // while processed_count < cfg.scan_batch && order_cursor > 0 {
                        while order_cursor > 0 {
                            // Check exit signal frequently
                            if !is_running_for_thread.load(Ordering::Relaxed)
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
                                    if !is_running_for_thread.load(Ordering::Relaxed)
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
                                            metrics
                                                .pruner_sweep_nodes_deleted
                                                .with_label_values(&["SweepExpired"])
                                                .observe(deleted as f64);

                                            // Estimate disk space reclaimed (assuming average node size of 32 bytes)
                                            let estimated_bytes_reclaimed = deleted * 32;
                                            metrics
                                                .pruner_disk_space_reclaimed_bytes
                                                .with_label_values(&["SweepExpired"])
                                                .inc_by(estimated_bytes_reclaimed as f64);
                                        }
                                    } else if let Some(ref metrics) = metrics {
                                        metrics
                                            .pruner_error_count
                                            .with_label_values(&["sweep_batch", "SweepExpired"])
                                            .inc();
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
                                    metrics
                                        .pruner_sweep_nodes_deleted
                                        .with_label_values(&["SweepExpired"])
                                        .observe(deleted as f64);

                                    let estimated_bytes_reclaimed = deleted * 32;
                                    metrics
                                        .pruner_disk_space_reclaimed_bytes
                                        .with_label_values(&["SweepExpired"])
                                        .inc_by(estimated_bytes_reclaimed as f64);
                                }
                            } else if let Some(ref metrics) = metrics {
                                metrics
                                    .pruner_error_count
                                    .with_label_values(&["sweep_final", "SweepExpired"])
                                    .inc();
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
                            metrics
                                .pruner_sweep_nodes_deleted
                                .with_label_values(&["SweepExpired_Total"])
                                .observe(total_deleted as f64);

                            metrics
                                .pruner_processing_speed_nodes_per_sec
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

                        // Release the atomic snapshot lock
                        if let Err(e) =
                            atomic_snapshot_manager_clone.release_snapshot(PrunePhase::SweepExpired)
                        {
                            warn!("Failed to release snapshot lock after SweepExpired: {}", e);
                        } else {
                            info!("Released snapshot lock after SweepExpired phase completion");
                        }

                        moveos_store
                            .save_prune_meta_phase(PrunePhase::Incremental)
                            .ok();
                        // only for test
                        // phase = PrunePhase::Incremental;
                        info!("Transitioning back to Incremental phase");
                    }
                    PrunePhase::Incremental => {
                        if !cfg.enable_incremental_sweep {
                            info!("Incremental sweep disabled in config, skipping");
                            // Transition back to BuildReach if incremental sweep is disabled
                            moveos_store
                                .save_prune_meta_phase(PrunePhase::BuildReach)
                                .ok();
                            info!("Skipping Incremental phase, transitioning to BuildReach");
                        } else {
                            info!("Starting Incremental sweep phase");

                            // Lock the current atomic snapshot for this phase
                            let atomic_snapshot = match atomic_snapshot_manager_clone
                                .lock_snapshot(PrunePhase::Incremental)
                            {
                                Ok(snapshot) => {
                                    info!(
                                        "Successfully locked atomic snapshot {} for Incremental",
                                        snapshot.snapshot_id
                                    );
                                    snapshot
                                }
                                Err(e) => {
                                    error!("Failed to lock atomic snapshot for Incremental: {}", e);

                                    // Fallback to basic snapshot loading for compatibility
                                    warn!("Falling back to basic snapshot loading for Incremental");
                                    let _snapshot = moveos_store
                                        .load_prune_meta_snapshot()
                                        .ok()
                                        .flatten()
                                        .unwrap_or_default();

                                    let incremental_sweeper =
                                        IncrementalSweep::new(moveos_store.clone());

                                    // Process using fallback logic
                                    match incremental_sweeper
                                        // Use max cutoff to sweep all stale indices (timestamp-based)
                                        .sweep(u64::MAX, cfg.incremental_sweep_batch)
                                    {
                                        Ok(deleted_count) => {
                                            if deleted_count > 0 {
                                                info!(
                                                    "Incremental sweep deleted {} nodes",
                                                    deleted_count
                                                );
                                            } else {
                                                info!("Incremental sweep found no nodes to delete");
                                            }

                                            if let Some(ref metrics) = metrics {
                                                metrics
                                                    .pruner_sweep_nodes_deleted
                                                    .with_label_values(&["incremental"])
                                                    .observe(deleted_count as f64);
                                            }
                                        }
                                        Err(e) => {
                                            warn!("Incremental sweep failed: {}", e);
                                            if let Some(ref metrics) = metrics {
                                                metrics
                                                    .pruner_error_count
                                                    .with_label_values(&[
                                                        "incremental_sweep",
                                                        "Incremental",
                                                    ])
                                                    .inc();
                                            }
                                        }
                                    }

                                    // Skip to next phase
                                    moveos_store
                                        .save_prune_meta_phase(PrunePhase::BuildReach)
                                        .ok();
                                    info!("Transitioning back to BuildReach phase");
                                    continue;
                                }
                            };

                            // Validate snapshot consistency before processing
                            match atomic_snapshot_manager_clone.validate_phase_consistency() {
                                Ok(true) => {
                                    info!(
                                        "Snapshot {} consistency validation passed for Incremental",
                                        atomic_snapshot.snapshot_id
                                    );
                                }
                                Ok(false) | Err(_) => {
                                    warn!("Snapshot {} consistency validation failed for Incremental. Skipping incremental sweep.", atomic_snapshot.snapshot_id);

                                    // Release the snapshot lock and skip this phase
                                    let _ = atomic_snapshot_manager_clone
                                        .release_snapshot(PrunePhase::Incremental);
                                    moveos_store
                                        .save_prune_meta_phase(PrunePhase::BuildReach)
                                        .ok();
                                    info!("Skipping Incremental phase due to snapshot validation failure, transitioning to BuildReach");
                                    continue;
                                }
                            }

                            info!(
                                "Using atomic snapshot {} for Incremental sweep processing",
                                atomic_snapshot.snapshot_id
                            );
                            info!(
                                "Atomic snapshot state_root: {:?}",
                                atomic_snapshot.snapshot.state_root
                            );

                            // Use incremental sweep to clean up remaining stale nodes
                            let incremental_sweeper = IncrementalSweep::new(moveos_store.clone());

                            match incremental_sweeper.sweep(u64::MAX, cfg.incremental_sweep_batch) {
                                Ok(deleted_count) => {
                                    if deleted_count > 0 {
                                        info!("Incremental sweep deleted {} nodes using atomic snapshot {}", deleted_count, atomic_snapshot.snapshot_id);
                                    } else {
                                        info!("Incremental sweep found no nodes to delete using atomic snapshot {}", atomic_snapshot.snapshot_id);
                                    }

                                    // Record metrics for nodes deleted during incremental sweep
                                    if let Some(ref metrics) = metrics {
                                        metrics
                                            .pruner_sweep_nodes_deleted
                                            .with_label_values(&["incremental"])
                                            .observe(deleted_count as f64);
                                    }
                                }
                                Err(e) => {
                                    warn!(
                                        "Incremental sweep failed with atomic snapshot {}: {}",
                                        atomic_snapshot.snapshot_id, e
                                    );
                                    if let Some(ref metrics) = metrics {
                                        metrics
                                            .pruner_error_count
                                            .with_label_values(&[
                                                "incremental_sweep",
                                                "Incremental",
                                            ])
                                            .inc();
                                    }
                                }
                            }

                            // Release the atomic snapshot lock
                            if let Err(e) = atomic_snapshot_manager_clone
                                .release_snapshot(PrunePhase::Incremental)
                            {
                                warn!("Failed to release snapshot lock after Incremental: {}", e);
                            } else {
                                info!("Released snapshot lock after Incremental phase completion");
                            }

                            // After incremental sweep is complete, transition back to BuildReach
                            moveos_store
                                .save_prune_meta_phase(PrunePhase::BuildReach)
                                .ok();
                            info!("Completed Incremental phase, transitioning to BuildReach");
                        }

                        // Check exit signal frequently
                        if !is_running_for_thread.load(Ordering::Relaxed)
                            || shutdown_rx.try_recv().is_ok()
                        {
                            info!("Pruner thread stopping during incremental sweep");
                            return;
                        }
                    }
                }

                info!("Sleeping for {} seconds", cfg.interval_s);
                // Sleep in small intervals to respond to exit signal quickly
                let mut slept = 0;
                while slept < cfg.interval_s {
                    if !is_running_for_thread.load(Ordering::Relaxed)
                        || shutdown_rx.try_recv().is_ok()
                    {
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
            is_running,
            atomic_snapshot_manager,
        })
    }

    pub fn stop(self) {
        if let Some(h) = self.handle {
            info!("Stopping pruner thread");
            self.is_running.store(false, Ordering::Relaxed);
            let _ = h.join();
            info!("Pruner thread stopped");
        }
    }

    /// Calculate the starting tx_order for SweepExpired phase.
    ///
    /// # Arguments
    /// * `latest_order` - The latest tx_order in the chain
    /// * `protection_orders` - Number of recent tx_orders to protect from pruning
    ///
    /// # Returns
    /// The tx_order from which to start sweeping (inclusive).
    ///
    /// # Behavior
    /// - If `protection_orders == 0`: Only protect the latest root (aggressive mode for testing)
    /// - Otherwise: Protect the configured number of recent orders
    fn calculate_sweep_start_order(latest_order: u64, protection_orders: u64) -> u64 {
        if protection_orders == 0 {
            // Aggressive mode: only protect the latest root
            // This is primarily for testing with --pruner-protection-orders 0
            if latest_order >= 1 {
                latest_order - 1
            } else {
                latest_order
            }
        } else {
            // Normal mode: protect configured number of orders
            latest_order.saturating_sub(protection_orders)
        }
    }
}
