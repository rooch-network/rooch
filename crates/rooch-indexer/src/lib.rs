// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;
use std::string::ToString;
use std::time::Duration;

use anyhow::Result;
use diesel::r2d2::ConnectionManager;
use diesel::sqlite::SqliteConnection;

use crate::store::sqlite_store::SqliteIndexerStore;
use crate::store::traits::IndexerStoreTrait;
use crate::types::{
    IndexedEvent, IndexedGlobalState, IndexedTableChangeSet, IndexedTableState, IndexedTransaction,
};
use crate::utils::create_all_tables_if_not_exists;
use errors::IndexerError;
use rooch_config::indexer_config::ROOCH_INDEXER_DB_DIR;

use once_cell::sync::Lazy;

pub mod actor;
pub mod errors;
pub mod indexer_reader;
pub mod models;
pub mod proxy;
pub mod schema;
pub mod store;
#[cfg(test)]
mod tests;
pub mod types;
pub mod utils;

/// Type alias to improve readability.
pub type IndexerTableName = &'static str;
pub const INDEXER_EVENTS_TABLE_NAME: IndexerTableName = "events";
pub const INDEXER_GLOBAL_STATES_TABLE_NAME: IndexerTableName = "global_states";
pub const INDEXER_TABLE_CHANGE_SETS_TABLE_NAME: IndexerTableName = "table_change_sets";
pub const INDEXER_TABLE_STATES_TABLE_NAME: IndexerTableName = "table_states";
pub const INDEXER_TRANSACTIONS_TABLE_NAME: IndexerTableName = "transactions";

/// Please note that adding new indexer table needs to be added in vec simultaneously.
static INDEXER_VEC_TABLE_NAME: Lazy<Vec<IndexerTableName>> = Lazy::new(|| {
    vec![
        INDEXER_EVENTS_TABLE_NAME,
        INDEXER_GLOBAL_STATES_TABLE_NAME,
        INDEXER_TABLE_CHANGE_SETS_TABLE_NAME,
        INDEXER_TABLE_STATES_TABLE_NAME,
        INDEXER_TRANSACTIONS_TABLE_NAME,
    ]
});

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct IndexerStoreMeta {}

impl IndexerStoreMeta {
    pub fn get_indexer_table_names() -> &'static [IndexerTableName] {
        &INDEXER_VEC_TABLE_NAME
    }
}

pub type SqliteConnectionPool = diesel::r2d2::Pool<ConnectionManager<SqliteConnection>>;
pub type SqlitePoolConnection = diesel::r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;

#[derive(Clone)]
pub struct IndexerStore {
    pub sqlite_store_mapping: HashMap<String, SqliteIndexerStore>,
}

impl IndexerStore {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let tables = IndexerStoreMeta::get_indexer_table_names().to_vec();

        let mut sqlite_store_mapping = HashMap::<String, SqliteIndexerStore>::new();
        for table in tables {
            let indexer_db_path = db_path.clone().join(table);
            if !indexer_db_path.exists() {
                std::fs::File::create(indexer_db_path.clone())?;
            };
            let indexer_db_url = indexer_db_path
                .to_str()
                .ok_or(anyhow::anyhow!("Invalid indexer db path"))?
                .to_string();
            let sqlite_cp = new_sqlite_connection_pool(indexer_db_url.as_str())?;
            let sqlite_store = SqliteIndexerStore::new(sqlite_cp);
            sqlite_store_mapping.insert(table.to_string(), sqlite_store);
        }

        let store = Self {
            sqlite_store_mapping,
        };
        Ok(store)
    }

    pub fn get_sqlite_store(&self, table_name: &str) -> Result<SqliteIndexerStore> {
        Ok(self
            .sqlite_store_mapping
            .get(table_name)
            .ok_or(anyhow::anyhow!("Sqlite store not exist"))?
            .clone())
    }

    pub fn mock_db_dir() -> Result<PathBuf> {
        let tmpdir = moveos_config::temp_dir();
        let indexer_db_dir = tmpdir.path().join(ROOCH_INDEXER_DB_DIR);

        if !indexer_db_dir.exists() {
            std::fs::create_dir_all(indexer_db_dir.clone())?;
        }
        Ok(indexer_db_dir)
    }

    pub fn mock_indexer_store() -> Result<Self> {
        let indexer_db_dir = Self::mock_db_dir()?;
        Self::new(indexer_db_dir)
    }

    pub fn create_all_tables_if_not_exists(&self) -> Result<()> {
        for (k, v) in &self.sqlite_store_mapping {
            let mut connection = get_sqlite_pool_connection(&v.connection_pool)?;
            create_all_tables_if_not_exists(&mut connection, k.clone())?;
        }
        Ok(())
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
    fn persist_or_update_global_states(
        &self,
        states: Vec<IndexedGlobalState>,
    ) -> Result<(), IndexerError> {
        self.get_sqlite_store(INDEXER_GLOBAL_STATES_TABLE_NAME)?
            .persist_or_update_global_states(states)
    }

    fn delete_global_states(&self, state_pks: Vec<String>) -> Result<(), IndexerError> {
        self.get_sqlite_store(INDEXER_GLOBAL_STATES_TABLE_NAME)?
            .delete_global_states(state_pks)
    }

    fn persist_or_update_table_states(
        &self,
        states: Vec<IndexedTableState>,
    ) -> Result<(), IndexerError> {
        self.get_sqlite_store(INDEXER_TABLE_STATES_TABLE_NAME)?
            .persist_or_update_table_states(states)
    }

    fn delete_table_states(&self, state_pks: Vec<(String, String)>) -> Result<(), IndexerError> {
        self.get_sqlite_store(INDEXER_TABLE_STATES_TABLE_NAME)?
            .delete_table_states(state_pks)
    }

    fn delete_table_states_by_table_handle(
        &self,
        table_handles: Vec<String>,
    ) -> Result<(), IndexerError> {
        self.get_sqlite_store(INDEXER_TABLE_STATES_TABLE_NAME)?
            .delete_table_states_by_table_handle(table_handles)
    }

    fn persist_table_change_sets(
        &self,
        table_change_sets: Vec<IndexedTableChangeSet>,
    ) -> Result<(), IndexerError> {
        self.get_sqlite_store(INDEXER_TABLE_CHANGE_SETS_TABLE_NAME)?
            .persist_table_change_sets(table_change_sets)
    }

    fn persist_transactions(
        &self,
        transactions: Vec<IndexedTransaction>,
    ) -> Result<(), IndexerError> {
        self.get_sqlite_store(INDEXER_TRANSACTIONS_TABLE_NAME)?
            .persist_transactions(transactions)
    }

    fn persist_events(&self, events: Vec<IndexedEvent>) -> Result<(), IndexerError> {
        self.get_sqlite_store(INDEXER_EVENTS_TABLE_NAME)?
            .persist_events(events)
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
    const DEFAULT_POOL_SIZE: u32 = 30;
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
