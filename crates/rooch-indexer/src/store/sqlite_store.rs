// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use diesel::QueryDsl;
use diesel::{ExpressionMethods, RunQueryDsl};

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

    pub fn persist_or_update_global_states(
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

        // Diesel for SQLite don't support batch update yet, so implements batch update directly via raw SQL
        let values_clause = states
            .into_iter()
            .map(|state| {
                format!(
                    "(\'{}\', \'{}\', {}, \'{}\', \'{}\', {}, {}, {})",
                    state.object_id,
                    state.owner,
                    state.flag,
                    state.value,
                    state.key_type,
                    state.size,
                    state.created_at,
                    state.updated_at,
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let query = format!(
            "
                INSERT INTO global_states (object_id, owner, flag,value, key_type, size, created_at, updated_at) \
                VALUES {} \
                ON CONFLICT (object_id) DO UPDATE SET \
                owner = excluded.owner, \
                flag = excluded.flag, \
                value = excluded.value, \
                size = excluded.size, \
                updated_at = excluded.updated_at
            ",
            values_clause
        );

        println!("Upsert global states Executing Query: {}", query);
        // Execute the raw SQL query
        diesel::sql_query(query)
            .execute(&mut connection)
            .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
            .context("Failed to write or update global states to SQLiteDB")?;

        Ok(())
    }

    pub fn delete_global_states(&self, state_pks: Vec<String>) -> Result<(), IndexerError> {
        if state_pks.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;

        diesel::delete(
            global_states::table.filter(global_states::object_id.eq_any(state_pks.as_slice())),
        )
        .execute(&mut connection)
        .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
        .context("Failed to delete global states to SQLiteDB")?;

        Ok(())
    }

    pub fn persist_or_update_leaf_states(
        &self,
        states: Vec<IndexedLeafState>,
    ) -> Result<(), IndexerError> {
        if states.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        let states = states
            .into_iter()
            .map(StoredLeafState::from)
            .collect::<Vec<_>>();

        // Diesel for SQLite don't support batch update yet, so implements batch update directly via raw SQL
        let values_clause = states
            .into_iter()
            .map(|state| {
                format!(
                    "(\'{}\', \'{}\', \'{}\', \'{}\', \'{}\', {}, {})",
                    state.id,
                    state.object_id,
                    state.key_hash,
                    state.value,
                    state.value_type,
                    state.created_at,
                    state.updated_at,
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let query = format!(
            "
                INSERT INTO leaf_states (id, object_id, key_hash, value, value_type, created_at, updated_at) \
                VALUES {} \
                ON CONFLICT (id) DO UPDATE SET \
                value = excluded.value, \
                value_type = excluded.value_type, \
                updated_at = excluded.updated_at
            ",
            values_clause
        );

        println!("Upsert leaf states Executing Query: {}", query);
        // Execute the raw SQL query
        diesel::sql_query(query)
            .execute(&mut connection)
            .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
            .context("Failed to write or update leaf states to SQLiteDB")?;

        Ok(())
    }

    pub fn delete_leaf_states(&self, state_pks: Vec<String>) -> Result<(), IndexerError> {
        if state_pks.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        diesel::delete(leaf_states::table.filter(leaf_states::id.eq_any(state_pks.as_slice())))
            .execute(&mut connection)
            .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
            .context("Failed to delete leaf states to SQLiteDB")?;

        Ok(())
    }

    pub fn delete_leaf_states_by_table_handle(
        &self,
        table_handles: Vec<String>,
    ) -> Result<(), IndexerError> {
        if table_handles.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        diesel::delete(
            leaf_states::table.filter(leaf_states::object_id.eq_any(table_handles.as_slice())),
        )
        .execute(&mut connection)
        .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
        .context("Failed to delete leaf states by table handles to SQLiteDB")?;

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
            .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
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
            .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
            .context("Failed to write events to SQLiteDB")?;

        Ok(())
    }
}
