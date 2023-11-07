// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use async_trait::async_trait;
use itertools::Itertools;

use moveos_types::h256::H256;
use rooch_types::transaction::TransactionWithInfo;
use tracing::info;

use crate::errors::IndexerError;
use crate::IndexerStore;

use crate::types::{IndexedEvent, IndexedTransaction};

use super::IndexerStoreTrait;

#[macro_export]
macro_rules! chunk {
    ($data: expr, $size: expr) => {{
        $data
            .into_iter()
            .chunks($size)
            .into_iter()
            .map(|c| c.collect())
            .collect::<Vec<Vec<_>>>()
    }};
}

// #[derive(Clone)]
// pub struct IndexerStore {
//     pub sqlite_store: SqliteIndexerStore,
// }
//
// impl IndexerStore {
//     pub fn new(cp_pool: SqliteConnectionPool) -> Result<Self> {
//         let store = Self {
//             sqlite_store: SqliteIndexerStore::new(cp_pool),
//         };
//         Ok(store)
//     }
//
//     // //TODO implement a mock indexer store
//     // pub fn mock_indexer_store() -> Result<Self> {
//     // }
// }
//
// impl Display for IndexerStore {
//     fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
//         write!(f, "{}", self.clone())
//     }
// }
// impl Debug for IndexerStore {
//     fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
//         write!(f, "{}", self)
//     }
// }

#[async_trait]
impl IndexerStoreTrait for IndexerStore {
    async fn persist_transactions(
        &self,
        transactions: Vec<IndexedTransaction>,
    ) -> Result<(), IndexerError> {
        let len = transactions.len();

        let chunks = chunk!(transactions, self.sqlite_store.parallel_chunk_size);
        let futures = chunks
            .into_iter()
            .map(|c| {
                self.sqlite_store
                    .spawn_blocking_task(move |this| this.persist_transactions_chunk(c))
            })
            .collect::<Vec<_>>();

        futures::future::join_all(futures)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                IndexerError::SQLiteWriteError(format!(
                    "Failed to persist all transactions chunks: {:?}",
                    e
                ))
            })?;
        info!("Persisted {} transactions", len);
        Ok(())
    }

    async fn persist_events(&self, events: Vec<IndexedEvent>) -> Result<(), IndexerError> {
        if events.is_empty() {
            return Ok(());
        }
        let len = events.len();
        let chunks = chunk!(events, self.sqlite_store.parallel_chunk_size);
        let futures = chunks
            .into_iter()
            .map(|c| {
                self.sqlite_store
                    .spawn_blocking_task(move |this| this.persist_events_chunk(c))
            })
            .collect::<Vec<_>>();

        futures::future::join_all(futures)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                IndexerError::SQLiteWriteError(format!(
                    "Failed to persist all events chunks: {:?}",
                    e
                ))
            })?;
        info!("Persisted {} events", len);
        Ok(())
    }

    async fn query_transactions_by_hash(
        &self,
        _tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<TransactionWithInfo>>, IndexerError> {
        Ok(vec![])
    }
}
