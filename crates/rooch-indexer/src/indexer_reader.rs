// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::types::IndexerResult;
use crate::{
    errors::IndexerError, SqliteConnectionConfig, SqliteConnectionPoolConfig, SqlitePoolConnection,
};
use anyhow::{anyhow, Result};
use diesel::{
    r2d2::ConnectionManager, Connection, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection,
};
use std::ops::DerefMut;

use crate::models::events::StoredEvent;
use crate::models::states::{StoredGlobalState, StoredTableChangeSet, StoredTableState};
use crate::models::transactions::StoredTransaction;
use crate::schema::global_states;
use crate::schema::{events, table_change_sets, table_states, transactions};
use crate::utils::format_struct_tag;
use rooch_types::indexer::event_filter::{EventFilter, IndexerEvent, IndexerEventID};
use rooch_types::indexer::state::{
    GlobalStateFilter, IndexerGlobalState, IndexerStateID, IndexerTableChangeSet,
    IndexerTableState, StateSyncFilter, TableStateFilter,
};
use rooch_types::indexer::transaction_filter::TransactionFilter;
use rooch_types::transaction::TransactionWithInfo;

pub const TX_ORDER_STR: &str = "tx_order";
pub const TX_HASH_STR: &str = "tx_hash";
pub const TX_SENDER_STR: &str = "sender";
pub const CREATED_AT_STR: &str = "created_at";
pub const OBJECT_ID_STR: &str = "object_id";

pub const TRANSACTION_ORIGINAL_ADDRESS_STR: &str = "multichain_original_address";

pub const EVENT_HANDLE_ID_STR: &str = "event_handle_id";
pub const EVENT_INDEX_STR: &str = "event_index";
pub const EVENT_SEQ_STR: &str = "event_seq";
pub const EVENT_TYPE_STR: &str = "event_type";

