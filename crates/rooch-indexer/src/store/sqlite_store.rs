// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::errors::{Context, IndexerError};
use anyhow::Result;
use diesel::QueryDsl;
use diesel::{ExpressionMethods, RunQueryDsl};
use function_name::named;
use rooch_types::indexer::event::IndexerEvent;
use rooch_types::indexer::field::IndexerField;
use rooch_types::indexer::state::IndexerObjectState;
use rooch_types::indexer::transaction::IndexerTransaction;
use std::sync::Arc;

use crate::models::events::StoredEvent;
use crate::models::fields::StoredField;
use crate::models::inscriptions::StoredInscription;
use crate::models::states::StoredObjectState;
use crate::models::transactions::{escape_transaction, StoredTransaction};
use crate::models::utxos::StoredUTXO;
use crate::schema::{events, fields, inscriptions, object_states, transactions, utxos};
use crate::store::metrics::IndexerDBMetrics;
use crate::utils::escape_sql_string;
use crate::{get_sqlite_pool_connection, SqliteConnectionPool};

#[derive(Clone)]
pub struct SqliteIndexerStore {
    pub(crate) connection_pool: SqliteConnectionPool,
    db_metrics: Arc<IndexerDBMetrics>,
}

impl SqliteIndexerStore {
    pub fn new(connection_pool: SqliteConnectionPool, db_metrics: Arc<IndexerDBMetrics>) -> Self {
        Self {
            connection_pool,
            db_metrics,
        }
    }

