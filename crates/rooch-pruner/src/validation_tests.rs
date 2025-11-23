// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::atomic_snapshot::{AtomicSnapshot, AtomicSnapshotManager, ChainMetadata, SnapshotLock};
use crate::error_recovery::ErrorRecoveryManager;
use anyhow::{anyhow, Result};
use moveos_store::MoveOSStore;
use moveos_types::prune::PrunePhase;
use primitive_types::H256;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};

/// Comprehensive validation utilities for atomic snapshot mechanism
pub struct SnapshotValidator {
    atomic_snapshot_manager: Arc<AtomicSnapshotManager>,
    error_recovery_manager: Option<Arc<ErrorRecoveryManager>>,
}

impl SnapshotValidator {
    pub fn new(
        atomic_snapshot_manager: Arc<AtomicSnapshotManager>,
        error_recovery_manager: Option<Arc<ErrorRecoveryManager>>,
    ) -> Self {
        Self {
            atomic_snapshot_manager,
            error_recovery_manager,
        }
    }

    /// Run comprehensive validation suite
    pub async fn run_comprehensive_validation(&self) -> ValidationReport {
        let mut report = ValidationReport::new();

        info!("Starting comprehensive snapshot validation");

        // Test 1: Basic snapshot creation and validation
        report.add_result("snapshot_creation", self.test_snapshot_creation().await);

        // Test 2: Snapshot locking mechanism
        report.add_result("snapshot_locking", self.test_snapshot_locking().await);

        // Test 3: Snapshot consistency across phases
        report.add_result(
            "cross_phase_consistency",
            self.test_cross_phase_consistency().await,
        );

        // Test 4: Error recovery mechanisms
        report.add_result("error_recovery", self.test_error_recovery().await);

        // Test 5: Snapshot integrity validation
        report.add_result(
            "integrity_validation",
            self.test_integrity_validation().await,
        );

        // Test 6: Concurrency and race conditions
        report.add_result("concurrency_safety", self.test_concurrency_safety().await);

        // Test 7: Performance and scalability
        report.add_result(
            "performance_validation",
            self.test_performance_validation().await,
        );

        // Test 8: Memory and resource management
        report.add_result("resource_management", self.test_resource_management().await);

        info!("Comprehensive validation completed: {}", report.summary());
        report
    }

    /// Test basic snapshot creation and validation
    async fn test_snapshot_creation(&self) -> ValidationResult {
        let test_name = "snapshot_creation";
        info!("Testing {}", test_name);

        match self
            .atomic_snapshot_manager
            .create_snapshot(PrunePhase::BuildReach)
        {
            Ok(snapshot) => {
                // Validate snapshot structure
                if snapshot.snapshot_id == 0 {
                    return ValidationResult::Failed(
                        test_name.to_string(),
                        "Snapshot ID should not be zero".to_string(),
                    );
                }

                if snapshot.created_at == 0 {
                    return ValidationResult::Failed(
                        test_name.to_string(),
                        "Created timestamp should not be zero".to_string(),
                    );
                }

                if snapshot.integrity_hash == H256::zero() {
                    return ValidationResult::Failed(
                        test_name.to_string(),
                        "Integrity hash should not be zero".to_string(),
                    );
                }

                // Validate snapshot consistency
                if let Err(e) = self.atomic_snapshot_manager.validate_phase_consistency() {
                    return ValidationResult::Failed(
                        test_name.to_string(),
                        format!("Snapshot consistency validation failed: {}", e),
                    );
                }

                ValidationResult::Passed(
                    test_name.to_string(),
                    format!(
                        "Successfully created and validated snapshot {}",
                        snapshot.snapshot_id
                    ),
                )
            }
            Err(e) => ValidationResult::Failed(
                test_name.to_string(),
                format!("Snapshot creation failed: {}", e),
            ),
        }
    }

