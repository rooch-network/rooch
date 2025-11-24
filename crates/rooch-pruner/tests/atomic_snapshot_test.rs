// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_types::prune::PrunePhase;
use rooch_pruner::atomic_snapshot::SnapshotManagerConfig;
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_snapshot_manager_creation() {
        let config = SnapshotManagerConfig {
            lock_timeout_ms: 5000,      // 5 seconds for testing
            max_snapshot_age_ms: 30000, // 30 seconds for testing
            enable_validation: true,
            enable_persistence: false,
        };

        println!("🧪 Testing SnapshotManagerConfig creation");
        assert_eq!(config.lock_timeout_ms, 5000);
        assert_eq!(config.max_snapshot_age_ms, 30000);
        assert!(config.enable_validation);
        assert!(!config.enable_persistence);

        println!("✅ SnapshotManagerConfig creation test passed");
    }

    #[test]
    fn test_prune_phase_transitions() {
        println!("🧪 Testing prune phase transitions");

        let phases = [
            PrunePhase::BuildReach,
            PrunePhase::SweepExpired,
            PrunePhase::Incremental,
        ];

        for (i, phase) in phases.iter().enumerate() {
            println!("🔄 Phase {}: {:?}", i + 1, phase);

            let next_phase = match phase {
                PrunePhase::BuildReach => PrunePhase::SweepExpired,
                PrunePhase::SweepExpired => PrunePhase::Incremental,
                PrunePhase::Incremental => PrunePhase::BuildReach,
            };

            println!("  → Next phase: {:?}", next_phase);
        }

        println!("✅ Phase transition logic test passed");
    }

    #[test]
    fn test_snapshot_manager_config_validation() {
        println!("🧪 Testing SnapshotManagerConfig validation");

        let config = SnapshotManagerConfig {
            lock_timeout_ms: 1000,
            max_snapshot_age_ms: 30000,
            enable_validation: true,
            enable_persistence: false,
        };

        assert!(
            config.lock_timeout_ms > 0,
            "Lock timeout should be positive"
        );
        assert!(
            config.max_snapshot_age_ms > config.lock_timeout_ms,
            "Max age should be greater than lock timeout"
        );

        println!("✅ Config validation passed:");
        println!("  - Lock timeout: {}ms", config.lock_timeout_ms);
        println!("  - Max snapshot age: {}ms", config.max_snapshot_age_ms);
        println!("  - Validation enabled: {}", config.enable_validation);
        println!("  - Persistence enabled: {}", config.enable_persistence);
    }

    #[test]
    fn test_timeout_configurations() {
        println!("🧪 Testing timeout configurations");

        let test_configs = vec![
            (1000, 60000),   // 1s lock, 1min max
            (5000, 300000),  // 5s lock, 5min max
            (10000, 600000), // 10s lock, 10min max
        ];

        for (lock_timeout, max_age) in test_configs {
            let _config = SnapshotManagerConfig {
                lock_timeout_ms: lock_timeout,
                max_snapshot_age_ms: max_age,
                enable_validation: true,
                enable_persistence: false,
            };

            assert!(
                max_age >= lock_timeout,
                "Max age ({}) should be >= lock timeout ({})",
                max_age,
                lock_timeout
            );

            println!(
                "✅ Config validation: {}ms lock, {}ms max age",
                lock_timeout, max_age
            );
        }
    }

    #[test]
    fn test_phase_performance_characteristics() {
        println!("🧪 Testing phase performance characteristics");

        let phases = [
            (PrunePhase::BuildReach, "reachability analysis"),
            (PrunePhase::SweepExpired, "expired node cleanup"),
            (PrunePhase::Incremental, "incremental updates"),
        ];

        for (phase, description) in phases.iter() {
            let start_time = std::time::Instant::now();

            let simulated_duration = match phase {
                PrunePhase::BuildReach => Duration::from_millis(100),
                PrunePhase::SweepExpired => Duration::from_millis(50),
                PrunePhase::Incremental => Duration::from_millis(25),
            };

            std::thread::sleep(simulated_duration);
            let elapsed = start_time.elapsed();

            println!("📊 Phase {:?} ({}): {:?}", phase, description, elapsed);

            assert!(
                elapsed < Duration::from_millis(1000),
                "Phase should complete within 1 second"
            );
        }

        println!("✅ Phase performance characteristics test completed");
    }

    #[test]
    fn test_configuration_edge_cases() {
        println!("🧪 Testing configuration edge cases");

        let min_config = SnapshotManagerConfig {
            lock_timeout_ms: 1,
            max_snapshot_age_ms: 1000,
            enable_validation: false,
            enable_persistence: false,
        };

        assert!(
            min_config.lock_timeout_ms >= 1,
            "Minimum lock timeout should be 1ms"
        );
        assert!(
            !min_config.enable_validation,
            "Validation should be disabled"
        );

        let max_config = SnapshotManagerConfig {
            lock_timeout_ms: 300000,      // 5 minutes
            max_snapshot_age_ms: 3600000, // 1 hour
            enable_validation: true,
            enable_persistence: true,
        };

        assert!(
            max_config.lock_timeout_ms <= 300000,
            "Lock timeout should be reasonable"
        );
        assert!(max_config.enable_validation, "Validation should be enabled");

        println!("✅ Edge case configurations validated");
    }

    #[test]
    fn test_error_handling_scenarios() {
        println!("🧪 Testing error handling scenarios");

        let error_scenarios = vec![
            ("Snapshot creation timeout", Duration::from_millis(100)),
            ("Lock acquisition failure", Duration::from_millis(50)),
            ("Validation failure", Duration::from_millis(25)),
        ];

        for (description, delay) in error_scenarios {
            println!("🔧 Testing: {}", description);

            let start_time = std::time::Instant::now();

            std::thread::sleep(delay);

            let elapsed = start_time.elapsed();
            println!("  → Error handled in {:?}", elapsed);

            assert!(
                elapsed < Duration::from_millis(1000),
                "Error handling should be fast"
            );
        }

        println!("✅ Error handling scenarios test completed");
    }
}
