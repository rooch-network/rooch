// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::errors::IndexerError;
use crate::types::{IndexedEvent, IndexedGlobalState, IndexedLeafState, IndexedTransaction};

pub trait IndexerStoreTrait: Send + Sync {
    fn persist_global_states(&self, states: Vec<IndexedGlobalState>) -> Result<(), IndexerError>;

    fn update_global_states(&self, states: Vec<IndexedGlobalState>) -> Result<(), IndexerError>;

    fn remove_global_states(&self, states: Vec<IndexedGlobalState>) -> Result<(), IndexerError>;

    fn persist_leaf_states(&self, states: Vec<IndexedLeafState>) -> Result<(), IndexerError>;

    fn update_leaf_states(&self, states: Vec<IndexedLeafState>) -> Result<(), IndexerError>;

    fn remove_leaf_states(&self, states: Vec<IndexedLeafState>) -> Result<(), IndexerError>;

    fn persist_transactions(
        &self,
        transactions: Vec<IndexedTransaction>,
    ) -> Result<(), IndexerError>;

    fn persist_events(&self, events: Vec<IndexedEvent>) -> Result<(), IndexerError>;
}
