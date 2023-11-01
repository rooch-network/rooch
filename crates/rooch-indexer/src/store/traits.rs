// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::errors::IndexerError;
use crate::types::{IndexedEvent, IndexedTransaction};
use async_trait::async_trait;

#[async_trait]
pub trait IndexerStore {
    async fn persist_transactions(
        &self,
        transactions: Vec<IndexedTransaction>,
    ) -> Result<(), IndexerError>;

    async fn persist_events(&self, events: Vec<IndexedEvent>) -> Result<(), IndexerError>;
}
