// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::errors::IndexerError;
use rooch_types::indexer::event::IndexerEvent;
use rooch_types::indexer::state::{IndexerFieldState, IndexerObjectState};
use rooch_types::indexer::transaction::IndexerTransaction;

pub trait IndexerStoreTrait: Send + Sync {
    fn persist_or_update_object_states(
        &self,
        states: Vec<IndexerObjectState>,
    ) -> Result<(), IndexerError>;

    fn delete_object_states(&self, state_pks: Vec<String>) -> Result<(), IndexerError>;

    fn persist_or_update_field_states(
        &self,
        states: Vec<IndexerFieldState>,
    ) -> Result<(), IndexerError>;

    fn delete_field_states(&self, state_pks: Vec<(String, String)>) -> Result<(), IndexerError>;

    fn delete_field_states_by_object_id(&self, object_ids: Vec<String>)
        -> Result<(), IndexerError>;

    fn persist_transactions(
        &self,
        transactions: Vec<IndexerTransaction>,
    ) -> Result<(), IndexerError>;

    fn persist_events(&self, events: Vec<IndexerEvent>) -> Result<(), IndexerError>;
}
