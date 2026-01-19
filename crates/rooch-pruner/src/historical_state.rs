// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//! Historical state collection for multi-root GC protection

use anyhow::Result;
use moveos_store::transaction_store::TransactionStore as MoveOSTransactionStore;
use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use moveos_types::transaction::TransactionExecutionInfo;
use rooch_store::transaction_store::TransactionStore as RoochTransactionStore;
use rooch_store::RoochStore;
use rooch_types::framework::chain_id::ChainID;
use rooch_types::rooch_network::BuiltinChainID;
use tracing::{debug, info, warn};

/// Configuration for historical state collection
#[derive(Debug, Clone)]
pub struct HistoricalStateConfig {
    /// Number of recent state roots to protect from GC
    pub protected_roots_count: usize,
}

impl HistoricalStateConfig {
    /// Create config with network-aware default values
    pub fn new_with_network(chain_id: &ChainID) -> Self {
        let protected_roots_count = if let Some(builtin_id) = chain_id.to_builtin() {
            match builtin_id {
                BuiltinChainID::Local => 1,    // Local environment - minimal protection
                BuiltinChainID::Dev => 1000,   // Development - more protection for testing
                BuiltinChainID::Test => 1000,  // Test network - more protection
                BuiltinChainID::Main => 30000, // Main network - maximum protection
            }
        } else {
            // Custom chain ID - use a reasonable default
            1000
        };

        Self {
            protected_roots_count,
        }
    }

    /// Create config for local development (backward compatibility)
    pub fn new_local() -> Self {
        Self {
            protected_roots_count: 1,
        }
    }
}

impl Default for HistoricalStateConfig {
    fn default() -> Self {
        Self::new_local() // Default to local environment for backward compatibility
    }
}

/// Historical state root collector for multi-root GC protection
///
/// Collects recent state roots from transaction execution info to provide
/// time-window based protection in garbage collection.
pub struct HistoricalStateCollector {
    moveos_store: MoveOSStore,
    rooch_store: RoochStore,
    config: HistoricalStateConfig,
}

impl HistoricalStateCollector {
    /// Create a new historical state collector with required stores
    pub fn new(
        moveos_store: MoveOSStore,
        rooch_store: RoochStore,
        config: HistoricalStateConfig,
    ) -> Self {
        Self {
            moveos_store,
            rooch_store,
            config,
        }
    }

    /// Collect recent state roots for GC protection
    pub fn collect_recent_state_roots(&self) -> Result<Vec<H256>> {
        if self.config.protected_roots_count == 0 {
            return Ok(vec![]);
        }

        info!(
            "Collecting {} recent state roots",
            self.config.protected_roots_count
        );

        let roots = self.get_state_roots_from_execution_info(self.config.protected_roots_count)?;

        info!(
            "Successfully collected {} state roots for protection",
            roots.len()
        );
        for (i, root) in roots.iter().enumerate() {
            debug!("  Root {}: {}", i, root);
        }

        Ok(roots)
    }

    /// Get state roots from transaction execution info
    fn get_state_roots_from_execution_info(&self, count: usize) -> Result<Vec<H256>> {
        let latest_tx_order = self.get_latest_transaction_order()?;

        let tx_hashes = self.get_recent_transaction_hashes(latest_tx_order, count)?;
        let execution_infos = self.get_execution_infos_batch(&tx_hashes)?;

        Ok(self.extract_state_roots(execution_infos, count))
    }

    /// Get the latest transaction order
    fn get_latest_transaction_order(&self) -> Result<u64> {
        self.rooch_store
            .get_meta_store()
            .get_sequencer_info()
            .map_err(|e| anyhow::anyhow!("Failed to get sequencer info: {}", e))?
            .map(|info| info.last_order)
            .ok_or_else(|| anyhow::anyhow!("No sequencer info available"))
    }

    /// Get recent transaction hashes
    fn get_recent_transaction_hashes(&self, latest_order: u64, count: usize) -> Result<Vec<H256>> {
        let start_order = latest_order.saturating_sub(count as u64 - 1);
        let limit = latest_order - start_order + 1;

        let tx_hash_options = self
            .rooch_store
            .get_tx_hashes_by_order(Some(start_order), limit)
            .map_err(|e| anyhow::anyhow!("Failed to get transaction hashes: {}", e))?;

        Ok(tx_hash_options.into_iter().flatten().collect())
    }

    /// Get execution infos in batch
    fn get_execution_infos_batch(
        &self,
        tx_hashes: &[H256],
    ) -> Result<Vec<TransactionExecutionInfo>> {
        let mut infos = Vec::new();

        for tx_hash in tx_hashes {
            match self
                .moveos_store
                .transaction_store
                .get_tx_execution_info(*tx_hash)
            {
                Ok(Some(info)) => infos.push(info),
                Ok(None) => debug!("No execution info for tx: {}", tx_hash),
                Err(e) => warn!("Failed to get execution info for tx {}: {}", tx_hash, e),
            }
        }

        Ok(infos)
    }

    /// Extract state roots from execution infos (newest to oldest)
    fn extract_state_roots(
        &self,
        infos: Vec<TransactionExecutionInfo>,
        max_count: usize,
    ) -> Vec<H256> {
        // Process in reverse order (newest to oldest)
        infos
            .into_iter()
            .rev()
            .map(|info| info.state_root)
            .take(max_count)
            .collect()
    }
}

#[cfg(feature = "gc-tests")]
mod tests {
    use super::*;
    use rooch_types::framework::chain_id::ChainID;
    use rooch_types::rooch_network::BuiltinChainID;

    #[test]
    fn test_historical_state_config_default() {
        let config = HistoricalStateConfig::default();
        assert_eq!(config.protected_roots_count, 1); // Local default
    }

    #[test]
    fn test_network_aware_config() {
        // Test local network
        let local_chain_id = ChainID::from(BuiltinChainID::Local);
        let local_config = HistoricalStateConfig::new_with_network(&local_chain_id);
        assert_eq!(local_config.protected_roots_count, 1);

        // Test dev network
        let dev_chain_id = ChainID::from(BuiltinChainID::Dev);
        let dev_config = HistoricalStateConfig::new_with_network(&dev_chain_id);
        assert_eq!(dev_config.protected_roots_count, 1000);

        // Test test network
        let test_chain_id = ChainID::from(BuiltinChainID::Test);
        let test_config = HistoricalStateConfig::new_with_network(&test_chain_id);
        assert_eq!(test_config.protected_roots_count, 1000);

        // Test main network
        let main_chain_id = ChainID::from(BuiltinChainID::Main);
        let main_config = HistoricalStateConfig::new_with_network(&main_chain_id);
        assert_eq!(main_config.protected_roots_count, 30000);
    }

    #[test]
    fn test_custom_config() {
        // Test custom config (manual override)
        let custom_config = HistoricalStateConfig {
            protected_roots_count: 5000,
        };
        assert_eq!(custom_config.protected_roots_count, 5000);

        // Test custom chain ID (should use reasonable default)
        let custom_chain_id = ChainID::new(999);
        let custom_config = HistoricalStateConfig::new_with_network(&custom_chain_id);
        assert_eq!(custom_config.protected_roots_count, 1000);
    }

    #[test]
    fn test_new_local() {
        let local_config = HistoricalStateConfig::new_local();
        assert_eq!(local_config.protected_roots_count, 1);
    }
}