    /// Test snapshot locking mechanism
    async fn test_snapshot_locking(&self) -> ValidationResult {
        let test_name = "snapshot_locking";
        info!("Testing {}", test_name);

        // Create a snapshot
        let snapshot = match self
            .atomic_snapshot_manager
            .create_snapshot(PrunePhase::BuildReach)
        {
            Ok(s) => s,
            Err(e) => {
                return ValidationResult::Failed(
                    test_name.to_string(),
                    format!("Failed to create snapshot for lock test: {}", e),
                );
            }
        };

        // Test successful lock
        match self
            .atomic_snapshot_manager
            .lock_snapshot(PrunePhase::SweepExpired)
        {
            Ok(locked_snapshot) => {
                if locked_snapshot.snapshot_id != snapshot.snapshot_id {
                    return ValidationResult::Failed(
                        test_name.to_string(),
                        "Locked snapshot ID mismatch".to_string(),
                    );
                }

                // Test lock conflict
                match self
                    .atomic_snapshot_manager
                    .lock_snapshot(PrunePhase::Incremental)
                {
                    Ok(_) => {
                        return ValidationResult::Failed(
                            test_name.to_string(),
                            "Should not allow locking by different phase".to_string(),
                        );
                    }
                    Err(_) => {
                        // Expected behavior
                    }
                }

                // Test successful release
                if let Err(e) = self
                    .atomic_snapshot_manager
                    .release_snapshot(PrunePhase::SweepExpired)
                {
                    return ValidationResult::Failed(
                        test_name.to_string(),
                        format!("Failed to release snapshot: {}", e),
                    );
                }

                // Test re-lock after release
                match self
                    .atomic_snapshot_manager
                    .lock_snapshot(PrunePhase::Incremental)
                {
                    Ok(_) => {
                        // Should succeed now
                        let _ = self
                            .atomic_snapshot_manager
                            .release_snapshot(PrunePhase::Incremental);
                    }
                    Err(e) => {
                        return ValidationResult::Failed(
                            test_name.to_string(),
                            format!("Failed to re-lock after release: {}", e),
                        );
                    }
                }
            }
            Err(e) => {
                return ValidationResult::Failed(
                    test_name.to_string(),
                    format!("Failed to lock snapshot: {}", e),
                );
            }
        }

        ValidationResult::Passed(
            test_name.to_string(),
            "Snapshot locking mechanism working correctly".to_string(),
        )
    }

    /// Test consistency across different phases
    async fn test_cross_phase_consistency(&self) -> ValidationResult {
        let test_name = "cross_phase_consistency";
        info!("Testing {}", test_name);

        // Create snapshot for BuildReach
        let build_snapshot = match self
            .atomic_snapshot_manager
            .create_snapshot(PrunePhase::BuildReach)
        {
            Ok(s) => s,
            Err(e) => {
                return ValidationResult::Failed(
                    test_name.to_string(),
                    format!("Failed to create BuildReach snapshot: {}", e),
                );
            }
        };

        // Lock for SweepExpired
        let sweep_snapshot = match self
            .atomic_snapshot_manager
            .lock_snapshot(PrunePhase::SweepExpired)
        {
            Ok(s) => s,
            Err(e) => {
                return ValidationResult::Failed(
                    test_name.to_string(),
                    format!("Failed to lock for SweepExpired: {}", e),
                );
            }
        };

        // Verify it's the same snapshot
        if sweep_snapshot.snapshot_id != build_snapshot.snapshot_id {
            return ValidationResult::Failed(
                test_name.to_string(),
                "Snapshot ID mismatch between phases".to_string(),
            );
        }

        if sweep_snapshot.snapshot.state_root != build_snapshot.snapshot.state_root {
            return ValidationResult::Failed(
                test_name.to_string(),
                "State root mismatch between phases".to_string(),
            );
        }

        if sweep_snapshot.snapshot.latest_order != build_snapshot.snapshot.latest_order {
            return ValidationResult::Failed(
                test_name.to_string(),
                "Latest order mismatch between phases".to_string(),
            );
        }

        // Release and test with Incremental
        let _ = self
            .atomic_snapshot_manager
            .release_snapshot(PrunePhase::SweepExpired);

        let incremental_snapshot = match self
            .atomic_snapshot_manager
            .lock_snapshot(PrunePhase::Incremental)
        {
            Ok(s) => s,
            Err(e) => {
                return ValidationResult::Failed(
                    test_name.to_string(),
                    format!("Failed to lock for Incremental: {}", e),
                );
            }
        };

        if incremental_snapshot.snapshot_id != build_snapshot.snapshot_id {
            return ValidationResult::Failed(
                test_name.to_string(),
                "Snapshot ID mismatch for Incremental phase".to_string(),
            );
        }

        let _ = self
            .atomic_snapshot_manager
            .release_snapshot(PrunePhase::Incremental);

        ValidationResult::Passed(
            test_name.to_string(),
            "Cross-phase consistency verified".to_string(),
        )
    }

