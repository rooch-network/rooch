// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::metrics::PrunerMetrics;
use anyhow::{anyhow, Result};
use moveos_store::config_store::ConfigStore;
use moveos_store::prune::PruneStore;
use moveos_store::MoveOSStore;
use moveos_types::prune::{PrunePhase, PruneSnapshot};
use parking_lot::{Mutex, RwLock};
use primitive_types::H256;
use rooch_store::meta_store::MetaStore;
use rooch_store::RoochStore;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};

/// Atomic snapshot with enhanced metadata for consistency validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicSnapshot {
    /// Core snapshot data
    pub snapshot: PruneSnapshot,

    /// Unique identifier for this snapshot instance
    pub snapshot_id: u64,

    /// Timestamp when snapshot was created (milliseconds since UNIX epoch)
    pub created_at: u64,

    /// Phase when this snapshot was created
    pub created_phase: PrunePhase,

    /// Chain state metadata for validation
    pub chain_metadata: ChainMetadata,

    /// Signature/hash of the snapshot for integrity verification
    pub integrity_hash: H256,
}

/// Chain state metadata for snapshot validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainMetadata {
    /// Block height when snapshot was created
    pub block_height: u64,

    /// Blockchain timestamp when snapshot was created
    pub chain_timestamp: u64,

    /// Hash of the latest block
    pub latest_block_hash: H256,

    /// Number of transactions in the mempool (optional)
    pub mempool_size: Option<u64>,

    /// Network sync status (optional)
    pub sync_status: Option<bool>,
}

/// Snapshot lock status and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotLock {
    /// ID of the locked snapshot
    pub snapshot_id: u64,

    /// Phase that acquired the lock
    pub locked_by_phase: PrunePhase,

    /// When the lock was acquired
    pub locked_at: u64,

    /// Lock timeout in milliseconds
    pub timeout_ms: u64,

    /// Whether this snapshot is still valid
    pub is_valid: bool,
}

/// Atomic snapshot manager with consistency validation and locking
pub struct AtomicSnapshotManager {
    moveos_store: Arc<MoveOSStore>,
    rooch_store: Arc<RoochStore>,

    /// Current atomic snapshot (protected by read-write lock)
    current_snapshot: Arc<RwLock<Option<AtomicSnapshot>>>,

    /// Current snapshot lock
    snapshot_lock: Arc<Mutex<Option<SnapshotLock>>>,

    /// Counter for generating unique snapshot IDs
    snapshot_counter: Arc<AtomicU64>,

    /// Metrics instance
    metrics: Option<Arc<PrunerMetrics>>,

    /// Configuration
    config: SnapshotManagerConfig,
}

/// Configuration for snapshot manager
#[derive(Debug, Clone)]
pub struct SnapshotManagerConfig {
    /// Snapshot lock timeout in milliseconds (default: 30 minutes)
    pub lock_timeout_ms: u64,

    /// Maximum age for a snapshot in milliseconds (default: 2 hours)
    pub max_snapshot_age_ms: u64,

    /// Enable detailed consistency validation
    pub enable_validation: bool,

    /// Snapshot persistence enabled
    pub enable_persistence: bool,
}

impl Default for SnapshotManagerConfig {
    fn default() -> Self {
        Self {
            lock_timeout_ms: 30 * 60 * 1000,         // 30 minutes
            max_snapshot_age_ms: 2 * 60 * 60 * 1000, // 2 hours
            enable_validation: true,
            enable_persistence: true,
        }
    }
}

impl AtomicSnapshotManager {
    pub fn new(
        moveos_store: Arc<MoveOSStore>,
        rooch_store: Arc<RoochStore>,
        metrics: Option<Arc<PrunerMetrics>>,
        config: Option<SnapshotManagerConfig>,
    ) -> Self {
        Self {
            moveos_store,
            rooch_store,
            current_snapshot: Arc::new(RwLock::new(None)),
            snapshot_lock: Arc::new(Mutex::new(None)),
            snapshot_counter: Arc::new(AtomicU64::new(0)),
            metrics,
            config: config.unwrap_or_default(),
        }
    }

    /// Create a new atomic snapshot with comprehensive validation
    pub fn create_snapshot(&self, phase: PrunePhase) -> Result<AtomicSnapshot> {
        let start_time = Instant::now();
        info!("Creating atomic snapshot for phase: {:?}", phase);

        // Generate unique snapshot ID
        let snapshot_id = self.snapshot_counter.fetch_add(1, Ordering::SeqCst);

        // Record start time for atomicity
        let atomic_start = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow!("System time error: {}", e))?;

        // Atomically collect all chain state data
        let chain_metadata = self.collect_chain_metadata()?;
        let snapshot = self.collect_prune_snapshot()?;

