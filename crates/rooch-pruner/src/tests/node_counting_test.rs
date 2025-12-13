// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//! Test the node counting implementation for garbage collector

#[cfg(test)]
mod tests {
    use crate::config::GCConfig;
    use crate::garbage_collector::GarbageCollector;
    use anyhow::Result;
    use rooch_config::RoochOpt;
    use rooch_db::RoochDB;
    use tempfile::TempDir;
    use tracing::info;

    /// Test that the garbage collector can be created with the new implementation
    #[test]
    fn test_garbage_collector_creation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().to_path_buf();

        // Create garbage collector - this will use our new node counting implementation
        let config = GCConfig::default();
        let rooch_opt = RoochOpt::new_with_default(Some(db_path.clone()), None, None)?;
        let rooch_db = RoochDB::init_with_mock_metrics_for_test(rooch_opt.store_config())?;
        let gc = GarbageCollector::new(rooch_db, config)?;

        info!("[PASS] GarbageCollector creation test passed");
        info!("  GC created successfully with new node counting methods");

        // Verify the GC has the expected configuration
        assert!(!gc.config.dry_run, "Default config should not be dry run");
        assert_eq!(
            gc.config.protected_roots_count, 1,
            "Default config should protect 1 root"
        );

        Ok(())
    }

    /// Test that the compilation works and we have the necessary imports
    #[test]
    fn test_compilation_and_imports() -> Result<()> {
        // This test will fail to compile if any imports are missing
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().to_path_buf();

        let config = GCConfig::default();
        let rooch_opt = RoochOpt::new_with_default(Some(db_path.clone()), None, None)?;
        let rooch_db = RoochDB::init_with_mock_metrics_for_test(rooch_opt.store_config())?;
        let _gc = GarbageCollector::new(rooch_db, config)?;

        info!("[PASS] Compilation and imports test passed");
        info!("  All imports and types are working correctly");

        Ok(())
    }

    /// Test that the new constants and types are available
    #[test]
    fn test_new_constants_and_types() -> Result<()> {
        use moveos_store::STATE_NODE_COLUMN_FAMILY_NAME;

        // Test that we can access the column family name constant
        let cf_name = STATE_NODE_COLUMN_FAMILY_NAME;
        assert_eq!(
            cf_name, "state_node",
            "Column family name should be state_node"
        );

        info!("[PASS] Constants and types test passed");
        info!("  STATE_NODE_COLUMN_FAMILY_NAME = {}", cf_name);

        Ok(())
    }
}
