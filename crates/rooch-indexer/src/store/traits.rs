// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::errors::IndexerError;
use crate::types::{
    IndexedEvent, IndexedFieldState, IndexedObjectState, IndexedTableChangeSet, IndexedTransaction,
};

pub trait IndexerStoreTrait: Send + Sync {
    fn persist_or_update_object_states(
        &self,
        states: Vec<IndexedObjectState>,
    ) -> Result<(), IndexerError>;

    fn delete_object_states(&self, state_pks: Vec<String>) -> Result<(), IndexerError>;

    fn persist_or_update_field_states(
        &self,
        states: Vec<IndexedFieldState>,
    ) -> Result<(), IndexerError>;

    fn delete_field_states(&self, state_pks: Vec<(String, String)>) -> Result<(), IndexerError>;

    fn delete_field_states_by_object_id(&self, object_ids: Vec<String>)
        -> Result<(), IndexerError>;

    fn persist_table_change_sets(
        &self,
        table_change_sets: Vec<IndexedTableChangeSet>,
    ) -> Result<(), IndexerError>;

    fn persist_transactions(
        &self,
        transactions: Vec<IndexedTransaction>,
    ) -> Result<(), IndexerError>;

    fn persist_events(&self, events: Vec<IndexedEvent>) -> Result<(), IndexerError>;
}
