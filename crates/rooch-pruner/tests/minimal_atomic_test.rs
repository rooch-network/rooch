// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    #[test]
    fn test_atomic_snapshot_compilation() {
        // This test just verifies that the atomic snapshot code compiles successfully
        use rooch_pruner::atomic_snapshot::SnapshotManagerConfig;

        let config = SnapshotManagerConfig {
            lock_timeout_ms: 5000,
            max_snapshot_age_ms: 30000,
            enable_validation: true,
            enable_persistence: false,
        };

        assert_eq!(config.lock_timeout_ms, 5000);
        assert!(config.enable_validation);

        println!("✅ Atomic snapshot compilation test passed");
    }
}
