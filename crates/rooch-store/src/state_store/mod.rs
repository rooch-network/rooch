// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::STATE_CHANGE_SET_COLUMN_FAMILY_NAME;
use anyhow::Result;
use moveos_types::state::StateChangeSetExt;
use raw_store::CodecKVStore;
use raw_store::{derive_store, SchemaStore, StoreInstance};

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

        let mut results = Vec::new();

        // Try to get an iterator from the underlying RocksDB store
        if let Some(rocks_store) = self.state_change_set_store.get_store().store().db() {
            let cf_name = STATE_CHANGE_SET_COLUMN_FAMILY_NAME;

            // Create iterator and seek to the start key
            let mut iter = rocks_store.iter::<u64, StateChangeSetExt>(cf_name)?;

            // Seek to the start order
            iter.seek(from_order.to_le_bytes().to_vec())?;

            // Iterate until we reach the end order or limit
            for item_result in iter {
                let (tx_order, changeset) = item_result?;

                // Stop if we've reached or passed the to_order
                if tx_order >= to_order {
                    break;
                }

                // Only include items that are >= from_order (in case seek landed on a smaller key)
                if tx_order >= from_order {
                    results.push((tx_order, changeset));

                    // Stop if we've reached the limit
                    if results.len() >= limit {
                        break;
                    }
                }
            }

            Ok(results)
        } else {
            // Fallback to batch queries for non-RocksDB stores (e.g., in-memory tests)
            self.get_changesets_range_batch_fallback(from_order, to_order, limit)
        }
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
