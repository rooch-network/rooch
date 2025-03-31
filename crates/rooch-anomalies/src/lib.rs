// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use include_dir::{include_dir, Dir};
use moveos_types::h256::H256;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tracing::info;

const STATIC_ANOMALIES_DIR: Dir = include_dir!("static");

pub fn load_tx_anomalies(genesis_namespace: String) -> anyhow::Result<Option<TxAnomalies>> {
    let tx_anomalies_opt = STATIC_ANOMALIES_DIR
        .get_file(&genesis_namespace)
        .map(|file| TxAnomalies::decode(file.contents()))
        .transpose()?;

    if let Some(ref anomalies) = tx_anomalies_opt {
        info!("Loaded tx anomalies: {}", anomalies.summary());
    }

    Ok(tx_anomalies_opt)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxAnomalies {
    pub genesis_namespace: String,
    pub dup_hash: HashMap<H256, Vec<u64>>,
    pub no_execution_info: HashMap<H256, u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accumulator_should_revert: Option<HashMap<u64, H256>>,
}

impl TxAnomalies {
    pub fn summary(&self) -> String {
        let dup_count = self.dup_hash.len();
        let no_execution_count = self.no_execution_info.len();
        let revert_count = self
            .accumulator_should_revert
            .as_ref()
            .map_or(0, |map| map.len());
        format!(
            "Namespace: {}, Dup count: {}, No execution count: {}, Accumulator Should Revert count: {}",
            self.genesis_namespace, dup_count, no_execution_count, revert_count
        )
    }

    pub fn is_dup_hash(&self, hash: &H256) -> bool {
        self.dup_hash.contains_key(hash)
    }

    pub fn get_accumulator_should_revert(&self, order: u64) -> Option<H256> {
        self.accumulator_should_revert
            .as_ref()
            .and_then(|map| map.get(&order).cloned())
    }

    pub fn has_no_execution_info(&self, hash: &H256) -> bool {
        self.no_execution_info.contains_key(hash)
    }

    pub fn get_genesis_namespace(&self) -> String {
        self.genesis_namespace.clone()
    }

    pub fn decode(bytes: &[u8]) -> anyhow::Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }

    pub fn encode(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("TxAnomalies bcs::to_bytes should success")
    }

    pub fn load_from<P: AsRef<Path>>(anomalies_file: P) -> anyhow::Result<Self> {
        let file_path = anomalies_file.as_ref();
        let anomalies_from_bcs_ret = bcs::from_bytes(&std::fs::read(file_path)?);
        let anomalies: TxAnomalies = match anomalies_from_bcs_ret {
            Ok(anomalies_from_bcs) => anomalies_from_bcs,
            Err(_) => {
                let anomalies_from_json_ret = serde_json::from_slice(&std::fs::read(file_path)?);
                match anomalies_from_json_ret {
                    Ok(anomalies_from_json) => anomalies_from_json,
                    Err(_) => return Err(anyhow::anyhow!("Failed to load anomalies from file")),
                }
            }
        };

        info!("Loaded tx anomalies: {}", anomalies.summary());

        Ok(anomalies)
    }

    pub fn save_to<P: AsRef<Path>>(&self, anomalies_file: P) -> anyhow::Result<()> {
        let mut file = File::create(anomalies_file)?;
        let contents = bcs::to_bytes(&self)?;
        file.write_all(&contents)?;
        file.sync_data()?;
        Ok(())
    }

    pub fn save_plain_text_to<P: AsRef<Path>>(&self, anomalies_file: P) -> anyhow::Result<()> {
        let mut file = File::create(anomalies_file)?;
        let contents = serde_json::to_string_pretty(&self)?;
        file.write_all(contents.as_bytes())?;
        file.sync_data()?;
        Ok(())
    }
}

/// A helper struct to map transaction orders to their actual indices in the accumulator.
/// Due to bugs in the revert mechanism, there are transactions that should have been reverted
/// but still occupy indices in the accumulator, causing a gap between order and index.
#[derive(Debug, Clone)]
pub struct AccumulatorIndexMapper {
    /// Sorted list of orders that should have been reverted
    reverted_orders: Vec<u64>,
}

impl AccumulatorIndexMapper {
    /// Create a new AccumulatorIndexMapper from a TxAnomalies struct
    pub fn new(tx_anomalies: Option<TxAnomalies>) -> Self {
        if tx_anomalies.is_none() {
            return Self {
                reverted_orders: vec![],
            };
        }

        let tx_anomalies = tx_anomalies.unwrap();
        let should_have_reverted = tx_anomalies
            .accumulator_should_revert
            .clone()
            .unwrap_or_default();
        let mut reverted_orders: Vec<u64> = should_have_reverted.keys().cloned().collect();
        reverted_orders.sort(); // Ensure orders are sorted

        Self { reverted_orders }
    }