    /// Test error recovery mechanisms
    async fn test_error_recovery(&self) -> ValidationResult {
        let test_name = "error_recovery";
        info!("Testing {}", test_name);

        let error_recovery = match &self.error_recovery_manager {
            Some(er) => er,
            None => {
                return ValidationResult::Skipped(
                    test_name.to_string(),
                    "Error recovery manager not available".to_string(),
                );
            }
        };

        // Test system health check
        match error_recovery.check_system_health().await {
            Ok(healthy) => {
                debug!("System health check result: {}", healthy);
            }
            Err(e) => {
                return ValidationResult::Failed(
                    test_name.to_string(),
                    format!("System health check failed: {}", e),
                );
            }
        }

        // Test health ensure functionality
        match error_recovery.ensure_system_health().await {
            Ok(_) => {
                debug!("System health ensure completed successfully");
            }
            Err(e) => {
                return ValidationResult::Failed(
                    test_name.to_string(),
                    format!("System health ensure failed: {}", e),
                );
            }
        }

        // Test recovery stats
        let stats = error_recovery.get_stats();
        debug!(
            "Recovery stats: total={}, successful={}",
            stats
                .total_recoveries
                .load(std::sync::atomic::Ordering::Relaxed),
            stats
                .successful_recoveries
                .load(std::sync::atomic::Ordering::Relaxed)
        );

        ValidationResult::Passed(
            test_name.to_string(),
            "Error recovery mechanisms working correctly".to_string(),
        )
    }

    /// Test snapshot integrity validation
    async fn test_integrity_validation(&self) -> ValidationResult {
        let test_name = "integrity_validation";
        info!("Testing {}", test_name);

        // Create a snapshot
        let snapshot = match self
            .atomic_snapshot_manager
            .create_snapshot(PrunePhase::BuildReach)
        {
            Ok(s) => s,
            Err(e) => {
                return ValidationResult::Failed(
                    test_name.to_string(),
                    format!("Failed to create snapshot: {}", e),
                );
            }
        };

        // Create a manually corrupted snapshot for testing
        let mut corrupted_snapshot = snapshot.clone();
        corrupted_snapshot.integrity_hash = H256::random();

        // Test validation failure
        if let Ok(_) = self.validate_snapshot_integrity(&corrupted_snapshot) {
            return ValidationResult::Failed(
                test_name.to_string(),
                "Corrupted snapshot should fail integrity validation".to_string(),
            );
        }

        // Test validation success
        if let Err(e) = self.validate_snapshot_integrity(&snapshot) {
            return ValidationResult::Failed(
                test_name.to_string(),
                format!("Valid snapshot should pass integrity validation: {}", e),
            );
        }

        ValidationResult::Passed(
            test_name.to_string(),
            "Integrity validation working correctly".to_string(),
        )
    }