pub const STATE_TABLE_HANDLE_STR: &str = "table_handle";
pub const STATE_INDEX_STR: &str = "state_index";
pub const STATE_OBJECT_TYPE_STR: &str = "object_type";
pub const STATE_OWNER_STR: &str = "owner";

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
    //TODO split by table dimension
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
            TransactionFilter::OriginalAddress(address) => {
                format!("{TRANSACTION_ORIGINAL_ADDRESS_STR} = \"{}\"", address)
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

    pub fn query_global_states_with_filter(
        &self,
        filter: GlobalStateFilter,
        cursor: Option<IndexerStateID>,
        limit: usize,
        descending_order: bool,
    ) -> IndexerResult<Vec<IndexerGlobalState>> {
        let (tx_order, state_index) = if let Some(cursor) = cursor {
            let IndexerStateID {
                tx_order,
                state_index,
            } = cursor;
            (tx_order as i64, state_index as i64)
        } else if descending_order {
            let (max_tx_order, state_index): (i64, i64) =
                self.inner_indexer_reader.run_query(|conn| {
                    global_states::dsl::global_states
                        .select((global_states::tx_order, global_states::state_index))
                        .order_by((
                            global_states::tx_order.desc(),
                            global_states::state_index.desc(),
                        ))
                        .first::<(i64, i64)>(conn)
                })?;
            (max_tx_order + 1, state_index)
        } else {
            (-1, 0)
        };

        let main_where_clause = match filter {
            GlobalStateFilter::ObjectTypeWithOwner { object_type, owner } => {
                let object_type_str = format_struct_tag(object_type);
                format!(
                    "{STATE_OBJECT_TYPE_STR} = \"{}\" AND {STATE_OWNER_STR} = \"{}\"",
                    object_type_str,
                    owner.to_hex_literal()
                )
            }
            GlobalStateFilter::ObjectType(object_type) => {
                let object_type_str = format_struct_tag(object_type);
                format!("{STATE_OBJECT_TYPE_STR} = \"{}\"", object_type_str)
            }
            GlobalStateFilter::Owner(owner) => {
                format!("{STATE_OWNER_STR} = \"{}\"", owner.to_hex_literal())
            }
            GlobalStateFilter::ObjectId(object_id) => {
                format!("{OBJECT_ID_STR} = \"{}\"", object_id)
            }
        };

        let cursor_clause = if descending_order {
            format!(
                "AND ({TX_ORDER_STR} < {} OR ({TX_ORDER_STR} = {} AND {STATE_INDEX_STR} < {}))",
                tx_order, tx_order, state_index
            )
        } else {
            format!(
                "AND ({TX_ORDER_STR} > {} OR ({TX_ORDER_STR} = {} AND {STATE_INDEX_STR} > {}))",
                tx_order, tx_order, state_index
            )
        };
        let order_clause = if descending_order {
            format!("{TX_ORDER_STR} DESC, {STATE_INDEX_STR} DESC")
        } else {
            format!("{TX_ORDER_STR} ASC, {STATE_INDEX_STR} ASC")
        };

        let query = format!(
            "
                SELECT * FROM global_states \
                WHERE {} {} \
                ORDER BY {} \
                LIMIT {}
            ",
            main_where_clause, cursor_clause, order_clause, limit,
        );

        tracing::debug!("query global states: {}", query);
        let stored_states = self
            .inner_indexer_reader
            .run_query(|conn| diesel::sql_query(query).load::<StoredGlobalState>(conn))?;

        let result = stored_states
            .into_iter()
            .map(|v| v.try_into_indexer_global_state())
            .collect::<Result<Vec<_>>>()
            .map_err(|e| {
                IndexerError::SQLiteReadError(format!("Cast indexer global states failed: {:?}", e))
            })?;

        Ok(result)
    }

    pub fn query_table_states_with_filter(
        &self,
        filter: TableStateFilter,
        cursor: Option<IndexerStateID>,
        limit: usize,
        descending_order: bool,
    ) -> IndexerResult<Vec<IndexerTableState>> {
        let (tx_order, state_index) = if let Some(cursor) = cursor {
            let IndexerStateID {
                tx_order,
                state_index,
            } = cursor;
            (tx_order as i64, state_index as i64)
        } else if descending_order {
            let (max_tx_order, state_index): (i64, i64) =
                self.inner_indexer_reader.run_query(|conn| {
                    table_states::dsl::table_states
                        .select((table_states::tx_order, table_states::state_index))
                        .order_by((
                            table_states::tx_order.desc(),
                            table_states::state_index.desc(),
                        ))
                        .first::<(i64, i64)>(conn)
                })?;
            (max_tx_order + 1, state_index)
        } else {
            (-1, 0)
        };

        let main_where_clause = match filter {
            TableStateFilter::TableHandle(table_handle) => {
                format!("{STATE_TABLE_HANDLE_STR} = \"{}\"", table_handle)
            }
        };

        let cursor_clause = if descending_order {
            format!(
                "AND ({TX_ORDER_STR} < {} OR ({TX_ORDER_STR} = {} AND {STATE_INDEX_STR} < {}))",
                tx_order, tx_order, state_index
            )
        } else {
            format!(
                "AND ({TX_ORDER_STR} > {} OR ({TX_ORDER_STR} = {} AND {STATE_INDEX_STR} > {}))",
                tx_order, tx_order, state_index
            )
        };
        let order_clause = if descending_order {
            format!("{TX_ORDER_STR} DESC, {STATE_INDEX_STR} DESC")
        } else {
            format!("{TX_ORDER_STR} ASC, {STATE_INDEX_STR} ASC")
        };

        let query = format!(
            "
                SELECT * FROM table_states \
                WHERE {} {} \
                ORDER BY {} \
                LIMIT {}
            ",
            main_where_clause, cursor_clause, order_clause, limit,
        );

        tracing::debug!("query table states: {}", query);
        let stored_states = self
            .inner_indexer_reader
            .run_query(|conn| diesel::sql_query(query).load::<StoredTableState>(conn))?;

        let result = stored_states
            .into_iter()
            .map(|v| v.try_into_indexer_table_state())
            .collect::<Result<Vec<_>>>()
            .map_err(|e| {
                IndexerError::SQLiteReadError(format!("Cast indexer table states failed: {:?}", e))
            })?;

        Ok(result)
    }

    pub fn sync_states(
        &self,
        filter: Option<StateSyncFilter>,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerStateID>,
        limit: usize,
        descending_order: bool,
    ) -> IndexerResult<Vec<IndexerTableChangeSet>> {
        let (tx_order, state_index) = if let Some(cursor) = cursor {
            let IndexerStateID {
                tx_order,
                state_index,
            } = cursor;
            (tx_order as i64, state_index as i64)
        } else if descending_order {
            let (max_tx_order, state_index): (i64, i64) =
                self.inner_indexer_reader.run_query(|conn| {
                    table_change_sets::dsl::table_change_sets
                        .select((table_change_sets::tx_order, table_change_sets::state_index))
                        .order_by((
                            table_change_sets::tx_order.desc(),
                            table_change_sets::state_index.desc(),
                        ))
                        .first::<(i64, i64)>(conn)
                })?;
            (max_tx_order + 1, state_index)
        } else {
            (-1, 0)
        };

        let main_where_clause_opt = filter.map(|f| match f {
            StateSyncFilter::TableHandle(table_handle) => {
                format!("{STATE_TABLE_HANDLE_STR} = \"{}\"", table_handle)
            }
        });
        let cursor_clause = if descending_order {
            format!(
                " ({TX_ORDER_STR} < {} OR ({TX_ORDER_STR} = {} AND {STATE_INDEX_STR} < {}))",
                tx_order, tx_order, state_index
            )
        } else {
            format!(
                " ({TX_ORDER_STR} > {} OR ({TX_ORDER_STR} = {} AND {STATE_INDEX_STR} > {}))",
                tx_order, tx_order, state_index
            )
        };
        let where_clause = match main_where_clause_opt {
            Some(main_where_clause) => format!(" {} AND {} ", main_where_clause, cursor_clause),
            None => format!(" {} ", cursor_clause),
        };

        let order_clause = if descending_order {
            format!("{TX_ORDER_STR} DESC, {STATE_INDEX_STR} DESC")
        } else {
            format!("{TX_ORDER_STR} ASC, {STATE_INDEX_STR} ASC")
        };

        let query = format!(
            "
                SELECT * FROM table_change_sets \
                WHERE {} \
                ORDER BY {} \
                LIMIT {}
            ",
            where_clause, order_clause, limit,
        );

        tracing::debug!("sync states: {}", query);
        let stored_table_change_sets = self
            .inner_indexer_reader
            .run_query(|conn| diesel::sql_query(query).load::<StoredTableChangeSet>(conn))?;

        let result = stored_table_change_sets
            .into_iter()
            .map(|t| t.try_into_indexer_state_change_set())
            .collect::<Result<Vec<_>>>()
            .map_err(|e| {
                IndexerError::SQLiteReadError(format!(
                    "Cast indexer table change sets failed: {:?}",
                    e
                ))
            })?;

        Ok(result)
    }
}