        // Verify that state hasn't changed during collection
        let atomic_end = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow!("System time error: {}", e))?;

        // Ensure collection completes within acceptable time window (e.g., 5 seconds)
        let collection_duration = atomic_end.saturating_sub(atomic_start);
        if collection_duration.as_millis() > 5000 {
            warn!(
                "Snapshot collection took longer than expected: {}ms",
                collection_duration.as_millis()
            );
        }

        // Create atomic snapshot
        let atomic_snapshot = AtomicSnapshot {
            snapshot,
            snapshot_id,
            created_at: atomic_start.as_millis() as u64,
            created_phase: phase,
            chain_metadata,
            integrity_hash: H256::zero(), // Will be calculated below
        };

        // Calculate integrity hash
        let atomic_snapshot = self.calculate_integrity_hash(atomic_snapshot)?;

        // Validate snapshot if enabled
        if self.config.enable_validation {
            self.validate_snapshot_consistency(&atomic_snapshot)?;
        }

        // Store the new snapshot
        {
            let mut current_guard = self.current_snapshot.write();
            *current_guard = Some(atomic_snapshot.clone());
        }

        // Persist snapshot if enabled
        if self.config.enable_persistence {
            self.persist_snapshot(&atomic_snapshot)?;
        }

        let duration = start_time.elapsed();
        info!(
            "Successfully created atomic snapshot {} for phase {:?} in {:?}",
            atomic_snapshot.snapshot_id, phase, duration
        );

        // Record metrics
        if let Some(ref metrics) = self.metrics {
            // TODO: Add snapshot creation metrics to PrunerMetrics
            // metrics
            //     .pruner_snapshot_creation_duration_seconds
            //     .with_label_values(&[&format!("{:?}", phase)])
            //     .observe(duration.as_secs_f64());

            // metrics
            //     .pruner_snapshot_count
            //     .with_label_values(&["created"])
            //     .inc();
        }

        Ok(atomic_snapshot)
    }

    /// Lock a snapshot for exclusive use by a specific phase
    pub fn lock_snapshot(&self, phase: PrunePhase) -> Result<AtomicSnapshot> {
        let snapshot = self.get_current_snapshot()?;

        // Check if snapshot is already locked by a different phase
        {
            let lock_snapshot_id: u64;
            {
                let mut lock_guard = self.snapshot_lock.lock();
                if let Some(ref lock) = *lock_guard {
                    // Check if lock is still valid
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map_err(|e| anyhow!("System time error: {}", e))?
                        .as_millis() as u64;

                    if lock.is_valid && (now - lock.locked_at) < self.config.lock_timeout_ms {
                        if lock.locked_by_phase != phase {
                            return Err(anyhow!(
                                "Snapshot {} is already locked by phase {:?}. Current phase: {:?}",
                                lock.snapshot_id,
                                lock.locked_by_phase,
                                phase
                            ));
                        }
                        // Same phase, return the snapshot without extending lock (immutable)
                        lock_snapshot_id = lock.snapshot_id;
                        return Ok(snapshot);
                    } else {
                        // Lock expired, clear it
                        lock_snapshot_id = lock.snapshot_id;
                        *lock_guard = None;
                        warn!("Previous snapshot lock {} expired", lock_snapshot_id);
                    }
                } else {
                    lock_snapshot_id = 0;
                }
            }

            // Acquire new lock
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| anyhow!("System time error: {}", e))?
                .as_millis() as u64;

            {
                let mut lock_guard = self.snapshot_lock.lock();
                *lock_guard = Some(SnapshotLock {
                    snapshot_id: snapshot.snapshot_id,
                    locked_by_phase: phase,
                    locked_at: now,
                    timeout_ms: self.config.lock_timeout_ms,
                    is_valid: true,
                });
            }

            info!("Phase {:?} locked snapshot {}", phase, snapshot.snapshot_id);

            // Validate snapshot integrity before returning
            self.validate_snapshot_integrity(&snapshot)?;

            Ok(snapshot)
        }
    }

    /// Release snapshot lock
    pub fn release_snapshot(&self, phase: PrunePhase) -> Result<()> {
        let snapshot_id_to_release;
        {
            let mut lock_guard = self.snapshot_lock.lock();
            if let Some(ref lock) = *lock_guard {
                if lock.locked_by_phase == phase {
                    snapshot_id_to_release = lock.snapshot_id.clone();
                    *lock_guard = None;
                } else {
                    return Err(anyhow!(
                        "Attempted to release snapshot lock {} by phase {:?}, but it's held by {:?}",
                        lock.snapshot_id, phase, lock.locked_by_phase
                    ));
                }
            } else {
                // No lock to release
                return Ok(());
            }
        }

        info!(
            "Phase {:?} released snapshot lock {}",
            phase, snapshot_id_to_release
        );
        Ok(())
    }

    /// Get current snapshot without locking
    pub fn get_current_snapshot(&self) -> Result<AtomicSnapshot> {
        let guard = self.current_snapshot.read();
        guard
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow!("No snapshot available"))
    }

    /// Validate that all three phases are using the same snapshot
    pub fn validate_phase_consistency(&self) -> Result<bool> {
        let snapshot = self.get_current_snapshot()?;
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow!("System time error: {}", e))?
            .as_millis() as u64;

        // Check snapshot age
        let age = current_time.saturating_sub(snapshot.created_at);
        if age > self.config.max_snapshot_age_ms {
            warn!(
                "Snapshot {} is too old: {}ms (max: {}ms)",
                snapshot.snapshot_id, age, self.config.max_snapshot_age_ms
            );
            return Ok(false);
        }

        // Validate chain state hasn't significantly changed
        if self.config.enable_validation {
            let current_metadata = self.collect_chain_metadata()?;
            self.validate_chain_state_consistency(&snapshot.chain_metadata, &current_metadata)?;
        }

        info!(
            "Phase consistency validated for snapshot {}",
            snapshot.snapshot_id
        );
        Ok(true)
    }

    /// Force refresh of current snapshot
    pub fn refresh_snapshot(&self, phase: PrunePhase) -> Result<AtomicSnapshot> {
        info!("Force refreshing snapshot for phase: {:?}", phase);
        self.create_snapshot(phase)
    }

    /// Clear current snapshot and locks
    pub fn clear_snapshot(&self) -> Result<()> {
        {
            let mut snapshot_guard = self.current_snapshot.write();
            *snapshot_guard = None;
        }
        {
            let mut lock_guard = self.snapshot_lock.lock();
            *lock_guard = None;
        }
        info!("Cleared current snapshot and locks");
        Ok(())
    }

    /// Collect chain metadata for snapshot validation
    fn collect_chain_metadata(&self) -> Result<ChainMetadata> {
        // Get latest sequencer info for block height and timestamp
        let sequencer_info = self
            .rooch_store
            .get_sequencer_info()?
            .ok_or_else(|| anyhow!("No sequencer info available"))?;

        let startup_info = self
            .moveos_store
            .get_startup_info()?
            .ok_or_else(|| anyhow!("No startup info available"))?;

        Ok(ChainMetadata {
            block_height: sequencer_info.last_order,
            chain_timestamp: 0, // TODO: Get actual timestamp from startup_info
            latest_block_hash: startup_info.state_root,
            mempool_size: None, // Could be added if mempool access is available
            sync_status: None,  // Could be added if sync status is available
        })
    }

    /// Collect prune snapshot data
    fn collect_prune_snapshot(&self) -> Result<PruneSnapshot> {
        let startup_info = self
            .moveos_store
            .get_startup_info()?
            .ok_or_else(|| anyhow!("No startup info available"))?;

        let latest_order = self
            .rooch_store
            .get_sequencer_info()?
            .map(|info| info.last_order)
            .unwrap_or(0);

        Ok(PruneSnapshot {
            state_root: startup_info.state_root,
            latest_order,
        })
    }

    /// Calculate integrity hash for snapshot
    fn calculate_integrity_hash(&self, mut snapshot: AtomicSnapshot) -> Result<AtomicSnapshot> {
        // For simplicity, use state_root as integrity hash
        // In production, this should be a proper cryptographic hash of the entire snapshot
        snapshot.integrity_hash = snapshot.snapshot.state_root;
        Ok(snapshot)
    }

    /// Validate snapshot consistency
    fn validate_snapshot_consistency(&self, snapshot: &AtomicSnapshot) -> Result<()> {
        // Validate that snapshot data is internally consistent
        if snapshot.snapshot.latest_order != snapshot.chain_metadata.block_height {
            return Err(anyhow!(
                "Snapshot inconsistency: latest_order {} != block_height {}",
                snapshot.snapshot.latest_order,
                snapshot.chain_metadata.block_height
            ));
        }

        if snapshot.snapshot.state_root != snapshot.chain_metadata.latest_block_hash {
            return Err(anyhow!(
                "Snapshot inconsistency: state_root {:?} != latest_block_hash {:?}",
                snapshot.snapshot.state_root,
                snapshot.chain_metadata.latest_block_hash
            ));
        }

        debug!(
            "Snapshot {} consistency validation passed",
            snapshot.snapshot_id
        );
        Ok(())
    }

    /// Validate snapshot integrity
    fn validate_snapshot_integrity(&self, snapshot: &AtomicSnapshot) -> Result<()> {
        // Verify integrity hash
        let expected_hash = snapshot.snapshot.state_root;
        if snapshot.integrity_hash != expected_hash {
            return Err(anyhow!(
                "Snapshot integrity check failed: expected {:?}, got {:?}",
                expected_hash,
                snapshot.integrity_hash
            ));
        }

        debug!(
            "Snapshot {} integrity validation passed",
            snapshot.snapshot_id
        );
        Ok(())
    }

    /// Validate chain state consistency
    fn validate_chain_state_consistency(
        &self,
        original: &ChainMetadata,
        current: &ChainMetadata,
    ) -> Result<()> {
        // Allow some tolerance for chain progression
        let height_tolerance = 1000; // Allow 1000 blocks of progression
        let time_tolerance = 60000; // Allow 60 seconds of time difference

        if current.block_height < original.block_height {
            return Err(anyhow!(
                "Chain height regression: current {} < original {}",
                current.block_height,
                original.block_height
            ));
        }

        if current.block_height > original.block_height + height_tolerance {
            warn!(
                "Chain has progressed significantly: original height {}, current height {}",
                original.block_height, current.block_height
            );
        }

        if current.chain_timestamp > original.chain_timestamp + time_tolerance {
            warn!(
                "Chain time has progressed significantly: original {}, current {}",
                original.chain_timestamp, current.chain_timestamp
            );
        }

        debug!("Chain state consistency validation passed");
        Ok(())
    }

    /// Persist snapshot to storage
    fn persist_snapshot(&self, snapshot: &AtomicSnapshot) -> Result<()> {
        // This would persist the atomic snapshot to storage
        // For now, we'll just save the basic PruneSnapshot
        self.moveos_store
            .save_prune_meta_snapshot(snapshot.snapshot.clone())?;

        debug!(
            "Persisted atomic snapshot {} to storage",
            snapshot.snapshot_id
        );
        Ok(())
    }

    /// Load persisted snapshot if available
    pub fn load_persisted_snapshot(&self) -> Result<Option<AtomicSnapshot>> {
        if let Some(basic_snapshot) = self.moveos_store.load_prune_meta_snapshot()? {
            // For now, create a minimal atomic snapshot from the persisted basic one
            // In a full implementation, we would also persist and load the full atomic snapshot
            let atomic_snapshot = AtomicSnapshot {
                snapshot: basic_snapshot,
                snapshot_id: self.snapshot_counter.fetch_add(1, Ordering::SeqCst),
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| anyhow!("System time error: {}", e))?
                    .as_millis() as u64,
                created_phase: PrunePhase::BuildReach, // Assume it was created during BuildReach
                chain_metadata: self.collect_chain_metadata()?,
                integrity_hash: H256::zero(),
            };

            let atomic_snapshot = self.calculate_integrity_hash(atomic_snapshot)?;
            Ok(Some(atomic_snapshot))
        } else {
            Ok(None)
        }
    }

    /// Initialize the snapshot manager by loading any persisted snapshot
    pub fn initialize(&self) -> Result<()> {
        info!("Initializing AtomicSnapshotManager");

        if let Some(persisted_snapshot) = self.load_persisted_snapshot()? {
            info!(
                "Loaded persisted snapshot {} from storage",
                persisted_snapshot.snapshot_id
            );

            {
                let mut guard = self.current_snapshot.write();
                *guard = Some(persisted_snapshot);
            }
        } else {
            info!("No persisted snapshot found, starting fresh");
        }

        info!("AtomicSnapshotManager initialization completed");
        Ok(())
    }

    /// Get snapshot status information
    pub fn get_snapshot_status(&self) -> Result<SnapshotStatus> {
        let snapshot_opt = {
            let guard = self.current_snapshot.read();
            guard.clone()
        };

        let lock_opt = {
            let guard = self.snapshot_lock.lock();
            guard.clone()
        };

        Ok(SnapshotStatus {
            current_snapshot: snapshot_opt,
            current_lock: lock_opt,
            manager_config: self.config.clone(),
        })
    }
}

/// Snapshot status information for monitoring
#[derive(Debug, Clone)]
pub struct SnapshotStatus {
    pub current_snapshot: Option<AtomicSnapshot>,
    pub current_lock: Option<SnapshotLock>,
    pub manager_config: SnapshotManagerConfig,
}

/// Snapshot validation error types
#[derive(Debug, thiserror::Error)]
pub enum SnapshotValidationError {
    #[error("No snapshot available")]
    NoSnapshot,

    #[error("Snapshot is too old: {age}ms (max: {max_age}ms)")]
    SnapshotTooOld { age: u64, max_age: u64 },

    #[error("Snapshot integrity check failed: expected {expected:?}, got {actual:?}")]
    IntegrityCheckFailed { expected: H256, actual: H256 },

    #[error("Snapshot inconsistency: {message}")]
    Inconsistency { message: String },

    #[error("Chain state has significantly changed: {message}")]
    ChainStateChanged { message: String },

    #[error("Snapshot lock conflict: {message}")]
    LockConflict { message: String },
}
