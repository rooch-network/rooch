// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::types::IndexerResult;
use crate::{
    errors::IndexerError, models::transactions::StoredTransaction, SqliteConnectionConfig,
    SqliteConnectionPoolConfig, SqlitePoolConnection,
};
use anyhow::{anyhow, Result};
use diesel::{r2d2::ConnectionManager, SqliteConnection};
use rooch_types::transaction::TransactionWithInfo;

pub const SEQUENCE_NUMBER_STR: &str = "sequence_number";
pub const EVENT_SEQ_STR: &str = "event_seq";

#[derive(Clone)]
pub struct IndexerReader {
    pool: crate::SqliteConnectionPool,
}

// Impl for common initialization and utilities
impl IndexerReader {
    pub fn new<T: Into<String>>(db_url: T) -> Result<Self> {
        let config = SqliteConnectionPoolConfig::default();
        Self::new_with_config(db_url, config)
    }

    pub fn new_with_config<T: Into<String>>(
        db_url: T,
        config: SqliteConnectionPoolConfig,
    ) -> Result<Self> {
        let manager = ConnectionManager::<SqliteConnection>::new(db_url);

        let connection_config = SqliteConnectionConfig {
            statement_timeout: config.statement_timeout,
            read_only: true,
        };

        let pool = diesel::r2d2::Pool::builder()
            .max_size(config.pool_size)
            .connection_timeout(config.connection_timeout)
            .connection_customizer(Box::new(connection_config))
            .build(manager)
            .map_err(|e| anyhow!("Failed to initialize connection pool. Error: {:?}. If Error is None, please check whether the configured pool size (currently {}) exceeds the maximum number of connections allowed by the database.", e, config.pool_size))?;

        Ok(Self { pool })
    }

    fn get_connection(&self) -> Result<SqlitePoolConnection, IndexerError> {
        self.pool.get().map_err(|e| {
            IndexerError::SqlitePoolConnectionError(format!(
                "Failed to get connection from SQLite connection pool with error: {:?}",
                e
            ))
        })
    }

    pub fn run_query<T, E, F>(&self, _query: F) -> Result<T, IndexerError>
    where
        F: FnOnce(&mut SqliteConnection) -> Result<T, E>,
        E: From<diesel::result::Error> + std::error::Error,
    {
        blocking_call_is_ok_or_panic();

        // let mut connection = self.get_connection()?;

        // connection
        //     .transaction(query)
        //     .map_err(|e| IndexerError::SQLiteReadError(e.to_string()))

        // TransactionBuilder::new(&mut connection)
        //     .read_only()
        //     .run(query)
        //     .map_err(|e| IndexerError::SQLiteReadError(e.to_string()))

        //TODO implements sqlite query
        Err(IndexerError::SQLiteReadError("Not implements".to_string()))
    }

    pub async fn spawn_blocking<F, R, E>(&self, f: F) -> Result<R, E>
    where
        F: FnOnce(Self) -> Result<R, E> + Send + 'static,
        R: Send + 'static,
        E: Send + 'static,
    {
        let this = self.clone();
        let current_span = tracing::Span::current();
        tokio::task::spawn_blocking(move || {
            CALLED_FROM_BLOCKING_POOL
                .with(|in_blocking_pool| *in_blocking_pool.borrow_mut() = true);
            let _guard = current_span.enter();
            f(this)
        })
        .await
        .expect("propagate any panics")
    }

    pub async fn run_query_async<T, E, F>(&self, query: F) -> Result<T, IndexerError>
    where
        F: FnOnce(&mut SqliteConnection) -> Result<T, E> + Send + 'static,
        E: From<diesel::result::Error> + std::error::Error + Send + 'static,
        T: Send + 'static,
    {
        self.spawn_blocking(move |this| this.run_query(query)).await
    }
}

thread_local! {
    static CALLED_FROM_BLOCKING_POOL: std::cell::RefCell<bool> = std::cell::RefCell::new(false);
}

/// Check that we are in a context conducive to making blocking calls.
/// This is done by either:
/// - Checking that we are not inside a tokio runtime context
/// Or:
/// - If we are inside a tokio runtime context, ensure that the call went through
/// `IndexerReader::spawn_blocking` which properly moves the blocking call to a blocking thread
/// pool.
fn blocking_call_is_ok_or_panic() {
    if tokio::runtime::Handle::try_current().is_ok()
        && !CALLED_FROM_BLOCKING_POOL.with(|in_blocking_pool| *in_blocking_pool.borrow())
    {
        panic!(
            "You are calling a blocking DB operation directly on an async thread. \
                Please use IndexerReader::spawn_blocking instead to move the \
                operation to a blocking thread"
        );
    }
}

// Impl for reading data from the DB
impl IndexerReader {
    fn multi_get_transactions(
        &self,
        _tx_orders: Vec<i64>,
    ) -> Result<Vec<StoredTransaction>, IndexerError> {
        // // TODO multi_get
        // self.run_query(|conn| {
        //     transactions::table
        //         .filter(transactions::tx_order.eq_any(tx_orders))
        //         .load::<StoredTransaction>(conn)
        // })

        //TODO multi_get
        Ok(vec![])
    }

    fn stored_transaction_to_transaction_block(
        &self,
        stored_transactions: Vec<StoredTransaction>,
    ) -> IndexerResult<Vec<TransactionWithInfo>> {
        stored_transactions
            .into_iter()
            .map(|stored_transaction| stored_transaction.try_into_transaction_with_info())
            .collect::<IndexerResult<Vec<_>>>()
    }
}