    #[named]
    pub fn persist_or_update_object_states(
        &self,
        states: Vec<IndexerObjectState>,
    ) -> Result<(), IndexerError> {
        if states.is_empty() {
            return Ok(());
        }

        let fn_name = function_name!();
        let _timer = self
            .db_metrics
            .indexer_store_metrics
            .indexer_persist_or_update_or_delete_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        let states = states
            .into_iter()
            .map(StoredObjectState::from)
            .collect::<Vec<_>>();

        // Diesel for SQLite don't support batch update yet, so implements batch update directly via raw SQL
        let values_clause = states
            .into_iter()
            .map(|state| {
                format!(
                    "('{}', '{}', '{}', {}, {}, {}, {})",
                    escape_sql_string(state.id),
                    escape_sql_string(state.owner),
                    escape_sql_string(state.object_type),
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
                INSERT INTO object_states (id, owner, object_type, tx_order, state_index, created_at, updated_at) \
                VALUES {} \
                ON CONFLICT (id) DO UPDATE SET \
                owner = excluded.owner, \
                tx_order = excluded.tx_order, \
                state_index = excluded.state_index, \
                updated_at = excluded.updated_at
            ",
            values_clause
        );

        // // Perform multi-insert with ON CONFLICT update
        // diesel::insert_into(object_states::table)
        //     .values(states.as_slice())
        //     .on_conflict(object_states::object_id)
        //     .do_update()
        //     .set((
        //         object_states::owner.eq(excluded(object_states::owner)),
        //         object_states::flag.eq(excluded(object_states::flag)),
        //         object_states::value.eq(excluded(object_states::value)),
        //         object_states::size.eq(excluded(object_states::size)),
        //         object_states::updated_at.eq(excluded(object_states::updated_at)),
        //     ))
        //     .execute(&mut connection)
        //     .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
        //     .context("Failed to write or update object states to SQLiteDB");

        // Execute the raw SQL query
        diesel::sql_query(query.clone())
            .execute(&mut connection)
            .map_err(|e| {
                tracing::error!("Upsert object states Executing Query error: {}", query);
                IndexerError::SQLiteWriteError(e.to_string())
            })
            .context("Failed to write or update object states to SQLiteDB")?;

        Ok(())
    }

    #[named]
    pub fn persist_or_update_object_state_utxos(
        &self,
        states: Vec<IndexerObjectState>,
    ) -> Result<(), IndexerError> {
        if states.is_empty() {
            return Ok(());
        }

        let fn_name = function_name!();
        let _timer = self
            .db_metrics
            .indexer_store_metrics
            .indexer_persist_or_update_or_delete_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        let states = states.into_iter().map(StoredUTXO::from).collect::<Vec<_>>();

        // Diesel for SQLite don't support batch update yet, so implements batch update directly via raw SQL
        let values_clause = states
            .into_iter()
            .map(|state| {
                format!(
                    "('{}', '{}', {}, {}, {}, {})",
                    escape_sql_string(state.id),
                    escape_sql_string(state.owner),
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
                INSERT INTO utxos (id, owner, tx_order, state_index, created_at, updated_at) \
                VALUES {} \
                ON CONFLICT (id) DO UPDATE SET \
                owner = excluded.owner, \
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
                tracing::error!("Upsert object state utxos Executing Query error: {}", query);
                IndexerError::SQLiteWriteError(e.to_string())
            })
            .context("Failed to write or update object state utxos to SQLiteDB")?;

        Ok(())
    }

    #[named]
    pub fn persist_or_update_object_state_inscriptions(
        &self,
        states: Vec<IndexerObjectState>,
    ) -> Result<(), IndexerError> {
        if states.is_empty() {
            return Ok(());
        }

        let fn_name = function_name!();
        let _timer = self
            .db_metrics
            .indexer_store_metrics
            .indexer_persist_or_update_or_delete_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        let states = states
            .into_iter()
            .map(StoredInscription::from)
            .collect::<Vec<_>>();

        // Diesel for SQLite don't support batch update yet, so implements batch update directly via raw SQL
        let values_clause = states
            .into_iter()
            .map(|state| {
                format!(
                    "('{}', '{}', {}, {}, {}, {})",
                    escape_sql_string(state.id),
                    escape_sql_string(state.owner),
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
                INSERT INTO inscriptions (id, owner, tx_order, state_index, created_at, updated_at) \
                VALUES {} \
                ON CONFLICT (id) DO UPDATE SET \
                owner = excluded.owner, \
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
                tracing::error!(
                    "Upsert object state inscriptions Executing Query error: {}",
                    query
                );
                IndexerError::SQLiteWriteError(e.to_string())
            })
            .context("Failed to write or update object state inscriptions to SQLiteDB")?;

        Ok(())
    }

    #[named]
    pub fn delete_object_states(&self, state_pks: Vec<String>) -> Result<(), IndexerError> {
        if state_pks.is_empty() {
            return Ok(());
        }

        let fn_name = function_name!();
        let _timer = self
            .db_metrics
            .indexer_store_metrics
            .indexer_persist_or_update_or_delete_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;

        diesel::delete(object_states::table.filter(object_states::id.eq_any(state_pks.as_slice())))
            .execute(&mut connection)
            .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
            .context("Failed to delete object states to SQLiteDB")?;

        Ok(())
    }

    #[named]
    pub fn delete_object_state_utxos(&self, state_pks: Vec<String>) -> Result<(), IndexerError> {
        if state_pks.is_empty() {
            return Ok(());
        }

        let fn_name = function_name!();
        let _timer = self
            .db_metrics
            .indexer_store_metrics
            .indexer_persist_or_update_or_delete_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;

        diesel::delete(utxos::table.filter(utxos::id.eq_any(state_pks.as_slice())))
            .execute(&mut connection)
            .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
            .context("Failed to delete object state utxos to SQLiteDB")?;

        Ok(())
    }

    #[named]
    pub fn delete_object_state_inscriptions(
        &self,
        state_pks: Vec<String>,
    ) -> Result<(), IndexerError> {
        if state_pks.is_empty() {
            return Ok(());
        }

        let fn_name = function_name!();
        let _timer = self
            .db_metrics
            .indexer_store_metrics
            .indexer_persist_or_update_or_delete_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;

        diesel::delete(inscriptions::table.filter(inscriptions::id.eq_any(state_pks.as_slice())))
            .execute(&mut connection)
            .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
            .context("Failed to delete object state inscriptions to SQLiteDB")?;

        Ok(())
    }

    #[named]
    pub fn persist_transactions(
        &self,
        transactions: Vec<IndexerTransaction>,
    ) -> Result<(), IndexerError> {
        if transactions.is_empty() {
            return Ok(());
        }

        let fn_name = function_name!();
        let _timer = self
            .db_metrics
            .indexer_store_metrics
            .indexer_persist_or_update_or_delete_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        let transactions = transactions
            .into_iter()
            .map(|v| escape_transaction(StoredTransaction::from(v)))
            .collect::<Vec<_>>();

        diesel::insert_into(transactions::table)
            .values(transactions.as_slice())
            .execute(&mut connection)
            .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
            .context("Failed to write transactions to SQLiteDB")?;

        Ok(())
    }

    #[named]
    pub fn delete_transactions(&self, tx_orders: Vec<u64>) -> Result<(), IndexerError> {
        if tx_orders.is_empty() {
            return Ok(());
        }

        let fn_name = function_name!();
        let _timer = self
            .db_metrics
            .indexer_store_metrics
            .indexer_persist_or_update_or_delete_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;

        let tx_orders: Vec<_> = tx_orders.into_iter().map(|v| v as i64).collect();
        diesel::delete(
            transactions::table.filter(transactions::tx_order.eq_any(tx_orders.as_slice())),
        )
        .execute(&mut connection)
        .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
        .context("Failed to delete transactions to SQLiteDB")?;

        Ok(())
    }

    #[named]
    pub fn persist_events(&self, events: Vec<IndexerEvent>) -> Result<(), IndexerError> {
        if events.is_empty() {
            return Ok(());
        }

        let fn_name = function_name!();
        let _timer = self
            .db_metrics
            .indexer_store_metrics
            .indexer_persist_or_update_or_delete_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
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

    #[named]
    pub fn delete_events(&self, tx_orders: Vec<u64>) -> Result<(), IndexerError> {
        if tx_orders.is_empty() {
            return Ok(());
        }

        let fn_name = function_name!();
        let _timer = self
            .db_metrics
            .indexer_store_metrics
            .indexer_persist_or_update_or_delete_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;

        let tx_orders: Vec<_> = tx_orders.into_iter().map(|v| v as i64).collect();
        diesel::delete(events::table.filter(events::tx_order.eq_any(tx_orders.as_slice())))
            .execute(&mut connection)
            .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
            .context("Failed to delete events to SQLiteDB")?;

        Ok(())
    }

    #[named]
    pub fn persist_or_update_fields(&self, fields: Vec<IndexerField>) -> Result<(), IndexerError> {
        if fields.is_empty() {
            return Ok(());
        }

        let fn_name = function_name!();
        let _timer = self
            .db_metrics
            .indexer_store_metrics
            .indexer_persist_or_update_or_delete_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        let fields = fields
            .into_iter()
            .map(StoredField::from)
            .collect::<Vec<_>>();

        // Diesel for SQLite don't support batch update yet, so implements batch update directly via raw SQL
        let values_clause = fields
            .into_iter()
            .map(|field| {
                format!(
                    "('{}', '{}', '{}', {}, {}, {})",
                    escape_sql_string(field.id),
                    escape_sql_string(field.parent_id),
                    escape_sql_string(field.field_key),
                    field.sort_key,
                    field.created_at,
                    field.updated_at,
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let query = format!(
            "
                INSERT INTO fields (id, parent_id, field_key, sort_key, created_at, updated_at) \
                VALUES {} \
                ON CONFLICT (id) DO UPDATE SET \
                parent_id = excluded.parent_id, \
                field_key = excluded.field_key, \
                sort_key = excluded.sort_key, \
                updated_at = excluded.updated_at
            ",
            values_clause
        );

        // Execute the raw SQL query
        diesel::sql_query(query.clone())
            .execute(&mut connection)
            .map_err(|e| {
                tracing::error!("Upsert fields Executing Query error: {}", query);
                IndexerError::SQLiteWriteError(e.to_string())
            })
            .context("Failed to write or update fields to SQLiteDB")?;

        Ok(())
    }

    pub fn delete_fields(&self, field_pks: Vec<String>) -> Result<(), IndexerError> {
        if field_pks.is_empty() {
            return Ok(());
        }

        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;
        // Diesel for SQLite don't support batch delete on composite primary key yet, so implements batch delete directly via raw SQL
        let values_clause = field_pks
            .into_iter()
            .map(|pk| format!("('{}')", escape_sql_string(pk),))
            .collect::<Vec<_>>()
            .join(",");

        let query = format!(
            "
                DELETE FROM fields \
                WHERE (id) IN ({})
            ",
            values_clause
        );

        // Execute the raw SQL query
        diesel::sql_query(query.clone())
            .execute(&mut connection)
            .map_err(|e| {
                tracing::error!("Delete fields Executing Query error: {}", query);
                IndexerError::SQLiteWriteError(e.to_string())
            })
            .context("Failed to delete fields to SQLiteDB")?;

        Ok(())
    }

    #[named]
    pub fn delete_fields_by_parent_id(&self, ids: Vec<String>) -> Result<(), IndexerError> {
        if ids.is_empty() {
            return Ok(());
        }

        let fn_name = function_name!();
        let _timer = self
            .db_metrics
            .indexer_store_metrics
            .indexer_persist_or_update_or_delete_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
        let mut connection = get_sqlite_pool_connection(&self.connection_pool)?;

        diesel::delete(fields::table.filter(fields::parent_id.eq_any(ids.as_slice())))
            .execute(&mut connection)
            .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
            .context("Failed to delete fields to SQLiteDB")?;

        Ok(())
    }
}