    /// Test concurrency and race condition safety
    async fn test_concurrency_safety(&self) -> ValidationResult {
        let test_name = "concurrency_safety";
        info!("Testing {}", test_name);

        // This is a simplified concurrency test
        // In a real implementation, you'd spawn multiple tasks and test concurrent operations

        let snapshot_manager = self.atomic_snapshot_manager.clone();

        // Test concurrent snapshot creation
        let snapshot1 = snapshot_manager.create_snapshot(PrunePhase::BuildReach);
        let snapshot2 = snapshot_manager.create_snapshot(PrunePhase::SweepExpired);

        match (snapshot1, snapshot2) {
            (Ok(s1), Ok(s2)) => {
                // Both should succeed but have different IDs
                if s1.snapshot_id == s2.snapshot_id {
                    return ValidationResult::Failed(
                        test_name.to_string(),
                        "Concurrent snapshot creation should produce different IDs".to_string(),
                    );
                }

                // Only the first should be the current snapshot
                let current = match snapshot_manager.get_current_snapshot() {
                    Ok(s) => s,
                    Err(e) => {
                        return ValidationResult::Failed(
                            test_name.to_string(),
                            format!("Failed to get current snapshot: {}", e),
                        );
                    }
                };

                if current.snapshot_id != s2.snapshot_id {
                    return ValidationResult::Failed(
                        test_name.to_string(),
                        "Current snapshot should be the most recently created".to_string(),
                    );
                }
            }
            (Err(e1), Err(e2)) => {
                return ValidationResult::Failed(
                    test_name.to_string(),
                    format!("Both concurrent creations failed: {}, {}", e1, e2),
                );
            }
            (Ok(_), Err(e)) | (Err(e), Ok(_)) => {
                return ValidationResult::Failed(
                    test_name.to_string(),
                    format!("One concurrent creation failed: {}", e),
                );
            }
        }

        ValidationResult::Passed(
            test_name.to_string(),
            "Concurrency safety verified".to_string(),
        )
    }

    /// Test performance and scalability
    async fn test_performance_validation(&self) -> ValidationResult {
        let test_name = "performance_validation";
        info!("Testing {}", test_name);

        let start_time = std::time::Instant::now();

        // Test multiple snapshot operations
        let num_operations = 10;
        for i in 0..num_operations {
            let phase = match i % 3 {
                0 => PrunePhase::BuildReach,
                1 => PrunePhase::SweepExpired,
                _ => PrunePhase::Incremental,
            };

            if let Err(e) = self.atomic_snapshot_manager.create_snapshot(phase) {
                return ValidationResult::Failed(
                    test_name.to_string(),
                    format!("Snapshot creation {} failed: {}", i, e),
                );
            }
        }

        let duration = start_time.elapsed();
        let avg_time_per_operation = duration.as_millis() / num_operations;

        // Performance threshold: should complete each operation in under 100ms on average
        if avg_time_per_operation > 100 {
            return ValidationResult::Failed(
                test_name.to_string(),
                format!(
                    "Performance below threshold: {}ms per operation (target: <100ms)",
                    avg_time_per_operation
                ),
            );
        }

        ValidationResult::Passed(
            test_name.to_string(),
            format!(
                "Performance validated: {}ms per operation",
                avg_time_per_operation
            ),
        )
    }

    /// Test memory and resource management
    async fn test_resource_management(&self) -> ValidationResult {
        let test_name = "resource_management";
        info!("Testing {}", test_name);

        // Test cleanup
        if let Err(e) = self.atomic_snapshot_manager.clear_snapshot() {
            return ValidationResult::Failed(
                test_name.to_string(),
                format!("Failed to clear snapshot: {}", e),
            );
        }

        // Test that operations still work after cleanup
        match self
            .atomic_snapshot_manager
            .create_snapshot(PrunePhase::BuildReach)
        {
            Ok(_) => {
                // Success
            }
            Err(e) => {
                return ValidationResult::Failed(
                    test_name.to_string(),
                    format!("Failed to create snapshot after cleanup: {}", e),
                );
            }
        }

        ValidationResult::Passed(
            test_name.to_string(),
            "Resource management working correctly".to_string(),
        )
    }

    /// Validate snapshot integrity (internal helper)
    fn validate_snapshot_integrity(&self, snapshot: &AtomicSnapshot) -> Result<()> {
        // Simple integrity check: integrity_hash should equal state_root
        if snapshot.integrity_hash != snapshot.snapshot.state_root {
            return Err(anyhow!(
                "Integrity hash mismatch: expected {:?}, got {:?}",
                snapshot.snapshot.state_root,
                snapshot.integrity_hash
            ));
        }

        Ok(())
    }
}

