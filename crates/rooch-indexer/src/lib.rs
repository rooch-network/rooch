// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#![recursion_limit = "256"]

use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use diesel::r2d2::ConnectionManager;
use diesel::sqlite::SqliteConnection;
use tracing::info;

use errors::IndexerError;
use rooch_config::indexer_config::IndexerConfig;
use store::IndexerStore;

pub mod errors;
pub mod indexer;
pub mod models;
pub mod schema;
pub mod store;
pub mod test_utils;
pub mod types;
pub mod utils;

pub type SqliteConnectionPool = diesel::r2d2::Pool<ConnectionManager<SqliteConnection>>;
pub type SqlitePoolConnection = diesel::r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;

/// Returns all endpoints for which we have implemented on the indexer,
/// some of them are not validated yet.
/// NOTE: we only use this for integration testing
const IMPLEMENTED_METHODS: [&str; 4] = [
    "multi_get_transactions",
    "multi_get_events",
    // indexer apis
    "query_transactions",
    "query_events",
];

pub struct Indexer;

impl Indexer {
    pub async fn start<S: IndexerStore + Sync + Send + Clone + 'static>(
        _config: &IndexerConfig,
        _store: S,
    ) -> Result<(), IndexerError> {
        info!("Rooch indexer started...",);

        Ok(())
    }
}

pub fn new_sqlite_connection_pool(db_url: &str) -> Result<SqliteConnectionPool, IndexerError> {
    new_sqlite_connection_pool_impl(db_url, None)
}

pub fn new_sqlite_connection_pool_impl(
    db_url: &str,
    pool_size: Option<u32>,
) -> Result<SqliteConnectionPool, IndexerError> {
    let pool_config = SqliteConnectionPoolConfig::default();
    let manager = ConnectionManager::<SqliteConnection>::new(db_url);

    let pool_size = pool_size.unwrap_or(pool_config.pool_size);
    diesel::r2d2::Pool::builder()
        .max_size(pool_size)
        .connection_timeout(pool_config.connection_timeout)
        .connection_customizer(Box::new(pool_config.connection_config()))
        .build(manager)
        .map_err(|e| {
            IndexerError::SqliteConnectionPoolInitError(format!(
                "Failed to initialize connection pool with error: {:?}",
                e
            ))
        })
}

#[derive(Debug, Clone, Copy)]
pub struct SqliteConnectionPoolConfig {
    pool_size: u32,
    connection_timeout: Duration,
    statement_timeout: Duration,
}

impl SqliteConnectionPoolConfig {
    const DEFAULT_POOL_SIZE: u32 = 100;
    const DEFAULT_CONNECTION_TIMEOUT: u64 = 30;
    const DEFAULT_STATEMENT_TIMEOUT: u64 = 30;

    fn connection_config(&self) -> SqliteConnectionConfig {
        SqliteConnectionConfig {
            statement_timeout: self.statement_timeout,
            read_only: false,
        }
    }

    pub fn set_pool_size(&mut self, size: u32) {
        self.pool_size = size;
    }

    pub fn set_connection_timeout(&mut self, timeout: Duration) {
        self.connection_timeout = timeout;
    }

    pub fn set_statement_timeout(&mut self, timeout: Duration) {
        self.statement_timeout = timeout;
    }
}

impl Default for SqliteConnectionPoolConfig {
    fn default() -> Self {
        let db_pool_size = std::env::var("DB_POOL_SIZE")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(Self::DEFAULT_POOL_SIZE);
        let conn_timeout_secs = std::env::var("DB_CONNECTION_TIMEOUT")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(Self::DEFAULT_CONNECTION_TIMEOUT);
        let statement_timeout_secs = std::env::var("DB_STATEMENT_TIMEOUT")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(Self::DEFAULT_STATEMENT_TIMEOUT);

        Self {
            pool_size: db_pool_size,
            connection_timeout: Duration::from_secs(conn_timeout_secs),
            statement_timeout: Duration::from_secs(statement_timeout_secs),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SqliteConnectionConfig {
    statement_timeout: Duration,
    read_only: bool,
}

impl diesel::r2d2::CustomizeConnection<SqliteConnection, diesel::r2d2::Error>
    for SqliteConnectionConfig
{
    fn on_acquire(
        &self,
        conn: &mut SqliteConnection,
    ) -> std::result::Result<(), diesel::r2d2::Error> {
        use diesel::{sql_query, RunQueryDsl};

        sql_query(format!(
            "SET statement_timeout = {}",
            self.statement_timeout.as_millis(),
        ))
        .execute(conn)
        .map_err(diesel::r2d2::Error::QueryError)?;

        if self.read_only {
            sql_query("SET default_transaction_read_only = 't'")
                .execute(conn)
                .map_err(diesel::r2d2::Error::QueryError)?;
        }

        Ok(())
    }
}

pub fn get_sqlite_pool_connection(
    pool: &SqliteConnectionPool,
) -> Result<SqlitePoolConnection, IndexerError> {
    pool.get().map_err(|e| {
        IndexerError::SqlitePoolConnectionError(format!(
            "Failed to get connection from PG connection pool with error: {:?}",
            e
        ))
    })
}
