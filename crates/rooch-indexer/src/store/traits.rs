// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::errors::IndexerError;
use crate::types::{IndexedEvent, IndexedTransaction};
use moveos_types::h256::H256;
use rooch_types::transaction::TransactionWithInfo;

pub trait IndexerStoreTrait: Send + Sync {
    fn persist_transactions(
        &self,
        transactions: Vec<IndexedTransaction>,
    ) -> Result<(), IndexerError>;

    fn persist_events(&self, events: Vec<IndexedEvent>) -> Result<(), IndexerError>;

    fn query_transactions_by_hash(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<TransactionWithInfo>>, IndexerError>;
}