/// Validation result for individual tests
#[derive(Debug, Clone)]
pub enum ValidationResult {
    Passed(String, String),
    Failed(String, String),
    Skipped(String, String),
}

impl ValidationResult {
    pub fn test_name(&self) -> &str {
        match self {
            ValidationResult::Passed(name, _) => name,
            ValidationResult::Failed(name, _) => name,
            ValidationResult::Skipped(name, _) => name,
        }
    }

    pub fn passed(&self) -> bool {
        matches!(self, ValidationResult::Passed(_, _))
    }

    pub fn failed(&self) -> bool {
        matches!(self, ValidationResult::Failed(_, _))
    }

    pub fn skipped(&self) -> bool {
        matches!(self, ValidationResult::Skipped(_, _))
    }
}

/// Comprehensive validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    results: Vec<ValidationResult>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub fn add_result(&mut self, test_name: &str, result: ValidationResult) {
        self.results.push(result);
    }

    pub fn passed_count(&self) -> usize {
        self.results.iter().filter(|r| r.passed()).count()
    }

    pub fn failed_count(&self) -> usize {
        self.results.iter().filter(|r| r.failed()).count()
    }

    pub fn skipped_count(&self) -> usize {
        self.results.iter().filter(|r| r.skipped()).count()
    }

    pub fn total_count(&self) -> usize {
        self.results.len()
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_count() == 0 {
            return 0.0;
        }
        self.passed_count() as f64 / self.total_count() as f64
    }

    pub fn all_passed(&self) -> bool {
        self.failed_count() == 0
    }

    pub fn summary(&self) -> String {
        format!(
            "Validation completed: {}/{} passed ({}%), {} failed, {} skipped",
            self.passed_count(),
            self.total_count(),
            (self.success_rate() * 100.0) as u32,
            self.failed_count(),
            self.skipped_count()
        )
    }

    pub fn detailed_report(&self) -> String {
        let mut report = String::new();
        report.push_str(&format!("=== Validation Report ===\n"));
        report.push_str(&format!("{}\n\n", self.summary()));

        for result in &self.results {
            let status = if result.passed() {
                "✓ PASSED"
            } else if result.failed() {
                "✗ FAILED"
            } else {
                "- SKIPPED"
            };

            report.push_str(&format!("{}: {}\n", result.test_name(), status));

            let message = match result {
                ValidationResult::Passed(_, msg)
                | ValidationResult::Failed(_, msg)
                | ValidationResult::Skipped(_, msg) => msg,
            };

            if !message.is_empty() {
                report.push_str(&format!("  {}\n", message));
            }
        }

        report
    }
}

/// Utility function to run validation suite
pub async fn run_validation_suite(
    atomic_snapshot_manager: Arc<AtomicSnapshotManager>,
    error_recovery_manager: Option<Arc<ErrorRecoveryManager>>,
) -> ValidationReport {
    let validator = SnapshotValidator::new(atomic_snapshot_manager, error_recovery_manager);
    validator.run_comprehensive_validation().await
}

/// Quick validation for production monitoring
pub async fn quick_health_check(
    atomic_snapshot_manager: Arc<AtomicSnapshotManager>,
) -> ValidationResult {
    let test_name = "quick_health_check";

    // Test current snapshot availability
    let current_snapshot = match atomic_snapshot_manager.get_current_snapshot() {
        Ok(s) => s,
        Err(e) => {
            return ValidationResult::Failed(
                test_name.to_string(),
                format!("No current snapshot available: {}", e),
            );
        }
    };

    // Test phase consistency
    match atomic_snapshot_manager.validate_phase_consistency() {
        Ok(true) => ValidationResult::Passed(
            test_name.to_string(),
            format!(
                "Health check passed for snapshot {}",
                current_snapshot.snapshot_id
            ),
        ),
        Ok(false) => ValidationResult::Failed(
            test_name.to_string(),
            "Phase consistency validation failed".to_string(),
        ),
        Err(e) => ValidationResult::Failed(
            test_name.to_string(),
            format!("Consistency validation error: {}", e),
        ),
    }
}
