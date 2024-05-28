// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::errors::IndexerError;
use crate::models::events::StoredEvent;
use crate::models::states::{StoredFieldState, StoredObjectState};
use crate::models::transactions::StoredTransaction;
use crate::schema::object_states;
use crate::schema::{events, field_states, transactions};
use crate::utils::format_struct_tag;
use crate::{
    IndexerResult, IndexerStoreMeta, SqliteConnectionConfig, SqliteConnectionPoolConfig,
    SqlitePoolConnection, INDEXER_EVENTS_TABLE_NAME, INDEXER_FIELD_STATES_TABLE_NAME,
    INDEXER_OBJECT_STATES_TABLE_NAME, INDEXER_TRANSACTIONS_TABLE_NAME,
};
use anyhow::{anyhow, Result};
use diesel::{
    r2d2::ConnectionManager, Connection, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection,
};
use move_core_types::language_storage::StructTag;
use rooch_types::indexer::event::{EventFilter, IndexerEvent, IndexerEventID};
use rooch_types::indexer::state::{
    FieldStateFilter, IndexerFieldState, IndexerObjectState, IndexerStateID, ObjectStateFilter,
};
use rooch_types::indexer::transaction::{IndexerTransaction, TransactionFilter};
use std::collections::HashMap;
use std::ops::DerefMut;
use std::path::PathBuf;

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

pub const STATE_OBJECT_ID_STR: &str = "object_id";
pub const STATE_INDEX_STR: &str = "state_index";
pub const STATE_OBJECT_TYPE_STR: &str = "object_type";
pub const STATE_OWNER_STR: &str = "owner";

#[derive(Clone)]
pub(crate) struct InnerIndexerReader {
    pool: crate::SqliteConnectionPool,
}

impl InnerIndexerReader {
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
    pub(crate) inner_indexer_reader_mapping: HashMap<String, InnerIndexerReader>,
}

