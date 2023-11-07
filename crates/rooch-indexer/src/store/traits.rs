// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::errors::IndexerError;
use crate::types::{IndexedEvent, IndexedTransaction};
use async_trait::async_trait;
use moveos_types::h256::H256;
use rooch_types::transaction::TransactionWithInfo;

#[async_trait]
pub trait IndexerStoreTrait: Send + Sync {
    // async fn persist_transaction(
    //     &self,
    //     transaction: IndexedTransaction,
    // ) -> Result<(), IndexerError>;
    async fn persist_transactions(
        &self,
        transactions: Vec<IndexedTransaction>,
    ) -> Result<(), IndexerError>;

    async fn persist_events(&self, events: Vec<IndexedEvent>) -> Result<(), IndexerError>;

    async fn query_transactions_by_hash(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<TransactionWithInfo>>, IndexerError>;
}
