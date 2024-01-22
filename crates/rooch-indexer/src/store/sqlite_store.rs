// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use diesel::QueryDsl;
use diesel::{ExpressionMethods, RunQueryDsl};
use tracing::log;

use crate::errors::{Context, IndexerError};
use crate::models::events::StoredEvent;
use crate::models::states::{StoredGlobalState, StoredTableChangeSet, StoredTableState};
use crate::models::transactions::StoredTransaction;
use crate::schema::{events, global_states, table_change_sets, table_states, transactions};
use crate::types::{
    IndexedEvent, IndexedGlobalState, IndexedTableChangeSet, IndexedTableState, IndexedTransaction,
};
use crate::utils::escape_sql_string;
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
                    "('{}', '{}', {}, '{}', '{}', '{}', {}, {}, {}, {}, {})",
                    escape_sql_string(state.object_id),
                    escape_sql_string(state.owner),
                    state.flag,
                    escape_sql_string(state.value),
                    escape_sql_string(state.object_type),
                    escape_sql_string(state.state_root),
                    state.size,
                    state.tx_order,
                    state.state_index,
                    state.created_at,
                    state.updated_at,
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let query = format!(
            "
                INSERT INTO global_states (object_id, owner, flag, value, object_type, state_root, size, tx_order, state_index, created_at, updated_at) \
                VALUES {} \
                ON CONFLICT (object_id) DO UPDATE SET \
                owner = excluded.owner, \
                flag = excluded.flag, \
                value = excluded.value, \
                state_root = excluded.state_root, \
                size = excluded.size, \
                tx_order = excluded.tx_order, \
                state_index = excluded.state_index, \
                updated_at = excluded.updated_at
            ",
            values_clause
        );

        // // Perform multi-insert with ON CONFLICT update
        // diesel::insert_into(global_states::table)
        //     .values(states.as_slice())
        //     .on_conflict(global_states::object_id)
        //     .do_update()
        //     .set((
        //         global_states::owner.eq(excluded(global_states::owner)),
        //         global_states::flag.eq(excluded(global_states::flag)),
        //         global_states::value.eq(excluded(global_states::value)),
        //         global_states::size.eq(excluded(global_states::size)),
        //         global_states::updated_at.eq(excluded(global_states::updated_at)),
        //     ))
        //     .execute(&mut connection)
        //     .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
        //     .context("Failed to write or update global states to SQLiteDB");

        // Execute the raw SQL query
        diesel::sql_query(query.clone())
            .execute(&mut connection)
            .map_err(|e| {
                log::error!("Upsert global states Executing Query error: {}", query);
                IndexerError::SQLiteWriteError(e.to_string())
            })
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

    pub fn persist_or_update_table_states(
        &self,
        states: Vec<IndexedTableState>,
    ) -> Result<(), IndexerError> {
        if states.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        let states = states
            .into_iter()
            .map(StoredTableState::from)
            .collect::<Vec<_>>();

        // Diesel for SQLite don't support batch update yet, so implements batch update directly via raw SQL
        let values_clause = states
            .into_iter()
            .map(|state| {
                format!(
                    "('{}', '{}', '{}', '{}', '{}', '{}', {}, {}, {}, {})",
                    escape_sql_string(state.table_handle),
                    escape_sql_string(state.key_hex),
                    escape_sql_string(state.key_str),
                    escape_sql_string(state.value),
                    escape_sql_string(state.key_type),
                    escape_sql_string(state.value_type),
                    state.tx_order,
                    state.state_index,
                    state.created_at,
                    state.updated_at,
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let query = format!(
            "
                INSERT INTO table_states (table_handle, key_hex, key_str, value, key_type, value_type, tx_order, state_index, created_at, updated_at) \
                VALUES {} \
                ON CONFLICT (table_handle, key_hex) DO UPDATE SET \
                value = excluded.value, \
                value_type = excluded.value_type, \
                tx_order = excluded.tx_order, \
                state_index = excluded.state_index, \
                updated_at = excluded.updated_at
            ",
            values_clause
        );

        // Execute the raw SQL query
        diesel::sql_query(query.clone())
            .execute(&mut connection)
            .map_err(|e| {
                log::error!("Upsert table states Executing Query error: {}", query);
                IndexerError::SQLiteWriteError(e.to_string())
            })
            .context("Failed to write or update table states to SQLiteDB")?;

        Ok(())
    }

    pub fn delete_table_states(
        &self,
        state_pks: Vec<(String, String)>,
    ) -> Result<(), IndexerError> {
        if state_pks.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        // Diesel for SQLite don't support batch delete on composite primary key yet, so implements batch delete directly via raw SQL
        let values_clause = state_pks
            .into_iter()
            .map(|pk| {
                format!(
                    "('{}', '{}')",
                    escape_sql_string(pk.0),
                    escape_sql_string(pk.1),
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        let query = format!(
            "
                DELETE FROM table_states \
                WHERE (table_handle, key_hex) IN ({})
            ",
            values_clause
        );

        // Execute the raw SQL query
        diesel::sql_query(query.clone())
            .execute(&mut connection)
            .map_err(|e| {
                log::error!("Delete table states Executing Query error: {}", query);
                IndexerError::SQLiteWriteError(e.to_string())
            })
            .context("Failed to delete table states to SQLiteDB")?;

        Ok(())
    }

    pub fn delete_table_states_by_table_handle(
        &self,
        table_handles: Vec<String>,
    ) -> Result<(), IndexerError> {
        if table_handles.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        diesel::delete(
            table_states::table.filter(table_states::table_handle.eq_any(table_handles.as_slice())),
        )
        .execute(&mut connection)
        .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
        .context("Failed to delete table states by table handles to SQLiteDB")?;

        Ok(())
    }

    pub fn persist_table_change_sets(
        &self,
        table_change_sets: Vec<IndexedTableChangeSet>,
    ) -> Result<(), IndexerError> {
        if table_change_sets.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        let table_change_sets = table_change_sets
            .into_iter()
            .map(StoredTableChangeSet::from)
            .collect::<Vec<_>>();

        diesel::insert_into(table_change_sets::table)
            .values(table_change_sets.as_slice())
            .execute(&mut connection)
            .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
            .context("Failed to write table change sets to SQLiteDB")?;

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
