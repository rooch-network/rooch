// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::atomic_snapshot::AtomicSnapshotManager;
use crate::metrics::PrunerMetrics;
use anyhow::{anyhow, Result};
use moveos_store::prune::PruneStore;
use moveos_store::MoveOSStore;
use moveos_types::prune::PrunePhase;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

/// Error recovery strategy configuration
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// Maximum retry attempts for phase operations
    pub max_phase_retries: u32,

    /// Base delay between retries in milliseconds
    pub base_retry_delay_ms: u64,

    /// Maximum delay between retries in milliseconds
    pub max_retry_delay_ms: u64,

    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,

    /// Enable automatic snapshot recovery
    pub enable_snapshot_recovery: bool,

    /// Maximum time to wait for snapshot recovery in seconds
    pub snapshot_recovery_timeout_secs: u64,

    /// Enable phase rollback on critical errors
    pub enable_phase_rollback: bool,

    /// Minimum healthy phase cycles before considering system stable
    pub min_healthy_cycles: u32,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_phase_retries: 3,
            base_retry_delay_ms: 1000,
            max_retry_delay_ms: 30000,
            backoff_multiplier: 2.0,
            enable_snapshot_recovery: true,
            snapshot_recovery_timeout_secs: 300, // 5 minutes
            enable_phase_rollback: true,
            min_healthy_cycles: 3,
        }
    }
}

/// Recovery statistics for monitoring
#[derive(Debug, Default)]
pub struct RecoveryStats {
    pub total_recoveries: AtomicU64,
    pub snapshot_recoveries: AtomicU64,
    pub phase_rollbacks: AtomicU64,
    pub retry_attempts: AtomicU64,
    pub successful_recoveries: AtomicU64,
    pub failed_recoveries: AtomicU64,
}

impl Clone for RecoveryStats {
    fn clone(&self) -> Self {
        Self {
            total_recoveries: AtomicU64::new(self.total_recoveries.load(Ordering::Relaxed)),
            snapshot_recoveries: AtomicU64::new(self.snapshot_recoveries.load(Ordering::Relaxed)),
            phase_rollbacks: AtomicU64::new(self.phase_rollbacks.load(Ordering::Relaxed)),
            retry_attempts: AtomicU64::new(self.retry_attempts.load(Ordering::Relaxed)),
            successful_recoveries: AtomicU64::new(
                self.successful_recoveries.load(Ordering::Relaxed),
            ),
            failed_recoveries: AtomicU64::new(self.failed_recoveries.load(Ordering::Relaxed)),
        }
    }
}

/// Error recovery manager for pruner
pub struct ErrorRecoveryManager {
    atomic_snapshot_manager: Arc<AtomicSnapshotManager>,
    moveos_store: Arc<MoveOSStore>,
    #[allow(dead_code)]
    metrics: Option<Arc<PrunerMetrics>>,
    config: RecoveryConfig,
    stats: Arc<RecoveryStats>,
}

impl ErrorRecoveryManager {
    pub fn new(
        atomic_snapshot_manager: Arc<AtomicSnapshotManager>,
        moveos_store: Arc<MoveOSStore>,
        metrics: Option<Arc<PrunerMetrics>>,
        config: Option<RecoveryConfig>,
    ) -> Self {
        Self {
            atomic_snapshot_manager,
            moveos_store,
            metrics,
            config: config.unwrap_or_default(),
            stats: Arc::new(RecoveryStats::default()),
        }
    }

    /// Execute a phase operation with automatic retry and error recovery
    pub async fn execute_phase_with_recovery<F, Fut, T>(
        &self,
        phase: PrunePhase,
        operation: F,
    ) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut attempt = 0;
        let mut last_error = None;

