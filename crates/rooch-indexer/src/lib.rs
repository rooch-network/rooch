// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::store::metrics::IndexerDBMetrics;
use crate::store::sqlite_store::SqliteIndexerStore;
use crate::store::traits::IndexerStoreTrait;
use crate::utils::create_all_tables_if_not_exists;
use anyhow::Result;
use diesel::connection::SimpleConnection;
use diesel::r2d2::ConnectionManager;
use diesel::sqlite::SqliteConnection;
use diesel::ConnectionError::BadConnection;
use diesel::RunQueryDsl;
use errors::IndexerError;
use once_cell::sync::Lazy;
use prometheus::Registry;
use rooch_types::indexer::event::IndexerEvent;
use rooch_types::indexer::state::{
    IndexerObjectState, IndexerObjectStateChangeSet, IndexerObjectStateChanges, ObjectStateType,
};
use rooch_types::indexer::transaction::IndexerTransaction;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;
use std::string::ToString;
use std::sync::{Arc, RwLock};
use std::time::Duration;

pub mod actor;
pub mod errors;
pub mod indexer_reader;
pub mod metrics;
pub mod models;
pub mod proxy;
pub mod schema;
pub mod store;
#[cfg(test)]
mod tests;
pub mod utils;

/// Type alias to improve readability.
pub type IndexerResult<T> = Result<T, IndexerError>;

pub const DEFAULT_BUSY_TIMEOUT: u64 = 5000; // millsecond
pub type IndexerTableName = &'static str;
pub const INDEXER_EVENTS_TABLE_NAME: IndexerTableName = "events";
pub const INDEXER_OBJECT_STATES_TABLE_NAME: IndexerTableName = "object_states";
pub const INDEXER_OBJECT_STATE_UTXOS_TABLE_NAME: IndexerTableName = "utxos";
pub const INDEXER_OBJECT_STATE_INSCRIPTIONS_TABLE_NAME: IndexerTableName = "inscriptions";
pub const INDEXER_TRANSACTIONS_TABLE_NAME: IndexerTableName = "transactions";

