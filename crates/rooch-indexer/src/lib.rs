// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::fmt::{Debug, Display, Formatter};
use std::time::Duration;

use anyhow::Result;
use diesel::r2d2::ConnectionManager;
use diesel::sqlite::SqliteConnection;
// use tracing::info;

use crate::store::sqlite_store::SqliteIndexerStore;
use crate::store::traits::IndexerStoreTrait;
use crate::types::{IndexedEvent, IndexedTransaction};
use errors::IndexerError;
use moveos_types::h256::H256;
use rooch_types::transaction::TransactionWithInfo;

pub mod actor;
pub mod errors;
pub mod indexer_reader;
pub mod models;
pub mod proxy;
pub mod schema;
pub mod store;
pub mod types;
pub mod utils;

pub type SqliteConnectionPool = diesel::r2d2::Pool<ConnectionManager<SqliteConnection>>;
pub type SqlitePoolConnection = diesel::r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;

/// Returns all endpoints for which we have implemented on the indexer,
/// some of them are not validated yet.
/// NOTE: we only use this for integration testing
// const IMPLEMENTED_METHODS: [&str; 4] = [
//     "multi_get_transactions",
//     "multi_get_events",
//     // indexer apis
//     "query_transactions",
//     "query_events",
// ];

#[derive(Clone)]
pub struct IndexerStore {
    pub sqlite_store: SqliteIndexerStore,
}

impl IndexerStore {
    pub fn new(db_url: &str) -> Result<Self> {
        let sqlite_cp = new_sqlite_connection_pool(db_url)?;
        let store = Self {
            sqlite_store: SqliteIndexerStore::new(sqlite_cp),
        };
        Ok(store)
    }

    pub fn mock_indexer_store() -> Result<Self> {
        let tmpdir = moveos_config::temp_dir();
        let db_url = tmpdir
            .path()
            .to_str()
            .ok_or(anyhow::anyhow!("Invalid indexer db temp dir"))?;
        Self::new(db_url)
    }
}

impl Display for IndexerStore {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.clone())
    }
}
impl Debug for IndexerStore {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

pub fn new_sqlite_connection_pool(db_url: &str) -> Result<SqliteConnectionPool, IndexerError> {
    new_sqlite_connection_pool_impl(db_url, None)
}

impl IndexerStoreTrait for IndexerStore {
    fn persist_transactions(
        &self,
        transactions: Vec<IndexedTransaction>,
    ) -> Result<(), IndexerError> {
        self.sqlite_store.persist_transactions(transactions)
    }

    fn persist_events(&self, events: Vec<IndexedEvent>) -> Result<(), IndexerError> {
        self.sqlite_store.persist_events(events)
    }

    fn query_transactions_by_hash(
        &self,
        _tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<TransactionWithInfo>>, IndexerError> {
        Ok(vec![])
    }
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
}

impl SqliteConnectionPoolConfig {
    const DEFAULT_POOL_SIZE: u32 = 100;
    const DEFAULT_CONNECTION_TIMEOUT: u64 = 30;

    fn connection_config(&self) -> SqliteConnectionConfig {
        SqliteConnectionConfig { read_only: false }
    }

    pub fn set_pool_size(&mut self, size: u32) {
        self.pool_size = size;
    }

    pub fn set_connection_timeout(&mut self, timeout: Duration) {
        self.connection_timeout = timeout;
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

        Self {
            pool_size: db_pool_size,
            connection_timeout: Duration::from_secs(conn_timeout_secs),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SqliteConnectionConfig {
    // SQLite does not support the statement_timeout parameter
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

        // This will disable uncommitted reads, putting the connection into read-only mode
        if self.read_only {
            sql_query("PRAGMA read_uncommitted = 0")
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
            "Failed to get connection from SQLite connection pool with error: {:?}",
            e
        ))
    })
}