        while attempt < self.config.max_phase_retries {
            attempt += 1;
            self.stats.retry_attempts.fetch_add(1, Ordering::Relaxed);

            info!(
                "Executing phase {:?} (attempt {}/{})",
                phase, attempt, self.config.max_phase_retries
            );

            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        info!("Phase {:?} succeeded on attempt {}", phase, attempt);
                        self.stats
                            .successful_recoveries
                            .fetch_add(1, Ordering::Relaxed);

                        // Record recovery metrics
                        // TODO: Add pruner_recovery_attempts to metrics
                        // if let Some(ref metrics) = self.metrics {
                        //     metrics
                        //         .pruner_recovery_attempts
                        //         .with_label_values(&[&format!("{:?}", phase), "success"])
                        //         .inc();
                        // }
                    }
                    return Ok(result);
                }
                Err(e) => {
                    warn!("Phase {:?} failed on attempt {}: {}", phase, attempt, e);
                    last_error = Some(format!("{}", e));

                    // Attempt recovery based on error type
                    if let Err(recovery_error) = self.attempt_recovery(&phase, &e).await {
                        error!("Recovery failed for phase {:?}: {}", phase, recovery_error);
                        // Continue with retry even if recovery fails
                    }

                    // Record failure metrics
                    // TODO: Add pruner_recovery_attempts to metrics
                    // if let Some(ref metrics) = self.metrics {
                    //     metrics
                    //         .pruner_recovery_attempts
                    //         .with_label_values(&[&format!("{:?}", phase), "failure"])
                    //         .inc();
                    // }

                    // Add delay between retries with exponential backoff
                    if attempt < self.config.max_phase_retries {
                        let delay = self.calculate_retry_delay(attempt);
                        info!("Waiting {}ms before retry attempt {}", delay, attempt + 1);
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                    }
                }
            }
        }

        // All retries failed
        let error = last_error
            .unwrap_or_else(|| anyhow!("All retries failed for phase {:?}", phase).to_string());
        self.stats.failed_recoveries.fetch_add(1, Ordering::Relaxed);

        // Record final failure metrics
        // TODO: Add pruner_recovery_attempts to metrics
        // if let Some(ref metrics) = self.metrics {
        //     metrics
        //         .pruner_recovery_attempts
        //         .with_label_values(&[&format!("{:?}", phase), "exhausted"])
        //         .inc();
        // }

        Err(anyhow!(
            "Phase {:?} failed after {} attempts: {}",
            phase,
            attempt,
            error
        ))
    }

    /// Attempt recovery based on the error type and phase
    async fn attempt_recovery(&self, phase: &PrunePhase, error: &anyhow::Error) -> Result<()> {
        self.stats.total_recoveries.fetch_add(1, Ordering::Relaxed);
        info!("Attempting recovery for phase {:?}: {}", phase, error);

        let error_str = error.to_string().to_lowercase();

        // Check for snapshot-related errors
        if error_str.contains("snapshot") || error_str.contains("lock") {
            return self.recover_from_snapshot_error(phase).await;
        }

        // Check for database/storage errors
        if error_str.contains("database")
            || error_str.contains("storage")
            || error_str.contains("rocksdb")
        {
            return self.recover_from_storage_error(phase).await;
        }

        // Check for consistency/validation errors
        if error_str.contains("consistency") || error_str.contains("validation") {
            return self.recover_from_consistency_error(phase).await;
        }

        // Check for timeout errors
        if error_str.contains("timeout") || error_str.contains("deadline") {
            return self.recover_from_timeout_error(phase).await;
        }

        // Default recovery strategy
        self.default_recovery(phase).await
    }

    /// Recover from snapshot-related errors
    async fn recover_from_snapshot_error(&self, phase: &PrunePhase) -> Result<()> {
        if !self.config.enable_snapshot_recovery {
            warn!("Snapshot recovery is disabled");
            return self.default_recovery(phase).await;
        }

        self.stats
            .snapshot_recoveries
            .fetch_add(1, Ordering::Relaxed);
        info!("Attempting snapshot recovery for phase {:?}", phase);

        // Release any existing locks
        let _ = self.atomic_snapshot_manager.release_snapshot(*phase);

        // Clear the current snapshot
        if let Err(e) = self.atomic_snapshot_manager.clear_snapshot() {
            warn!("Failed to clear snapshot during recovery: {}", e);
        }

        // Wait for the system to stabilize
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Try to create a fresh snapshot
        let timeout = Duration::from_secs(self.config.snapshot_recovery_timeout_secs);
        let snapshot_creation = tokio::time::timeout(timeout, async {
            self.atomic_snapshot_manager.create_snapshot(*phase)
        });

        match snapshot_creation.await {
            Ok(Ok(snapshot)) => {
                info!(
                    "Successfully created new snapshot {} during recovery",
                    snapshot.snapshot_id
                );
                Ok(())
            }
            Ok(Err(e)) => {
                error!("Failed to create snapshot during recovery: {}", e);
                Err(anyhow!("Snapshot recovery failed: {}", e))
            }
            Err(_) => {
                error!(
                    "Snapshot recovery timed out after {} seconds",
                    self.config.snapshot_recovery_timeout_secs
                );
                Err(anyhow!("Snapshot recovery timed out"))
            }
        }
    }

    /// Recover from storage/database errors
    async fn recover_from_storage_error(&self, phase: &PrunePhase) -> Result<()> {
        info!("Attempting storage recovery for phase {:?}", phase);

        // Pause and retry after storage stabilizes
        tokio::time::sleep(Duration::from_secs(10)).await;

        // Try to re-establish database connections or flush caches
        // This would be implementation-specific based on the storage layer

        info!("Storage recovery completed for phase {:?}", phase);
        Ok(())
    }

    /// Recover from consistency/validation errors
    async fn recover_from_consistency_error(&self, phase: &PrunePhase) -> Result<()> {
        info!("Attempting consistency recovery for phase {:?}", phase);

        // Force rollback to previous consistent state
        if self.config.enable_phase_rollback {
            if let Err(e) = self.rollback_phase(phase).await {
                warn!("Phase rollback failed: {}", e);
            }
        }

        // Wait for system to reach consistent state
        tokio::time::sleep(Duration::from_secs(15)).await;

        // Try to validate system consistency again
        if let Ok(false) = self.atomic_snapshot_manager.validate_phase_consistency() {
            warn!("System consistency still invalid after recovery attempt");
            return Err(anyhow!("System consistency could not be restored"));
        }

        info!("Consistency recovery completed for phase {:?}", phase);
        Ok(())
    }

    /// Recover from timeout errors
    async fn recover_from_timeout_error(&self, phase: &PrunePhase) -> Result<()> {
        info!("Attempting timeout recovery for phase {:?}", phase);

        // Give the system more time to complete operations
        tokio::time::sleep(Duration::from_secs(30)).await;

        // Check if the operation actually completed despite the timeout
        // This might involve checking phase state or intermediate results

        info!("Timeout recovery completed for phase {:?}", phase);
        Ok(())
    }

    /// Default recovery strategy
    async fn default_recovery(&self, phase: &PrunePhase) -> Result<()> {
        info!("Applying default recovery for phase {:?}", phase);

        // Wait for system to stabilize
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Try to reset to a safe state
        if self.config.enable_phase_rollback {
            let _ = self.rollback_phase(phase).await;
        }

        info!("Default recovery completed for phase {:?}", phase);
        Ok(())
    }

    /// Rollback a phase to previous safe state
    async fn rollback_phase(&self, phase: &PrunePhase) -> Result<()> {
        self.stats.phase_rollbacks.fetch_add(1, Ordering::Relaxed);
        info!("Rolling back phase {:?}", phase);

        // Release any snapshot locks
        let _ = self.atomic_snapshot_manager.release_snapshot(*phase);

        // Reset phase to BuildReach (safe starting point)
        let rollback_phase = match phase {
            PrunePhase::BuildReach => PrunePhase::BuildReach, // Already at safe state
            PrunePhase::SweepExpired => PrunePhase::BuildReach,
            PrunePhase::Incremental => PrunePhase::BuildReach,
        };

        if rollback_phase != *phase {
            info!("Rolling back phase {:?} to {:?}", phase, rollback_phase);
            self.moveos_store.save_prune_meta_phase(rollback_phase)?;
        }

        info!("Phase rollback completed");
        Ok(())
    }

    /// Calculate retry delay with exponential backoff
    fn calculate_retry_delay(&self, attempt: u32) -> u64 {
        let base_delay = self.config.base_retry_delay_ms as f64;
        let exponential_delay =
            base_delay * self.config.backoff_multiplier.powi(attempt as i32 - 1);
        let jittered_delay = exponential_delay * (0.8 + (rand::random::<f64>() * 0.4)); // ±20% jitter

        (jittered_delay as u64)
            .min(self.config.max_retry_delay_ms)
            .max(self.config.base_retry_delay_ms)
    }

    /// Get recovery statistics
    pub fn get_stats(&self) -> &RecoveryStats {
        &self.stats
    }

    /// Check if system is healthy and stable
    pub async fn check_system_health(&self) -> Result<bool> {
        // Check snapshot manager health
        let snapshot_status = self.atomic_snapshot_manager.get_snapshot_status()?;

        // Validate snapshot consistency
        let consistency_ok = self
            .atomic_snapshot_manager
            .validate_phase_consistency()
            .unwrap_or(false);

        // Check for recent errors or recoveries
        let recent_recoveries = self.stats.total_recoveries.load(Ordering::Relaxed);

        info!(
            "System health check: snapshot_ok={}, consistency_ok={}, recent_recoveries={}",
            snapshot_status.current_snapshot.is_some(),
            consistency_ok,
            recent_recoveries
        );

        let healthy =
            snapshot_status.current_snapshot.is_some() && consistency_ok && recent_recoveries < 10;

        if !healthy {
            warn!("System health check failed: snapshot_ok={}, consistency_ok={}, recent_recoveries={}",
                  snapshot_status.current_snapshot.is_some(), consistency_ok, recent_recoveries);
        }

        Ok(healthy)
    }

    /// Perform system health check and attempt corrective actions
    pub async fn ensure_system_health(&self) -> Result<()> {
        if !self.check_system_health().await? {
            info!("System health check failed, attempting corrective actions");

            // Try to fix common issues
            if let Err(e) = self.atomic_snapshot_manager.clear_snapshot() {
                warn!("Failed to clear snapshot during health check: {}", e);
            }

            // Wait for system to stabilize
            tokio::time::sleep(Duration::from_secs(10)).await;

            // Try to create a new snapshot
            if let Err(e) = self
                .atomic_snapshot_manager
                .create_snapshot(PrunePhase::BuildReach)
            {
                error!("Failed to create new snapshot during health check: {}", e);
                return Err(anyhow!("System health recovery failed: {}", e));
            }

            info!("System health recovery completed");
        } else {
            info!("System health check passed");
        }

        Ok(())
    }

    /// Graceful shutdown with cleanup
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down error recovery manager");

        // Release any held snapshot locks
        for phase in [
            PrunePhase::BuildReach,
            PrunePhase::SweepExpired,
            PrunePhase::Incremental,
        ] {
            if let Err(e) = self.atomic_snapshot_manager.release_snapshot(phase) {
                warn!(
                    "Failed to release snapshot lock for phase {:?} during shutdown: {}",
                    phase, e
                );
            }
        }

        // Clear current snapshot
        if let Err(e) = self.atomic_snapshot_manager.clear_snapshot() {
            warn!("Failed to clear snapshot during shutdown: {}", e);
        }

        info!("Error recovery manager shutdown completed");
        Ok(())
    }
}

/// Utility function to create and start an error recovery manager
pub fn create_error_recovery_manager(
    atomic_snapshot_manager: Arc<AtomicSnapshotManager>,
    moveos_store: Arc<MoveOSStore>,
    metrics: Option<Arc<PrunerMetrics>>,
) -> Arc<ErrorRecoveryManager> {
    let config = RecoveryConfig::default();
    Arc::new(ErrorRecoveryManager::new(
        atomic_snapshot_manager,
        moveos_store,
        metrics,
        Some(config),
    ))
}
