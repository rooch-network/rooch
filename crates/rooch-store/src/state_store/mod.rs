// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::STATE_CHANGE_SET_COLUMN_FAMILY_NAME;
use anyhow::Result;
use moveos_types::state::StateChangeSetExt;
use raw_store::CodecKVStore;
use raw_store::{derive_store, StoreInstance};

derive_store!(
    StateChangeSetStore,
    u64,
    StateChangeSetExt,
    STATE_CHANGE_SET_COLUMN_FAMILY_NAME
);

pub trait StateStore {
    fn save_state_change_set(
        &self,
        tx_order: u64,
        state_change_set: StateChangeSetExt,
    ) -> Result<()>;
    fn get_state_change_set(&self, tx_order: u64) -> Result<Option<StateChangeSetExt>>;
    fn multi_get_state_change_set(
        &self,
        tx_orders: Vec<u64>,
    ) -> Result<Vec<Option<StateChangeSetExt>>>;
    fn remove_state_change_set(&self, tx_order: u64) -> Result<()>;

    fn check_state_change_set(&self, tx_orders: Vec<u64>) -> Result<Vec<u64>>;

    /// Get a range of state changesets from `from_order` (inclusive) to `to_order` (exclusive)
    fn get_changesets_range(
        &self,
        from_order: u64,
        to_order: u64,
    ) -> Result<Vec<(u64, StateChangeSetExt)>>;

    /// Get a range of state changesets with a limit on the number of results
    fn get_changesets_range_with_limit(
        &self,
        from_order: u64,
        to_order: u64,
        limit: usize,
    ) -> Result<Vec<(u64, StateChangeSetExt)>>;
}

#[derive(Clone)]
pub struct StateDBStore {
    state_change_set_store: StateChangeSetStore,
}

impl StateDBStore {
    pub fn new(instance: StoreInstance) -> Self {
        StateDBStore {
            state_change_set_store: StateChangeSetStore::new(instance.clone()),
        }
    }

    #[cfg(test)]
    fn insert_dummy_changesets(&self, start: u64, count: u64) {
        for i in 0..count {
            let order = start + i;
            let dummy = StateChangeSetExt {
                state_change_set: moveos_types::state::StateChangeSet::new(
                    moveos_types::h256::H256::from_low_u64_be(order),
                    order,
                ),
                sequence_number: order,
            };
            self.save_state_change_set(order, dummy).unwrap();
        }
    }

    pub fn save_state_change_set(
        &self,
        tx_order: u64,
        state_change_set: StateChangeSetExt,
    ) -> Result<()> {
        self.state_change_set_store
            .kv_put(tx_order, state_change_set)
    }

    pub fn get_state_change_set(&self, tx_order: u64) -> Result<Option<StateChangeSetExt>> {
        self.state_change_set_store.kv_get(tx_order)
    }

    pub fn multi_get_state_change_set(
        &self,
        tx_orders: Vec<u64>,
    ) -> Result<Vec<Option<StateChangeSetExt>>> {
        self.state_change_set_store.multiple_get(tx_orders)
    }

    pub fn remove_state_change_set(&self, tx_order: u64) -> Result<()> {
        self.state_change_set_store.remove(tx_order)
    }

    pub fn check_state_change_set(&self, tx_orders: Vec<u64>) -> Result<Vec<u64>> {
        let values = self
            .state_change_set_store
            .multiple_get_raw(tx_orders.clone())?;

        let missing_tx_orders = tx_orders
            .into_iter()
            .zip(values)
            .filter_map(|(k, v)| if v.is_none() { Some(k) } else { None })
            .collect::<Vec<_>>();
        Ok(missing_tx_orders)
    }

    /// Get a range of state changesets from `from_order` (inclusive) to `to_order` (exclusive)
    pub fn get_changesets_range(
        &self,
        from_order: u64,
        to_order: u64,
    ) -> Result<Vec<(u64, StateChangeSetExt)>> {
        self.get_changesets_range_with_limit(from_order, to_order, usize::MAX)
    }

