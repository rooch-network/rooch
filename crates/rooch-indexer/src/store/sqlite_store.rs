// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use diesel::RunQueryDsl;

use crate::errors::{Context, IndexerError};

use crate::models::events::StoredEvent;
use crate::models::states::{StoredGlobalState, StoredLeafState};
use crate::models::transactions::StoredTransaction;
use crate::schema::{events, global_states, leaf_states, transactions};
use crate::types::{IndexedEvent, IndexedGlobalState, IndexedLeafState, IndexedTransaction};
use crate::{get_sqlite_pool_connection, SqliteConnectionPool};

#[derive(Clone)]
pub struct SqliteIndexerStore {
    pub(crate) connection_pool: SqliteConnectionPool,
}

impl SqliteIndexerStore {
    pub fn new(connection_pool: SqliteConnectionPool) -> Self {
        Self { connection_pool }
    }

    pub fn persist_global_states(
        &self,
        states: Vec<IndexedGlobalState>,
    ) -> Result<(), IndexerError> {
        if states.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        let states = states
            .into_iter()
            .map(StoredGlobalState::from)
            .collect::<Vec<_>>();

        diesel::insert_into(global_states::table)
            .values(states.as_slice())
            .execute(&mut connection)
            .map_err(IndexerError::from)
            .context("Failed to write global states to SQLiteDB")?;

        Ok(())
    }

    pub fn update_global_states(
        &self,
        states: Vec<IndexedGlobalState>,
    ) -> Result<(), IndexerError> {
        if states.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        let states = states
            .into_iter()
            .map(StoredGlobalState::from)
            .collect::<Vec<_>>();

        // diesel::update(global_states::table)
        //     .values(states.as_slice())
        //     .execute(&mut connection)
        //     .map_err(IndexerError::from)
        //     .context("Failed to update global states to SQLiteDB")?;

        Ok(())
    }

    pub fn delete_global_states(
        &self,
        states: Vec<IndexedGlobalState>,
    ) -> Result<(), IndexerError> {
        if states.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        let states = states
            .into_iter()
            .map(StoredGlobalState::from)
            .collect::<Vec<_>>();

        // diesel::delete(global_states::table)
        //     .values(states.as_slice())
        //     .execute(&mut connection)
        //     .map_err(IndexerError::from)
        //     .context("Failed to delete global states to SQLiteDB")?;

        Ok(())
    }

    pub fn persist_leaf_states(&self, states: Vec<IndexedLeafState>) -> Result<(), IndexerError> {
        if states.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        let states = states
            .into_iter()
            .map(StoredLeafState::from)
            .collect::<Vec<_>>();

        diesel::insert_into(leaf_states::table)
            .values(states.as_slice())
            .execute(&mut connection)
            .map_err(IndexerError::from)
            .context("Failed to write leaf states to SQLiteDB")?;

        Ok(())
    }

    pub fn update_leaf_states(&self, states: Vec<IndexedLeafState>) -> Result<(), IndexerError> {
        if states.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        let states = states
            .into_iter()
            .map(StoredLeafState::from)
            .collect::<Vec<_>>();

        // diesel::update(leaf_states::table)
        //     .values(states.as_slice())
        //     .execute(&mut connection)
        //     .map_err(IndexerError::from)
        //     .context("Failed to update leaf states to SQLiteDB")?;

        Ok(())
    }

    pub fn delete_leaf_states(&self, states: Vec<IndexedLeafState>) -> Result<(), IndexerError> {
        if states.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        let states = states
            .into_iter()
            .map(StoredLeafState::from)
            .collect::<Vec<_>>();

        // diesel::delete(leaf_states::table)
        //     .values(states.as_slice())
        //     .execute(&mut connection)
        //     .map_err(IndexerError::from)
        //     .context("Failed to delete leaf states to SQLiteDB")?;

        Ok(())
    }

    pub fn persist_transactions(
        &self,
        transactions: Vec<IndexedTransaction>,
    ) -> Result<(), IndexerError> {
        if transactions.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        let transactions = transactions
            .into_iter()
            .map(StoredTransaction::from)
            .collect::<Vec<_>>();

        diesel::insert_into(transactions::table)
            .values(transactions.as_slice())
            .execute(&mut connection)
            .map_err(IndexerError::from)
            .context("Failed to write transactions to SQLiteDB")?;

        Ok(())
    }

    pub fn persist_events(&self, events: Vec<IndexedEvent>) -> Result<(), IndexerError> {
        if events.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        let events = events
            .into_iter()
            .map(StoredEvent::from)
            .collect::<Vec<_>>();

        diesel::insert_into(events::table)
            .values(events.as_slice())
            .execute(&mut connection)
            .map_err(IndexerError::from)
            .context("Failed to write events to SQLiteDB")?;

        Ok(())
    }
}