    /// Get the actual index in the accumulator for a given transaction order.
    ///
    /// When an order should have been reverted, it creates two indices in the accumulator:
    /// 1. The index occupied by the reverted transaction
    /// 2. The index where the valid transaction with the same order is placed
    ///
    /// This method calculates the correct index for valid transactions.
    pub fn get_index_for_order(&self, order: u64) -> u64 {
        // Each order that should have been reverted occupies TWO indices:
        // 1. The index for the transaction that should have been reverted
        // 2. The index for the valid transaction with the same order

        // Count all "should have reverted" transactions, including the current one if applicable
        let reverted_tx_count = self
            .reverted_orders
            .iter()
            .filter(|&&reverted_order| reverted_order <= order)
            .count() as u64;

        // The index is order + reverted_tx_count
        order + reverted_tx_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::HashMap;

    // Helper function to create TxAnomalies with specific reverted orders
    fn create_tx_anomalies_with_reverted(reverted_orders: &[u64]) -> TxAnomalies {
        let mut accumulator_should_revert = HashMap::new();
        for &order in reverted_orders {
            // Using a fake H256 value for testing
            accumulator_should_revert.insert(order, H256::zero());
        }

        TxAnomalies {
            genesis_namespace: "test".to_string(),
            dup_hash: HashMap::new(),
            no_execution_info: HashMap::new(),
            accumulator_should_revert: Some(accumulator_should_revert),
        }
    }

    #[test]
    fn test_empty_mapper() {
        let mapper = AccumulatorIndexMapper::new(None);
        assert_eq!(mapper.reverted_orders.len(), 0);
        assert_eq!(mapper.get_index_for_order(0), 0);
        assert_eq!(mapper.get_index_for_order(10), 10);
    }

    #[test]
    fn test_empty_reverts() {
        let anomalies = create_tx_anomalies_with_reverted(&[]);
        let mapper = AccumulatorIndexMapper::new(Some(anomalies));
        assert_eq!(mapper.reverted_orders.len(), 0);
        assert_eq!(mapper.get_index_for_order(0), 0);
        assert_eq!(mapper.get_index_for_order(100), 100);
    }

    #[test]
    fn test_single_revert() {
        let anomalies = create_tx_anomalies_with_reverted(&[5]);
        let mapper = AccumulatorIndexMapper::new(Some(anomalies));

        assert_eq!(mapper.reverted_orders, vec![5]);

        // Orders before the revert
        assert_eq!(mapper.get_index_for_order(0), 0);
        assert_eq!(mapper.get_index_for_order(4), 4);

        // The reverted order itself
        assert_eq!(mapper.get_index_for_order(5), 6); // 5 + 1 revert

        // Orders after the revert
        assert_eq!(mapper.get_index_for_order(6), 7); // 6 + 1 revert
        assert_eq!(mapper.get_index_for_order(10), 11); // 10 + 1 revert
    }

    #[test]
    fn test_multiple_reverts() {
        let anomalies = create_tx_anomalies_with_reverted(&[3, 7, 10]);
        let mapper = AccumulatorIndexMapper::new(Some(anomalies));

        assert_eq!(mapper.reverted_orders, vec![3, 7, 10]);

        // Orders before any revert
        assert_eq!(mapper.get_index_for_order(0), 0);
        assert_eq!(mapper.get_index_for_order(2), 2);

        // Order at first revert point
        assert_eq!(mapper.get_index_for_order(3), 4); // 3 + 1 revert

        // Orders between first and second revert
        assert_eq!(mapper.get_index_for_order(4), 5); // 4 + 1 revert
        assert_eq!(mapper.get_index_for_order(6), 7); // 6 + 1 revert

        // Order at second revert point
        assert_eq!(mapper.get_index_for_order(7), 9); // 7 + 2 reverts

        // Orders between second and third revert
        assert_eq!(mapper.get_index_for_order(8), 10); // 8 + 2 reverts
        assert_eq!(mapper.get_index_for_order(9), 11); // 9 + 2 reverts

        // Order at third revert point
        assert_eq!(mapper.get_index_for_order(10), 13); // 10 + 3 reverts

        // Orders after all reverts
        assert_eq!(mapper.get_index_for_order(11), 14); // 11 + 3 reverts
        assert_eq!(mapper.get_index_for_order(100), 103); // 100 + 3 reverts
    }

    #[test]
    fn test_consecutive_reverts() {
        let anomalies = create_tx_anomalies_with_reverted(&[5, 6, 7]);
        let mapper = AccumulatorIndexMapper::new(Some(anomalies));

        assert_eq!(mapper.reverted_orders, vec![5, 6, 7]);

        assert_eq!(mapper.get_index_for_order(4), 4);
        assert_eq!(mapper.get_index_for_order(5), 6); // 5 + 1 revert
        assert_eq!(mapper.get_index_for_order(6), 8); // 6 + 2 reverts
        assert_eq!(mapper.get_index_for_order(7), 10); // 7 + 3 reverts
        assert_eq!(mapper.get_index_for_order(8), 11); // 8 + 3 reverts
    }

    #[test]
    fn test_unsorted_reverts_input() {
        // Create TxAnomalies with unsorted reverted orders (should be sorted by AccumulatorIndexMapper)
        let mut accumulator_should_revert = HashMap::new();
        for &order in &[10, 3, 7] {
            // Deliberately unsorted
            accumulator_should_revert.insert(order, H256::zero());
        }

        let anomalies = TxAnomalies {
            genesis_namespace: "test".to_string(),
            dup_hash: HashMap::new(),
            no_execution_info: HashMap::new(),
            accumulator_should_revert: Some(accumulator_should_revert),
        };

        let mapper = AccumulatorIndexMapper::new(Some(anomalies));

        // Should be sorted internally
        assert_eq!(mapper.reverted_orders, vec![3, 7, 10]);

        // Verify calculations are correct
        assert_eq!(mapper.get_index_for_order(3), 4); // 3 + 1 revert
        assert_eq!(mapper.get_index_for_order(7), 9); // 7 + 2 reverts
        assert_eq!(mapper.get_index_for_order(10), 13); // 10 + 3 reverts
    }

    #[test]
    fn test_large_orders() {
        let anomalies = create_tx_anomalies_with_reverted(&[1_000_000, 2_000_000]);
        let mapper = AccumulatorIndexMapper::new(Some(anomalies));

        assert_eq!(mapper.get_index_for_order(999_999), 999_999);
        assert_eq!(mapper.get_index_for_order(1_000_000), 1_000_001);
        assert_eq!(mapper.get_index_for_order(1_500_000), 1_500_001);
        assert_eq!(mapper.get_index_for_order(2_000_000), 2_000_002);
        assert_eq!(mapper.get_index_for_order(3_000_000), 3_000_002);
    }

    #[test]
    fn test_existed_anomalies() {
        let tx_anomalies = load_tx_anomalies("527d69c3".to_string()).unwrap();
        let mapper = AccumulatorIndexMapper::new(tx_anomalies);

        assert_eq!(mapper.get_index_for_order(0), 0);
        assert_eq!(mapper.get_index_for_order(1), 1);
        assert_eq!(mapper.get_index_for_order(98795306), 98795307);
        assert_eq!(mapper.get_index_for_order(98796093), 98796095);
        assert_eq!(mapper.get_index_for_order(98850541), 98850544);
        assert_eq!(mapper.get_index_for_order(99067968), 99067972);
        assert_eq!(mapper.get_index_for_order(99069924), 99069929);
        assert_eq!(mapper.get_index_for_order(99075174), 99075180);
        assert_eq!(mapper.get_index_for_order(99077164), 99077171);
        assert_eq!(mapper.get_index_for_order(99267679), 99267687);
        assert_eq!(mapper.get_index_for_order(99272004), 99272013);
        assert_eq!(mapper.get_index_for_order(99295404), 99295414);
        assert_eq!(mapper.get_index_for_order(99310232), 99310243);
        assert_eq!(mapper.get_index_for_order(99310233), 99310244);
        assert_eq!(mapper.get_index_for_order(99310234), 99310245);
    }

    // Property-based tests using proptest
    proptest! {
        #[test]
        fn prop_mapper_with_no_reverts(order in 0u64..1000u64) {
            let mapper = AccumulatorIndexMapper::new(None);
            prop_assert_eq!(mapper.get_index_for_order(order), order);
        }

        #[test]
        fn prop_orders_increase_monotonically(
            reverted_orders in prop::collection::vec(0u64..100, 0..10),
            test_orders in prop::collection::vec(0u64..200, 1..20)
        ) {
            let unique_reverted: Vec<u64> = reverted_orders.into_iter().collect::<std::collections::HashSet<_>>().into_iter().collect();
            let anomalies = create_tx_anomalies_with_reverted(&unique_reverted);
            let mapper = AccumulatorIndexMapper::new(Some(anomalies));

            for window in test_orders.windows(2) {
                if window[0] < window[1] {
                    let idx1 = mapper.get_index_for_order(window[0]);
                    let idx2 = mapper.get_index_for_order(window[1]);
                    prop_assert!(idx1 < idx2, "Index should increase for increasing orders");
                }
            }
        }

        #[test]
        fn prop_index_always_greater_than_order(
            reverted_orders in prop::collection::vec(0u64..100, 0..10),
            test_order in 0u64..200
        ) {
            let unique_reverted: Vec<u64> = reverted_orders.into_iter().collect::<std::collections::HashSet<_>>().into_iter().collect();
            let anomalies = create_tx_anomalies_with_reverted(&unique_reverted);
            let mapper = AccumulatorIndexMapper::new(Some(anomalies));

            let revert_count = unique_reverted.iter().filter(|&&x| x <= test_order).count() as u64;
            let expected_index = test_order + revert_count;

            prop_assert_eq!(mapper.get_index_for_order(test_order), expected_index);
        }
    }
}
