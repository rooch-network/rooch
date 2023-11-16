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
use std::ops::DerefMut;

use crate::models::events::StoredEvent;
use crate::schema::{events, transactions};
use rooch_types::indexer::event_filter::{EventFilter, IndexerEvent, IndexerEventID};
use rooch_types::indexer::transaction_filter::TransactionFilter;
use rooch_types::transaction::TransactionWithInfo;

pub const TX_ORDER_STR: &str = "tx_order";
pub const TX_HASH_STR: &str = "tx_hash";
pub const TX_SENDER_STR: &str = "sender";
pub const CREATED_AT_STR: &str = "created_at";

pub const TRANSACTION_SENDER_STR: &str = "sender";
pub const TRANSACTION_TYPE_STR: &str = "event_type";

pub const EVENT_HANDLE_ID_STR: &str = "event_handle_id";
pub const EVENT_INDEX_STR: &str = "event_index";
pub const EVENT_SEQ_STR: &str = "event_seq";
pub const EVENT_TYPE_STR: &str = "event_type";

#[derive(Clone)]
pub(crate) struct InnerIndexerReader {
    pool: crate::SqliteConnectionPool,
}

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
        let mut connection = self.get_connection()?;
        connection
            .deref_mut()
            .transaction(query)
            .map_err(|e| IndexerError::SQLiteReadError(e.to_string()))
    }
}

#[derive(Clone)]
pub struct IndexerReader {
    pub(crate) inner_indexer_reader: InnerIndexerReader,
}

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

    pub fn query_transactions_with_filter(
        &self,
        filter: TransactionFilter,
        cursor: Option<u64>,
        limit: usize,
        descending_order: bool,
    ) -> IndexerResult<Vec<TransactionWithInfo>> {
        let tx_order = if let Some(cursor) = cursor {
            cursor as i64
        } else if descending_order {
            let max_tx_order: i64 = self.inner_indexer_reader.run_query(|conn| {
                transactions::dsl::transactions
                    .select(transactions::tx_order)
                    .order_by(transactions::tx_order.desc())
                    .first::<i64>(conn)
            })?;
            max_tx_order + 1
        } else {
            -1
        };

        let main_where_clause = match filter {
            TransactionFilter::Sender(sender) => {
                format!("{TX_SENDER_STR} = \"{}\"", sender.to_hex_literal())
            }
            TransactionFilter::TxHashes(tx_hashes) => {
                let in_tx_hashes_str: String = tx_hashes
                    .iter()
                    .map(|tx_hash| format!("\"{:?}\"", tx_hash))
                    .collect::<Vec<String>>()
                    .join(",");
                format!("{TX_HASH_STR} in ({})", in_tx_hashes_str)
            }
            TransactionFilter::TimeRange {
                start_time,
                end_time,
            } => {
                format!(
                    "({CREATED_AT_STR} >= {} AND {CREATED_AT_STR} < {})",
                    start_time, end_time
                )
            }
            TransactionFilter::TxOrderRange {
                from_order,
                to_order,
            } => {
                format!(
                    "({TX_ORDER_STR} >= {} AND {TX_ORDER_STR} < {})",
                    from_order, to_order
                )
            }
        };

        let cursor_clause = if descending_order {
            format!("AND ({TX_ORDER_STR} < {})", tx_order)
        } else {
            format!("AND ({TX_ORDER_STR} > {})", tx_order)
        };
        let order_clause = if descending_order {
            format!("{TX_ORDER_STR} DESC")
        } else {
            format!("{TX_ORDER_STR} ASC")
        };

        let query = format!(
            "
                SELECT * FROM transactions \
                WHERE {} {} \
                ORDER BY {} \
                LIMIT {}
            ",
            main_where_clause, cursor_clause, order_clause, limit,
        );

        tracing::debug!("query transactions: {}", query);
        let stored_transactions = self
            .inner_indexer_reader
            .run_query(|conn| diesel::sql_query(query).load::<StoredTransaction>(conn))?;

        let result = stored_transactions
            .into_iter()
            .map(|t| t.try_into_transaction_with_info())
            .collect::<Result<Vec<_>>>()
            .map_err(|e| {
                IndexerError::SQLiteReadError(format!("Cast indexer transactions failed: {:?}", e))
            })?;

        Ok(result)
    }

    pub fn query_events_with_filter(
        &self,
        filter: EventFilter,
        cursor: Option<IndexerEventID>,
        limit: usize,
        descending_order: bool,
    ) -> IndexerResult<Vec<IndexerEvent>> {
        let (tx_order, event_index) = if let Some(cursor) = cursor {
            let IndexerEventID {
                tx_order,
                event_index,
            } = cursor;
            (tx_order as i64, event_index as i64)
        } else if descending_order {
            let (max_tx_order, event_index): (i64, i64) =
                self.inner_indexer_reader.run_query(|conn| {
                    events::dsl::events
                        .select((events::tx_order, events::event_index))
                        .order_by((events::tx_order.desc(), events::event_index.desc()))
                        .first::<(i64, i64)>(conn)
                })?;
            (max_tx_order + 1, event_index)
        } else {
            (-1, 0)
        };

        let main_where_clause = match filter {
            EventFilter::EventType(struct_tag) => {
                let event_type_str = format!("0x{}", struct_tag.to_canonical_string());
                format!("{EVENT_TYPE_STR} = \"{}\"", event_type_str)
            }
            EventFilter::Sender(sender) => {
                format!("{TX_SENDER_STR} = \"{}\"", sender.to_hex_literal())
            }
            EventFilter::TxHash(tx_hash) => {
                let tx_hash_str = format!("{:?}", tx_hash);
                format!("{TX_HASH_STR} = \"{}\"", tx_hash_str)
            }
            EventFilter::TimeRange {
                start_time,
                end_time,
            } => {
                format!(
                    "({CREATED_AT_STR} >= {} AND {CREATED_AT_STR} < {})",
                    start_time, end_time
                )
            }
            EventFilter::TxOrderRange {
                from_order,
                to_order,
            } => {
                format!(
                    "({TX_ORDER_STR} >= {} AND {TX_ORDER_STR} < {})",
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
            .map(|ev| ev.try_into_indexer_event())
            .collect::<Result<Vec<_>>>()
            .map_err(|e| {
                IndexerError::SQLiteReadError(format!("Cast indexer events failed: {:?}", e))
            })?;

        Ok(result)
    }
}