/// Please note that adding new indexer table needs to be added in vec simultaneously.
static INDEXER_VEC_TABLE_NAME: Lazy<Vec<IndexerTableName>> = Lazy::new(|| {
    vec![
        INDEXER_EVENTS_TABLE_NAME,
        INDEXER_OBJECT_STATES_TABLE_NAME,
        INDEXER_OBJECT_STATE_UTXOS_TABLE_NAME,
        INDEXER_OBJECT_STATE_INSCRIPTIONS_TABLE_NAME,
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
    pub fn new(db_path: PathBuf, registry: &Registry) -> Result<Self> {
        if !db_path.exists() {
            std::fs::create_dir_all(&db_path)?;
        }
        let tables = IndexerStoreMeta::get_indexer_table_names().to_vec();

        let db_metrics = IndexerDBMetrics::get_or_init(registry).clone();
        let mut sqlite_store_mapping = HashMap::<String, SqliteIndexerStore>::new();
        for table in tables {
            let indexer_db_path = db_path.as_path().join(table);
            if !indexer_db_path.exists() {
                std::fs::File::create(indexer_db_path.clone())?;
            };
            let indexer_db_url = indexer_db_path
                .to_str()
                .ok_or(anyhow::anyhow!("Invalid indexer db path"))?
                .to_string();
            let sqlite_cp = new_sqlite_connection_pool(indexer_db_url.as_str())?;
            let sqlite_store = SqliteIndexerStore::new(sqlite_cp, db_metrics.clone());
            sqlite_store_mapping.insert(table.to_string(), sqlite_store);
        }

        let store = Self {
            sqlite_store_mapping,
        };
        store.create_all_tables_if_not_exists()?;
        Ok(store)
    }

    pub fn get_sqlite_store(&self, table_name: &str) -> Result<SqliteIndexerStore> {
        Ok(self
            .sqlite_store_mapping
            .get(table_name)
            .ok_or(anyhow::anyhow!("Sqlite store not exist"))?
            .clone())
    }

    fn create_all_tables_if_not_exists(&self) -> Result<()> {
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
    fn apply_object_states(
        &self,
        object_state_change_set: IndexerObjectStateChangeSet,
    ) -> Result<(), IndexerError> {
        self.update_object_states_by_state_type(
            object_state_change_set.object_states,
            INDEXER_OBJECT_STATES_TABLE_NAME,
            ObjectStateType::ObjectState,
        )?;
        self.update_object_states_by_state_type(
            object_state_change_set.object_state_utxos,
            INDEXER_OBJECT_STATE_UTXOS_TABLE_NAME,
            ObjectStateType::UTXO,
        )?;
        self.update_object_states_by_state_type(
            object_state_change_set.object_state_inscriptions,
            INDEXER_OBJECT_STATE_INSCRIPTIONS_TABLE_NAME,
            ObjectStateType::Inscription,
        )
    }

    fn persist_or_update_object_states(
        &self,
        states: Vec<IndexerObjectState>,
    ) -> Result<(), IndexerError> {
        self.get_sqlite_store(INDEXER_OBJECT_STATES_TABLE_NAME)?
            .persist_or_update_object_states(states)
    }

    fn delete_object_states(&self, state_pks: Vec<String>) -> Result<(), IndexerError> {
        self.get_sqlite_store(INDEXER_OBJECT_STATES_TABLE_NAME)?
            .delete_object_states(state_pks)
    }

    fn persist_or_update_object_state_utxos(
        &self,
        states: Vec<IndexerObjectState>,
    ) -> Result<(), IndexerError> {
        self.get_sqlite_store(INDEXER_OBJECT_STATE_UTXOS_TABLE_NAME)?
            .persist_or_update_object_state_utxos(states)
    }

    fn delete_object_state_utxos(&self, state_pks: Vec<String>) -> Result<(), IndexerError> {
        self.get_sqlite_store(INDEXER_OBJECT_STATE_UTXOS_TABLE_NAME)?
            .delete_object_state_utxos(state_pks)
    }

    fn persist_or_update_object_state_inscriptions(
        &self,
        states: Vec<IndexerObjectState>,
    ) -> Result<(), IndexerError> {
        self.get_sqlite_store(INDEXER_OBJECT_STATE_INSCRIPTIONS_TABLE_NAME)?
            .persist_or_update_object_state_inscriptions(states)
    }

    fn delete_object_state_inscriptions(&self, state_pks: Vec<String>) -> Result<(), IndexerError> {
        self.get_sqlite_store(INDEXER_OBJECT_STATE_INSCRIPTIONS_TABLE_NAME)?
            .delete_object_state_inscriptions(state_pks)
    }

    fn persist_transactions(
        &self,
        transactions: Vec<IndexerTransaction>,
    ) -> Result<(), IndexerError> {
        self.get_sqlite_store(INDEXER_TRANSACTIONS_TABLE_NAME)?
            .persist_transactions(transactions)
    }

    fn persist_events(&self, events: Vec<IndexerEvent>) -> Result<(), IndexerError> {
        self.get_sqlite_store(INDEXER_EVENTS_TABLE_NAME)?
            .persist_events(events)
    }

    fn delete_transactions(&self, tx_orders: Vec<u64>) -> Result<(), IndexerError> {
        self.get_sqlite_store(INDEXER_TRANSACTIONS_TABLE_NAME)?
            .delete_transactions(tx_orders)
    }

    fn delete_events(&self, tx_orders: Vec<u64>) -> Result<(), IndexerError> {
        self.get_sqlite_store(INDEXER_EVENTS_TABLE_NAME)?
            .delete_events(tx_orders)
    }
}

impl IndexerStore {
    fn update_object_states_by_state_type(
        &self,
        mut object_state_change: IndexerObjectStateChanges,
        table_name: &str,
        state_type: ObjectStateType,
    ) -> Result<(), IndexerError> {
        let mut object_states_new_and_update = object_state_change.new_object_states;
        object_states_new_and_update.append(&mut object_state_change.update_object_states);
        match state_type {
            ObjectStateType::ObjectState => {
                self.get_sqlite_store(table_name)?
                    .persist_or_update_object_states(object_states_new_and_update)?;
                self.get_sqlite_store(table_name)?
                    .delete_object_states(object_state_change.remove_object_states)
            }
            ObjectStateType::UTXO => {
                self.get_sqlite_store(table_name)?
                    .persist_or_update_object_state_utxos(object_states_new_and_update)?;
                self.get_sqlite_store(table_name)?
                    .delete_object_state_utxos(object_state_change.remove_object_states)
            }
            ObjectStateType::Inscription => {
                self.get_sqlite_store(table_name)?
                    .persist_or_update_object_state_inscriptions(object_states_new_and_update)?;
                self.get_sqlite_store(table_name)?
                    .delete_object_state_inscriptions(object_state_change.remove_object_states)
            }
        }
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
    const DEFAULT_POOL_SIZE: u32 = 16;
    const DEFAULT_READER_POOL_SIZE: u32 = 32;
    const DEFAULT_CONNECTION_TIMEOUT: u64 = 120; // second

    fn connection_config(&self) -> SqliteConnectionConfig {
        let locker = Arc::new(RwLock::new(0));
        SqliteConnectionConfig {
            read_only: false,
            enable_wal: true,
            busy_timeout: DEFAULT_BUSY_TIMEOUT,
            locker,
        }
    }

    pub fn set_pool_size(&mut self, size: u32) {
        self.pool_size = size;
    }

    pub fn set_connection_timeout(&mut self, timeout: Duration) {
        self.connection_timeout = timeout;
    }

    pub fn pool_config(read_only: bool) -> Self {
        let default_pool_size = if read_only {
            Self::DEFAULT_READER_POOL_SIZE
        } else {
            Self::DEFAULT_POOL_SIZE
        };
        let db_pool_size = std::env::var("DB_POOL_SIZE")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(default_pool_size);
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

impl Default for SqliteConnectionPoolConfig {
    fn default() -> Self {
        Self::pool_config(false)
    }
}

#[derive(Debug, Clone)]
struct SqliteConnectionConfig {
    // SQLite does not support the statement_timeout parameter
    read_only: bool,
    enable_wal: bool,
    busy_timeout: u64,
    locker: Arc<RwLock<u64>>,
}

impl diesel::r2d2::CustomizeConnection<SqliteConnection, diesel::r2d2::Error>
    for SqliteConnectionConfig
{
    fn on_acquire(
        &self,
        conn: &mut SqliteConnection,
    ) -> std::result::Result<(), diesel::r2d2::Error> {
        let mut locker_cnt = self
            .locker
            .write()
            .map_err(|e| diesel::r2d2::Error::ConnectionError(BadConnection(e.to_string())))?;
        *locker_cnt += 1;
        tracing::trace!(
            "Sqlite CustomizeConnection on_acquire connection [rw:{}]",
            *locker_cnt
        );
        // https://github.com/diesel-rs/diesel/issues/2365
        diesel::sql_query(format!("PRAGMA busy_timeout = {};", self.busy_timeout))
            .execute(conn)
            .map_err(diesel::r2d2::Error::QueryError)?;

        let mut pragma_builder = String::new();
        if self.read_only {
            pragma_builder.push_str("PRAGMA query_only = true;");
            // The default page_size is 1024 byte.
            // The default cache_size value is -2000, which translates into a maximum of 2048000 bytes per cache.
            // The cache_size in SQLite is primarily associated with the database connection, not the database file itself.
            // pragma_builder.push_str("PRAGMA page_size = 4096; PRAGMA cache_size = 524288000;"); //512MB
            pragma_builder.push_str("PRAGMA page_size = 4096; PRAGMA cache_size = 262144000;");
            //256MB
        }
        // WAL mode has better write-concurrency. When synchronous is NORMAL it will fsync only in critical moments
        if self.enable_wal {
            pragma_builder.push_str("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;");
        }
        // The mmap_size in SQLite is primarily associated with the database file, not the connection.
        pragma_builder.push_str("PRAGMA mmap_size = 1073741824"); // 1GB
                                                                  // pragma_builder.push_str("PRAGMA mmap_size = 536870912"); // 512MB
        conn.batch_execute(&pragma_builder)
            .map_err(diesel::r2d2::Error::QueryError)?;

        Ok(())
    }

    fn on_release(&self, _conn: SqliteConnection) {
        tracing::trace!("Sqlite CustomizeConnection on_release connection");
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
