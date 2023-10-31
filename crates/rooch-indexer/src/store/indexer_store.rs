// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use core::result::Result::Ok;
use itertools::Itertools;
use std::time::Duration;
use tap::Tap;

use diesel::RunQueryDsl;
use tracing::info;

use crate::errors::{Context, IndexerError};

use crate::models::events::StoredEvent;
use crate::models::transactions::StoredTransaction;
use crate::store::diesel_macro::transactional_blocking_with_retry;
use crate::types::{IndexedEvent, IndexedTransaction};
use crate::SqliteConnectionPool;

use super::IndexerStore;

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

// In one DB transaction, the update could be chunked into
// a few statements, this is the amount of rows to update in one statement
// TODO: I think with the `per_db_tx` params, `SQLITE_COMMIT_CHUNK_SIZE_INTRA_DB_TX`
// is now less relevant. We should do experiments and remove it if it's true.
const SQLITE_COMMIT_CHUNK_SIZE_INTRA_DB_TX: usize = 1000;
// The amount of rows to update in one DB transcation
const SQLITE_COMMIT_PARALLEL_CHUNK_SIZE_PER_DB_TX: usize = 500;

#[derive(Clone)]
pub struct SqliteIndexerStore {
    blocking_cp: SqliteConnectionPool,
    parallel_chunk_size: usize,
}

impl SqliteIndexerStore {
    pub fn new(blocking_cp: SqliteConnectionPool) -> Self {
        let parallel_chunk_size = std::env::var("SQLITE_COMMIT_PARALLEL_CHUNK_SIZE")
            .unwrap_or_else(|_e| SQLITE_COMMIT_PARALLEL_CHUNK_SIZE_PER_DB_TX.to_string())
            .parse::<usize>()
            .unwrap();
        Self {
            blocking_cp,
            parallel_chunk_size,
        }
    }

    fn persist_transactions_chunk(
        &self,
        transactions: Vec<IndexedTransaction>,
    ) -> Result<(), IndexerError> {
        let transactions = transactions
            .iter()
            .map(StoredTransaction::from)
            .collect::<Vec<_>>();

        transactional_blocking_with_retry!(
            &self.blocking_cp,
            |conn| {
                for transaction_chunk in transactions.chunks(SQLITE_COMMIT_CHUNK_SIZE_INTRA_DB_TX) {
                    diesel::insert_into(transactions::table)
                        .values(transaction_chunk)
                        .on_conflict_do_nothing()
                        .execute(conn)
                        .map_err(IndexerError::from)
                        .context("Failed to write transactions to SQLiteDB")?;
                }
                Ok::<(), IndexerError>(())
            },
            Duration::from_secs(60)
        )
        .tap(|_| {
            info!(
                elapsed,
                "Persisted {} chunked transactions",
                transactions.len()
            )
        })
    }

    fn persist_events_chunk(&self, events: Vec<IndexedEvent>) -> Result<(), IndexerError> {
        let len = events.len();
        let events = events
            .into_iter()
            .map(StoredEvent::from)
            .collect::<Vec<_>>();

        transactional_blocking_with_retry!(
            &self.blocking_cp,
            |conn| {
                for event_chunk in events.chunks(SQLITE_COMMIT_CHUNK_SIZE_INTRA_DB_TX) {
                    diesel::insert_into(events::table)
                        .values(event_chunk)
                        .on_conflict_do_nothing()
                        .execute(conn)
                        .map_err(IndexerError::from)
                        .context("Failed to write events to SQLiteDB")?;
                }
                Ok::<(), IndexerError>(())
            },
            Duration::from_secs(60)
        )
        .tap(|_| info!(elapsed, "Persisted {} chunked events", len))
    }

    async fn execute_in_blocking_worker<F, R>(&self, f: F) -> Result<R, IndexerError>
    where
        F: FnOnce(Self) -> Result<R, IndexerError> + Send + 'static,
        R: Send + 'static,
    {
        let this = self.clone();
        let current_span = tracing::Span::current();
        tokio::task::spawn_blocking(move || {
            let _guard = current_span.enter();
            f(this)
        })
        .await
        .map_err(Into::into)
        .and_then(std::convert::identity)
    }

    fn spawn_blocking_task<F, R>(
        &self,
        f: F,
    ) -> tokio::task::JoinHandle<std::result::Result<R, IndexerError>>
    where
        F: FnOnce(Self) -> Result<R, IndexerError> + Send + 'static,
        R: Send + 'static,
    {
        let this = self.clone();
        let current_span = tracing::Span::current();
        tokio::task::spawn_blocking(move || {
            let _guard = current_span.enter();
            f(this)
        })
    }

    fn spawn_task<F, Fut, R>(&self, f: F) -> tokio::task::JoinHandle<Result<R, IndexerError>>
    where
        F: FnOnce(Self) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<R, IndexerError>> + Send + 'static,
        R: Send + 'static,
    {
        let this = self.clone();
        tokio::task::spawn(async move { f(this).await })
    }
}

#[async_trait]
impl IndexerStore for SqliteIndexerStore {
    async fn persist_transactions(
        &self,
        transactions: Vec<IndexedTransaction>,
    ) -> Result<(), IndexerError> {
        let len = transactions.len();

        let chunks = chunk!(transactions, self.parallel_chunk_size);
        let futures = chunks
            .into_iter()
            .map(|c| self.spawn_blocking_task(move |this| this.persist_transactions_chunk(c)))
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
        info!(elapsed, "Persisted {} transactions", len);
        Ok(())
    }

    async fn persist_events(&self, events: Vec<IndexedEvent>) -> Result<(), IndexerError> {
        if events.is_empty() {
            return Ok(());
        }
        let len = events.len();
        let chunks = chunk!(events, self.parallel_chunk_size);
        let futures = chunks
            .into_iter()
            .map(|c| self.spawn_blocking_task(move |this| this.persist_events_chunk(c)))
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
        info!(elapsed, "Persisted {} events", len);
        Ok(())
    }
}
