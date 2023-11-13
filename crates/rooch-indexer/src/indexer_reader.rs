// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::types::IndexerResult;
use crate::{
    errors::IndexerError, models::transactions::StoredTransaction, SqliteConnectionConfig,
    SqliteConnectionPoolConfig, SqlitePoolConnection,
};
use anyhow::{anyhow, Result};
use diesel::{
    r2d2::ConnectionManager, Connection, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection,
};

use crate::models::events::StoredEvent;
use crate::schema::{events, transactions};
use rooch_types::indexer::event_filter::{EventFilter, IndexerEvent, IndexerEventID};
use rooch_types::transaction::TransactionWithInfo;

pub const EVENT_HANDLE_ID_STR: &str = "event_handle_id";
pub const EVENT_INDEX_STR: &str = "event_index";
pub const EVENT_SEQ_STR: &str = "event_seq";
pub const TX_ORDER_STR: &str = "tx_order";
pub const TX_HASH_STR: &str = "tx_hash";
pub const EVENT_SENDER_STR: &str = "sender";
pub const EVENT_TYPE_STR: &str = "event_type";
pub const EVENT_CREATED_TIME_STR: &str = "created_time";

#[derive(Clone)]
pub(crate) struct InnerIndexerReader {
    pool: crate::SqliteConnectionPool,
}

// Impl for common initialization and utilities
#[allow(unused)]
impl InnerIndexerReader {
    pub fn new<T: Into<String>>(db_url: T) -> Result<Self> {
        let config = SqliteConnectionPoolConfig::default();
        Self::new_with_config(db_url, config)
    }

    pub fn new_with_config<T: Into<String>>(
        db_url: T,
        config: SqliteConnectionPoolConfig,
    ) -> Result<Self> {
        let manager = ConnectionManager::<SqliteConnection>::new(db_url);

        let connection_config = SqliteConnectionConfig { read_only: true };

        let pool = diesel::r2d2::Pool::builder()
            .max_size(config.pool_size)
            .connection_timeout(config.connection_timeout)
            .connection_customizer(Box::new(connection_config))
            .build(manager)
            .map_err(|e| anyhow!("Failed to initialize connection pool. Error: {:?}. If Error is None, please check whether the configured pool size (currently {}) exceeds the maximum number of connections allowed by the database.", e, config.pool_size))?;

        Ok(Self { pool })
    }

    pub fn get_connection(&self) -> Result<SqlitePoolConnection, IndexerError> {
        self.pool.get().map_err(|e| {
            IndexerError::SqlitePoolConnectionError(format!(
                "Failed to get connection from SQLite connection pool with error: {:?}",
                e
            ))
        })
    }

    pub fn run_query<T, E, F>(&self, query: F) -> Result<T, IndexerError>
    where
        F: FnOnce(&mut SqliteConnection) -> Result<T, E>,
        E: From<diesel::result::Error> + std::error::Error,
    {
        //TODO implements sqlite query

        let mut connection = self.get_connection()?;

        connection
            .transaction(query)
            .map_err(|e| IndexerError::SQLiteReadError(e.to_string()))

        // Err(IndexerError::SQLiteReadError("Not implements".to_string()))
    }

    // pub async fn spawn_blocking<F, R, E>(&self, f: F) -> Result<R, E>
    // where
    //     F: FnOnce(Self) -> Result<R, E> + Send + 'static,
    //     R: Send + 'static,
    //     E: Send + 'static,
    // {
    //     let this = self.clone();
    //     let current_span = tracing::Span::current();
    //     tokio::task::spawn_blocking(move || {
    //         CALLED_FROM_BLOCKING_POOL
    //             .with(|in_blocking_pool| *in_blocking_pool.borrow_mut() = true);
    //         let _guard = current_span.enter();
    //         f(this)
    //     })
    //     .await
    //     .expect("propagate any panics")
    // }

    // pub async fn run_query_async<T, E, F>(&self, query: F) -> Result<T, IndexerError>
    // where
    //     F: FnOnce(&mut SqliteConnection) -> Result<T, E> + Send + 'static,
    //     E: From<diesel::result::Error> + std::error::Error + Send + 'static,
    //     T: Send + 'static,
    // {
    //     self.spawn_blocking(move |this| this.run_query(query)).await
    // }
}

// thread_local! {
//     static CALLED_FROM_BLOCKING_POOL: std::cell::RefCell<bool> = std::cell::RefCell::new(false);
// }

/// Check that we are in a context conducive to making blocking calls.
/// This is done by either:
/// - Checking that we are not inside a tokio runtime context
/// Or:
/// - If we are inside a tokio runtime context, ensure that the call went through
/// `IndexerReader::spawn_blocking` which properly moves the blocking call to a blocking thread
/// pool.
// fn blocking_call_is_ok_or_panic() {
//     if tokio::runtime::Handle::try_current().is_ok()
//         && !CALLED_FROM_BLOCKING_POOL.with(|in_blocking_pool| *in_blocking_pool.borrow())
//     {
//         panic!(
//             "You are calling a blocking DB operation directly on an async thread. \
//                 Please use IndexerReader::spawn_blocking instead to move the \
//                 operation to a blocking thread"
//         );
//     }
// }