    /// Get a range of state changesets with a limit on the number of results
    pub fn get_changesets_range_with_limit(
        &self,
        from_order: u64,
        to_order: u64,
        limit: usize,
    ) -> Result<Vec<(u64, StateChangeSetExt)>> {
        if from_order >= to_order {
            return Ok(Vec::new());
        }

        // NOTE: State change set keys are BCS-encoded u64 (little-endian). RocksDB's
        // default lexicographic iterator order does not align with numeric order for
        // little-endian encoded integers. Using a forward iterator with range break
        // logic therefore skips most entries in a numeric span (observed: only
        // 18/4554 changesets loaded in production).
        // To guarantee correctness across existing databases, fall back to explicit
        // multi_get over the requested numeric range. Although slightly slower, the
        // range sizes we replay (thousands) are acceptable and correctness is critical.
        self.get_changesets_range_batch_fallback(from_order, to_order, limit)
    }

    /// Fallback implementation for non-RocksDB stores using batch queries
    fn get_changesets_range_batch_fallback(
        &self,
        from_order: u64,
        to_order: u64,
        limit: usize,
    ) -> Result<Vec<(u64, StateChangeSetExt)>> {
        // For now, implement a simple approach: query in batches
        let batch_size = std::cmp::min(1000, limit);
        let mut results = Vec::new();
        let mut current_order = from_order;

        while current_order < to_order && results.len() < limit {
            let end_order = std::cmp::min(current_order + batch_size as u64, to_order);
            let orders: Vec<u64> = (current_order..end_order).collect();

            let changesets = self.multi_get_state_change_set(orders)?;

            for (i, changeset) in changesets.into_iter().enumerate() {
                if let Some(cs) = changeset {
                    let tx_order = current_order + i as u64;
                    if tx_order < to_order {
                        results.push((tx_order, cs));
                        if results.len() >= limit {
                            break;
                        }
                    }
                }
            }

            current_order = end_order;
        }

        Ok(results)
    }
}

impl StateStore for StateDBStore {
    fn save_state_change_set(
        &self,
        tx_order: u64,
        state_change_set: StateChangeSetExt,
    ) -> Result<()> {
        self.save_state_change_set(tx_order, state_change_set)
    }

    fn get_state_change_set(&self, tx_order: u64) -> Result<Option<StateChangeSetExt>> {
        self.get_state_change_set(tx_order)
    }

    fn multi_get_state_change_set(
        &self,
        tx_orders: Vec<u64>,
    ) -> Result<Vec<Option<StateChangeSetExt>>> {
        self.multi_get_state_change_set(tx_orders)
    }

    fn remove_state_change_set(&self, tx_order: u64) -> Result<()> {
        self.remove_state_change_set(tx_order)
    }

    fn check_state_change_set(&self, tx_orders: Vec<u64>) -> Result<Vec<u64>> {
        self.check_state_change_set(tx_orders)
    }

    fn get_changesets_range(
        &self,
        from_order: u64,
        to_order: u64,
    ) -> Result<Vec<(u64, StateChangeSetExt)>> {
        self.get_changesets_range(from_order, to_order)
    }

    fn get_changesets_range_with_limit(
        &self,
        from_order: u64,
        to_order: u64,
        limit: usize,
    ) -> Result<Vec<(u64, StateChangeSetExt)>> {
        self.get_changesets_range_with_limit(from_order, to_order, limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use moveos_config::store_config::RocksdbConfig;
    use prometheus::Registry;
    use raw_store::metrics::DBMetrics;
    use raw_store::rocks::RocksDB;
    use raw_store::StoreInstance;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn temp_state_store() -> StateDBStore {
        let dir = TempDir::new().expect("create tempdir");
        let registry = Registry::new();
        let db = RocksDB::new(
            dir.path(),
            crate::StoreMeta::get_column_family_names().to_vec(),
            RocksdbConfig::default(),
        )
        .expect("create rocksdb");
        let db_metrics = DBMetrics::new(&registry);
        let instance = StoreInstance::new_db_instance(db, Arc::new(db_metrics));
        StateDBStore::new(instance)
    }

    #[test]
    fn get_changesets_range_returns_full_span() {
        let store = temp_state_store();

        // Insert contiguous changesets keyed by numeric order
        store.insert_dummy_changesets(110, 30);

        // Fetch the span and ensure none are skipped
        let got = store
            .get_changesets_range_with_limit(110, 140, usize::MAX)
            .expect("load range");

        assert_eq!(got.len(), 30, "should load all changesets in range");
        assert_eq!(got.first().unwrap().0, 110);
        assert_eq!(got.last().unwrap().0, 139);
    }
}
