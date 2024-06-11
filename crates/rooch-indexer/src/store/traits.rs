// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::errors::IndexerError;
use rooch_types::indexer::event::IndexerEvent;
use rooch_types::indexer::state::{IndexerObjectState, IndexerObjectStateChanges};
use rooch_types::indexer::transaction::IndexerTransaction;

pub trait IndexerStoreTrait: Send + Sync {
    fn update_object_states(
        &self,
        object_state_change: IndexerObjectStateChanges,
    ) -> Result<(), IndexerError>;
    fn persist_or_update_object_states(
        &self,
        states: Vec<IndexerObjectState>,
    ) -> Result<(), IndexerError>;

    fn delete_object_states(&self, state_pks: Vec<String>) -> Result<(), IndexerError>;

    fn persist_transactions(
        &self,
        transactions: Vec<IndexerTransaction>,
    ) -> Result<(), IndexerError>;

    fn persist_events(&self, events: Vec<IndexerEvent>) -> Result<(), IndexerError>;
}