impl IndexerReader {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let config = SqliteConnectionPoolConfig::default();
        Self::new_with_config(db_path, config)
    }

    pub fn new_with_config(db_path: PathBuf, config: SqliteConnectionPoolConfig) -> Result<Self> {
        let tables = IndexerStoreMeta::get_indexer_table_names().to_vec();

        let mut inner_indexer_reader_mapping = HashMap::<String, InnerIndexerReader>::new();
        for table in tables {
            let indexer_db_url = db_path
                .clone()
                .join(table)
                .to_str()
                .ok_or(anyhow::anyhow!("Invalid indexer db path"))?
                .to_string();

            let inner_indexer_reader = InnerIndexerReader::new_with_config(indexer_db_url, config)?;
            inner_indexer_reader_mapping.insert(table.to_string(), inner_indexer_reader);
        }

        Ok(IndexerReader {
            inner_indexer_reader_mapping,
        })
    }

    fn get_inner_indexer_reader(&self, table_name: &str) -> Result<InnerIndexerReader> {
        Ok(self
            .inner_indexer_reader_mapping
            .get(table_name)
            .ok_or(anyhow::anyhow!("Inner indexer reader not exist"))?
            .clone())
    }

    pub fn query_transactions_with_filter(
        &self,
        filter: TransactionFilter,
        cursor: Option<u64>,
        limit: usize,
        descending_order: bool,
    ) -> IndexerResult<Vec<IndexerTransaction>> {
        let tx_order = if let Some(cursor) = cursor {
            cursor as i64
        } else if descending_order {
            let max_tx_order: i64 = self
                .get_inner_indexer_reader(INDEXER_TRANSACTIONS_TABLE_NAME)?
                .run_query(|conn| {
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
            .get_inner_indexer_reader(INDEXER_TRANSACTIONS_TABLE_NAME)?
            .run_query(|conn| diesel::sql_query(query).load::<StoredTransaction>(conn))?;

        let result = stored_transactions
            .into_iter()
            .map(IndexerTransaction::try_from)
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
            let (max_tx_order, event_index): (i64, i64) = self
                .get_inner_indexer_reader(INDEXER_EVENTS_TABLE_NAME)?
                .run_query(|conn| {
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
            .get_inner_indexer_reader(INDEXER_EVENTS_TABLE_NAME)?
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

    pub fn query_object_states_with_filter(
        &self,
        filter: ObjectStateFilter,
        cursor: Option<IndexerStateID>,
        limit: usize,
        descending_order: bool,
    ) -> IndexerResult<Vec<IndexerObjectState>> {
        let (tx_order, state_index) = if let Some(cursor) = cursor {
            let IndexerStateID {
                tx_order,
                state_index,
            } = cursor;
            (tx_order as i64, state_index as i64)
        } else if descending_order {
            let (max_tx_order, state_index): (i64, i64) = self
                .get_inner_indexer_reader(INDEXER_OBJECT_STATES_TABLE_NAME)?
                .run_query(|conn| {
                    object_states::dsl::object_states
                        .select((object_states::tx_order, object_states::state_index))
                        .order_by((
                            object_states::tx_order.desc(),
                            object_states::state_index.desc(),
                        ))
                        .first::<(i64, i64)>(conn)
                })?;
            (max_tx_order + 1, state_index)
        } else {
            (-1, 0)
        };

        let main_where_clause = match filter {
            ObjectStateFilter::ObjectTypeWithOwner { object_type, owner } => {
                let object_query = object_type_query(&object_type);
                format!(
                    "{} AND {STATE_OWNER_STR} = \"{}\"",
                    object_query,
                    owner.to_hex_literal()
                )
            }
            ObjectStateFilter::ObjectType(object_type) => object_type_query(&object_type),
            ObjectStateFilter::Owner(owner) => {
                format!("{STATE_OWNER_STR} = \"{}\"", owner.to_hex_literal())
            }
            ObjectStateFilter::ObjectId(object_ids) => {
                let object_ids_str = object_ids
                    .into_iter()
                    .map(|obj_id| format!("\"{}\"", obj_id))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("{OBJECT_ID_STR} IN ({object_ids_str})")
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
                SELECT * FROM object_states \
                WHERE {} {} \
                ORDER BY {} \
                LIMIT {}
            ",
            main_where_clause, cursor_clause, order_clause, limit,
        );

        tracing::debug!("query global states: {}", query);
        let stored_states = self
            .get_inner_indexer_reader(INDEXER_OBJECT_STATES_TABLE_NAME)?
            .run_query(|conn| diesel::sql_query(query).load::<StoredObjectState>(conn))?;

        let result = stored_states
            .into_iter()
            .map(|v| v.try_into_indexer_global_state())
            .collect::<Result<Vec<_>>>()
            .map_err(|e| {
                IndexerError::SQLiteReadError(format!("Cast indexer global states failed: {:?}", e))
            })?;

        Ok(result)
    }

    pub fn query_field_states_with_filter(
        &self,
        filter: FieldStateFilter,
        cursor: Option<IndexerStateID>,
        limit: usize,
        descending_order: bool,
    ) -> IndexerResult<Vec<IndexerFieldState>> {
        let (tx_order, state_index) = if let Some(cursor) = cursor {
            let IndexerStateID {
                tx_order,
                state_index,
            } = cursor;
            (tx_order as i64, state_index as i64)
        } else if descending_order {
            let (max_tx_order, state_index): (i64, i64) = self
                .get_inner_indexer_reader(INDEXER_FIELD_STATES_TABLE_NAME)?
                .run_query(|conn| {
                    field_states::dsl::field_states
                        .select((field_states::tx_order, field_states::state_index))
                        .order_by((
                            field_states::tx_order.desc(),
                            field_states::state_index.desc(),
                        ))
                        .first::<(i64, i64)>(conn)
                })?;
            (max_tx_order + 1, state_index)
        } else {
            (-1, 0)
        };

        let main_where_clause = match filter {
            FieldStateFilter::ObjectId(object_id) => {
                format!("{STATE_OBJECT_ID_STR} = \"{}\"", object_id)
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
                SELECT * FROM field_states \
                WHERE {} {} \
                ORDER BY {} \
                LIMIT {}
            ",
            main_where_clause, cursor_clause, order_clause, limit,
        );

        tracing::debug!("query table states: {}", query);
        let stored_states = self
            .get_inner_indexer_reader(INDEXER_FIELD_STATES_TABLE_NAME)?
            .run_query(|conn| diesel::sql_query(query).load::<StoredFieldState>(conn))?;

        let result = stored_states
            .into_iter()
            .map(|v| v.try_into_indexer_table_state())
            .collect::<Result<Vec<_>>>()
            .map_err(|e| {
                IndexerError::SQLiteReadError(format!("Cast indexer table states failed: {:?}", e))
            })?;

        Ok(result)
    }
}

fn object_type_query(object_type: &StructTag) -> String {
    let object_type_str = format_struct_tag(object_type);
    // if the caller does not specify the type parameters, we will use the prefix match
    if object_type.type_params.is_empty() {
        format!("{STATE_OBJECT_TYPE_STR} like \"{}%\"", object_type_str)
    } else {
        format!("{STATE_OBJECT_TYPE_STR} = \"{}\"", object_type_str)
    }
}