#[derive(Clone)]
pub struct IndexerReader {
    pub(crate) inner_indexer_reader: InnerIndexerReader,
}

// Impl for reading data from the DB
impl IndexerReader {
    pub fn new<T: Into<String>>(db_url: T) -> Result<Self> {
        let inner_indexer_reader = InnerIndexerReader::new(db_url)?;
        Ok(IndexerReader {
            inner_indexer_reader,
        })
    }

    pub fn new_with_config<T: Into<String>>(
        db_url: T,
        config: SqliteConnectionPoolConfig,
    ) -> Result<Self> {
        let inner_indexer_reader = InnerIndexerReader::new_with_config(db_url, config)?;
        Ok(IndexerReader {
            inner_indexer_reader,
        })
    }

    pub fn multi_get_transactions(
        &self,
        tx_orders: Vec<i64>,
    ) -> Result<Vec<StoredTransaction>, IndexerError> {
        // TODO multi_get
        self.inner_indexer_reader.run_query(|conn| {
            transactions::table
                .filter(transactions::tx_order.eq_any(tx_orders.as_slice()))
                .load::<StoredTransaction>(conn)
        })

        // Ok(vec![])
    }

    pub fn stored_transaction_to_transaction_block(
        &self,
        stored_transactions: Vec<StoredTransaction>,
    ) -> IndexerResult<Vec<TransactionWithInfo>> {
        stored_transactions
            .into_iter()
            .map(|stored_transaction| stored_transaction.try_into_transaction_with_info())
            .collect::<IndexerResult<Vec<_>>>()
    }

    pub fn query_events_with_filter(
        &self,
        filter: EventFilter,
        cursor: Option<IndexerEventID>,
        limit: usize,
        descending_order: bool,
    ) -> IndexerResult<Vec<IndexerEvent>> {
        let (tx_order, event_index) = if let Some(cursor) = cursor.clone() {
            let IndexerEventID {
                tx_order,
                event_index,
            } = cursor;
            (tx_order, event_index)
        } else if descending_order {
            let (max_tx_order, event_index): (i128, u64) =
                self.inner_indexer_reader.run_query(|conn| {
                    events::dsl::events
                        .select((events::tx_order, events::event_index))
                        // .order((events::tx_order.desc(), events::event_index.desc())
                        .order_by((events::tx_order.desc(), events::event_index.desc()))
                        .first::<(i128, u64)>(conn)
                })?;
            // (max_tx_order + 1, 0)
            (max_tx_order + 1, event_index)
        } else {
            (-1, 0)
        };

        let main_where_clause = match filter {
            EventFilter::EventType(struct_tag) => {
                format!("e.{EVENT_TYPE_STR} = {}", struct_tag.to_canonical_string())
            }
            EventFilter::Sender(sender) => {
                format!("e.{EVENT_SENDER_STR} = {}", sender.to_canonical_string())
            }
            EventFilter::TxHash(tx_hash) => {
                format!("e.{TX_HASH_STR} = {}", tx_hash.to_string())
            }
            EventFilter::TimeRange {
                start_time,
                end_time,
            } => {
                format!(
                    "(e.{EVENT_CREATED_TIME_STR} >= {} AND e.{EVENT_CREATED_TIME_STR} < {})",
                    start_time, end_time
                )
            }
            EventFilter::TxOrderRange {
                from_order,
                to_order,
            } => {
                format!(
                    "(e.{TX_ORDER_STR} >= {} AND e.{TX_ORDER_STR} < {})",
                    from_order, to_order
                )
            }
        };

        let cursor_clause = if descending_order {
            format!(
                "AND ({TX_ORDER_STR} < {} OR ({TX_ORDER_STR} = {} AND {EVENT_INDEX_STR} < {}))",
                tx_order, tx_order, event_index
            )
        } else {
            format!(
                "AND ({TX_ORDER_STR} > {} OR ({TX_ORDER_STR} = {} AND {EVENT_INDEX_STR} > {}))",
                tx_order, tx_order, event_index
            )
        };
        let order_clause = if descending_order {
            format!("{TX_ORDER_STR} DESC, {EVENT_INDEX_STR} DESC")
        } else {
            format!("{TX_ORDER_STR} ASC, {EVENT_INDEX_STR} ASC")
        };

        let query = format!(
            "
                SELECT * FROM events \
                WHERE {} {} \
                ORDER BY {} \
                LIMIT {}
            ",
            main_where_clause, cursor_clause, order_clause, limit,
        );

        tracing::debug!("query events: {}", query);
        let stored_events = self
            .inner_indexer_reader
            .run_query(|conn| diesel::sql_query(query).load::<StoredEvent>(conn))?;

        let result = stored_events
            .into_iter()
            .map(|se| IndexerEvent::from)
            .collect();

        println!("Debug indexer reader run_query result {:?}", result);
        Ok(result)
    }
}
