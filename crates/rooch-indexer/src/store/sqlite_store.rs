// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use diesel::RunQueryDsl;
use tracing::info;

use crate::errors::{Context, IndexerError};

use crate::models::events::StoredEvent;
use crate::models::transactions::StoredTransaction;
use crate::schema::transactions;
use crate::types::{IndexedEvent, IndexedTransaction};
use crate::{get_sqlite_pool_connection, SqliteConnectionPool};

#[derive(Clone)]
#[allow(unused)]
pub struct SqliteIndexerStore {
    pub(crate) connection_pool: SqliteConnectionPool,
}

impl SqliteIndexerStore {
    pub fn new(connection_pool: SqliteConnectionPool) -> Self {
        Self { connection_pool }
    }

    pub fn persist_transactions(
        &self,
        transactions: Vec<IndexedTransaction>,
    ) -> Result<(), IndexerError> {
        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;

        let transactions = transactions
            .into_iter()
            .map(StoredTransaction::from)
            .collect::<Vec<_>>();

        diesel::insert_into(transactions::table)
            // .default_values()
            .values(transactions.as_slice())
            // .on_conflict_do_nothing()
            .execute(&mut connection)
            .map_err(IndexerError::from)
            .context("Failed to write transactions to SQLiteDB")?;

        Ok(())
    }

    pub fn persist_events(&self, events: Vec<IndexedEvent>) -> Result<(), IndexerError> {
        if events.is_empty() {
            return Ok(());
        }

        let events = events
            .into_iter()
            .map(StoredEvent::from)
            .collect::<Vec<_>>();

        info!("Persisted events: {:?}", events);
        Ok(())
    }
}
