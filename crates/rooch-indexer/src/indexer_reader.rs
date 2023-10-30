// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::IndexerError,
    models::{
        checkpoints::StoredCheckpoint,
        display::StoredDisplay,
        epoch::StoredEpochInfo,
        events::StoredEvent,
        objects::{CoinBalance, ObjectRefColumn, StoredObject},
        packages::StoredPackage,
        transactions::StoredTransaction,
        tx_indices::TxSequenceNumber,
    },
    schema::{checkpoints, display, epochs, events, objects, packages, transactions},
    types::{IndexerResult, OwnerType},
    SqliteConnectionConfig, SqliteConnectionPoolConfig, SqlitePoolConnection,
};
use anyhow::{anyhow, Result};
use diesel::{
    r2d2::ConnectionManager, ExpressionMethods, OptionalExtension, SqliteConnection, QueryDsl,
    RunQueryDsl,
};
use fastcrypto::encoding::Encoding;
use fastcrypto::encoding::Hex;
use itertools::{any, Itertools};
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, RwLock},
};
use sui_json_rpc_types::{Balance, Coin as SuiCoin};
use sui_json_rpc_types::{
    CheckpointId, EpochInfo, EventFilter, SuiEvent, SuiTransactionBlockResponse, TransactionFilter,
};
use sui_types::{
    base_types::{ObjectID, ObjectRef, SequenceNumber, SuiAddress, VersionNumber},
    committee::EpochId,
    digests::{ObjectDigest, TransactionDigest},
    dynamic_field::DynamicFieldInfo,
    move_package::MovePackage,
    object::{Object, ObjectRead},
    sui_system_state::{sui_system_state_summary::SuiSystemStateSummary, SuiSystemStateTrait},
};
use sui_types::{dynamic_field::DynamicFieldName, event::EventID};

pub const TX_SEQUENCE_NUMBER_STR: &str = "tx_sequence_number";
pub const TRANSACTION_DIGEST_STR: &str = "transaction_digest";
pub const EVENT_SEQUENCE_NUMBER_STR: &str = "event_sequence_number";

#[derive(Clone)]
pub struct IndexerReader {
    pool: crate::SqliteConnectionPool,
    package_cache: PackageCache,
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

        Ok(Self {
            pool,
            package_cache: Default::default(),
        })
    }

    fn get_connection(&self) -> Result<SqlitePoolConnection, IndexerError> {
        self.pool.get().map_err(|e| {
            IndexerError::SqlitePoolConnectionError(format!(
                "Failed to get connection from PG connection pool with error: {:?}",
                e
            ))
        })
    }

    pub fn run_query<T, E, F>(&self, query: F) -> Result<T, IndexerError>
    where
        F: FnOnce(&mut SqliteConnection) -> Result<T, E>,
        E: From<diesel::result::Error> + std::error::Error,
    {
        blocking_call_is_ok_or_panic();

        let mut connection = self.get_connection()?;
        connection
            .build_transaction()
            .read_only()
            .run(query)
            .map_err(|e| IndexerError::PostgresReadError(e.to_string()))
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
        digests: &[TransactionDigest],
    ) -> Result<Vec<StoredTransaction>, IndexerError> {
        let digests = digests
            .iter()
            .map(|digest| digest.inner().to_vec())
            .collect::<Vec<_>>();
        self.run_query(|conn| {
            transactions::table
                .filter(transactions::transaction_digest.eq_any(digests))
                .load::<StoredTransaction>(conn)
        })
    }

    fn stored_transaction_to_transaction_block(
        &self,
        stored_txes: Vec<StoredTransaction>,
        options: sui_json_rpc_types::SuiTransactionBlockResponseOptions,
    ) -> IndexerResult<Vec<SuiTransactionBlockResponse>> {
        stored_txes
            .into_iter()
            .map(|stored_tx| stored_tx.try_into_sui_transaction_block_response(&options, self))
            .collect::<IndexerResult<Vec<_>>>()
    }

    fn multi_get_transactions_with_sequence_numbers(
        &self,
        tx_sequence_numbers: Vec<i64>,
        // Some(true) for desc, Some(false) for asc, None for undefined order
        is_descending: Option<bool>,
    ) -> Result<Vec<StoredTransaction>, IndexerError> {
        let mut query = transactions::table
            .filter(transactions::tx_sequence_number.eq_any(tx_sequence_numbers))
            .into_boxed();
        match is_descending {
            Some(true) => {
                query = query.order(transactions::dsl::tx_sequence_number.desc());
            }
            Some(false) => {
                query = query.order(transactions::dsl::tx_sequence_number.asc());
            }
            None => (),
        }
        self.run_query(|conn| query.load::<StoredTransaction>(conn))
    }

    pub async fn query_events_in_blocking_task(
        &self,
        filter: EventFilter,
        cursor: Option<EventID>,
        limit: usize,
        descending_order: bool,
    ) -> IndexerResult<Vec<SuiEvent>> {
        self.spawn_blocking(move |this| {
            this.query_events_impl(filter, cursor, limit, descending_order)
        })
        .await
    }


    fn query_transaction_blocks_by_checkpoint_impl(
        &self,
        checkpoint_seq: u64,
        options: sui_json_rpc_types::SuiTransactionBlockResponseOptions,
        cursor_tx_seq: Option<i64>,
        limit: usize,
        is_descending: bool,
    ) -> IndexerResult<Vec<SuiTransactionBlockResponse>> {
        let mut query = transactions::dsl::transactions
            .filter(transactions::dsl::checkpoint_sequence_number.eq(checkpoint_seq as i64))
            .into_boxed();

        // Translate transaction digest cursor to tx sequence number
        if let Some(cursor_tx_seq) = cursor_tx_seq {
            if is_descending {
                query = query.filter(transactions::dsl::tx_sequence_number.le(cursor_tx_seq));
            } else {
                query = query.filter(transactions::dsl::tx_sequence_number.ge(cursor_tx_seq));
            }
        }
        if is_descending {
            query = query.order(transactions::dsl::tx_sequence_number.desc());
        } else {
            query = query.order(transactions::dsl::tx_sequence_number.asc());
        }

        let stored_txes =
            self.run_query(|conn| query.limit((limit) as i64).load::<StoredTransaction>(conn))?;

        self.stored_transaction_to_transaction_block(stored_txes, options)
    }

    pub async fn query_transaction_blocks_in_blocking_task(
        &self,
        filter: Option<TransactionFilter>,
        options: sui_json_rpc_types::SuiTransactionBlockResponseOptions,
        cursor: Option<TransactionDigest>,
        limit: usize,
        is_descending: bool,
    ) -> IndexerResult<Vec<SuiTransactionBlockResponse>> {
        self.spawn_blocking(move |this| {
            this.query_transaction_blocks_impl(filter, options, cursor, limit, is_descending)
        })
        .await
    }

    fn query_transaction_blocks_impl(
        &self,
        filter: Option<TransactionFilter>,
        options: sui_json_rpc_types::SuiTransactionBlockResponseOptions,
        cursor: Option<TransactionDigest>,
        limit: usize,
        is_descending: bool,
    ) -> IndexerResult<Vec<SuiTransactionBlockResponse>> {
        let cursor_tx_seq = if let Some(cursor) = cursor {
            Some(self.run_query(|conn| {
                transactions::dsl::transactions
                    .select(transactions::tx_sequence_number)
                    .filter(transactions::dsl::transaction_digest.eq(cursor.into_inner().to_vec()))
                    .first::<i64>(conn)
            })?)
        } else {
            None
        };
        let cursor_clause = if let Some(cursor_tx_seq) = cursor_tx_seq {
            if is_descending {
                format!("AND {TX_SEQUENCE_NUMBER_STR} < {}", cursor_tx_seq)
            } else {
                format!("AND {TX_SEQUENCE_NUMBER_STR} > {}", cursor_tx_seq)
            }
        } else {
            "".to_string()
        };
        let order_str = if is_descending { "DESC" } else { "ASC" };
        let (table_name, main_where_clause) = match filter {
            // Processed above
            Some(TransactionFilter::Checkpoint(seq)) => {
                return self.query_transaction_blocks_by_checkpoint_impl(
                    seq,
                    options,
                    cursor_tx_seq,
                    limit,
                    is_descending,
                )
            }
            // FIXME: sanitize module & function
            Some(TransactionFilter::MoveFunction {
                package,
                module,
                function,
            }) => {
                let package = Hex::encode(package.to_vec());
                match (module, function) {
                    (Some(module), Some(function)) => (
                        "tx_calls".into(),
                        format!(
                            "package = '\\x{}'::bytea AND module = '{}' AND func = '{}'",
                            package, module, function
                        ),
                    ),
                    (Some(module), None) => (
                        "tx_calls".into(),
                        format!(
                            "package = '\\x{}'::bytea AND module = '{}'",
                            package, module
                        ),
                    ),
                    (None, Some(_)) => {
                        return Err(IndexerError::InvalidArgumentError(
                            "Function cannot be present wihtout Module.".into(),
                        ));
                    }
                    (None, None) => (
                        "tx_calls".into(),
                        format!("package = '\\x{}'::bytea", package),
                    ),
                }
            }
            Some(TransactionFilter::InputObject(object_id)) => {
                let object_id = Hex::encode(object_id.to_vec());
                (
                    "tx_input_objects".into(),
                    format!("object_id = '\\x{}'::bytea", object_id),
                )
            }
            Some(TransactionFilter::ChangedObject(object_id)) => {
                let object_id = Hex::encode(object_id.to_vec());
                (
                    "tx_changed_objects".into(),
                    format!("object_id = '\\x{}'::bytea", object_id),
                )
            }
            Some(TransactionFilter::FromAddress(from_address)) => {
                let from_address = Hex::encode(from_address.to_vec());
                (
                    "tx_senders".into(),
                    format!("sender = '\\x{}'::bytea", from_address),
                )
            }
            Some(TransactionFilter::ToAddress(to_address)) => {
                let to_address = Hex::encode(to_address.to_vec());
                (
                    "tx_recipients".into(),
                    format!("recipient = '\\x{}'::bytea", to_address),
                )
            }
            Some(TransactionFilter::FromAndToAddress { from, to }) => {
                let from_address = Hex::encode(from.to_vec());
                let to_address = Hex::encode(to.to_vec());
                // Need to remove ambiguities for tx_sequence_number column
                let cursor_clause = if let Some(cursor_tx_seq) = cursor_tx_seq {
                    if is_descending {
                        format!(
                            "AND tx_senders.{TX_SEQUENCE_NUMBER_STR} < {}",
                            cursor_tx_seq
                        )
                    } else {
                        format!(
                            "AND tx_senders.{TX_SEQUENCE_NUMBER_STR} > {}",
                            cursor_tx_seq
                        )
                    }
                } else {
                    "".to_string()
                };
                let inner_query = format!(
                    "(SELECT tx_senders.{TX_SEQUENCE_NUMBER_STR} \
                    FROM tx_senders \
                    JOIN tx_recipients \
                    ON tx_senders.{TX_SEQUENCE_NUMBER_STR} = tx_recipients.{TX_SEQUENCE_NUMBER_STR} \
                    WHERE tx_senders.sender = '\\x{}'::BYTEA \
                    AND tx_recipients.recipient = '\\x{}'::BYTEA \
                    {} \
                    ORDER BY {TX_SEQUENCE_NUMBER_STR} {} \
                    LIMIT {}) AS inner_query
                    ",
                    from_address,
                    to_address,
                    cursor_clause,
                    order_str,
                    limit,
                );
                (inner_query, "1 = 1".into())
            }
            Some(TransactionFilter::FromOrToAddress { addr }) => {
                let address = Hex::encode(addr.to_vec());
                let inner_query = format!(
                    "( \
                        ( \
                            SELECT {TX_SEQUENCE_NUMBER_STR} FROM tx_senders \
                            WHERE sender = '\\x{}'::BYTEA {} \
                            ORDER BY {TX_SEQUENCE_NUMBER_STR} {} \
                            LIMIT {} \
                        ) \
                        UNION \
                        ( \
                            SELECT {TX_SEQUENCE_NUMBER_STR} FROM tx_recipients \
                            WHERE recipient = '\\x{}'::BYTEA {} \
                            ORDER BY {TX_SEQUENCE_NUMBER_STR} {} \
                            LIMIT {} \
                        ) \
                    ) AS combined",
                    address,
                    cursor_clause,
                    order_str,
                    limit,
                    address,
                    cursor_clause,
                    order_str,
                    limit,
                );
                (inner_query, "1 = 1".into())
            }
            Some(
                TransactionFilter::TransactionKind(_) | TransactionFilter::TransactionKindIn(_),
            ) => {
                return Err(IndexerError::NotSupportedError(
                    "TransactionKind filter is not supported.".into(),
                ));
            }
            None => {
                // apply no filter
                ("transactions".into(), "1 = 1".into())
            }
        };

        let query = format!(
            "SELECT {TX_SEQUENCE_NUMBER_STR} FROM {} WHERE {} {} ORDER BY {TX_SEQUENCE_NUMBER_STR} {} LIMIT {}",
            table_name,
            main_where_clause,
            cursor_clause,
            order_str,
            limit,
        );

        tracing::debug!("query transaction blocks: {}", query);

        let tx_sequence_numbers = self
            .run_query(|conn| diesel::sql_query(query.clone()).load::<TxSequenceNumber>(conn))?
            .into_iter()
            .map(|tsn| tsn.tx_sequence_number)
            .collect::<Vec<_>>();

        self.multi_get_transaction_block_response_by_sequence_numbers(
            tx_sequence_numbers,
            options,
            Some(is_descending),
        )
    }

    fn multi_get_transaction_block_response_impl(
        &self,
        digests: &[TransactionDigest],
        options: sui_json_rpc_types::SuiTransactionBlockResponseOptions,
    ) -> Result<Vec<sui_json_rpc_types::SuiTransactionBlockResponse>, IndexerError> {
        let stored_txes = self.multi_get_transactions(digests)?;
        self.stored_transaction_to_transaction_block(stored_txes, options)
    }

    fn multi_get_transaction_block_response_by_sequence_numbers(
        &self,
        tx_sequence_numbers: Vec<i64>,
        options: sui_json_rpc_types::SuiTransactionBlockResponseOptions,
        // Some(true) for desc, Some(false) for asc, None for undefined order
        is_descending: Option<bool>,
    ) -> Result<Vec<sui_json_rpc_types::SuiTransactionBlockResponse>, IndexerError> {
        let stored_txes: Vec<StoredTransaction> =
            self.multi_get_transactions_with_sequence_numbers(tx_sequence_numbers, is_descending)?;
        self.stored_transaction_to_transaction_block(stored_txes, options)
    }

    pub async fn multi_get_transaction_block_response_in_blocking_task(
        &self,
        digests: Vec<TransactionDigest>,
        options: sui_json_rpc_types::SuiTransactionBlockResponseOptions,
    ) -> Result<Vec<sui_json_rpc_types::SuiTransactionBlockResponse>, IndexerError> {
        self.spawn_blocking(move |this| {
            this.multi_get_transaction_block_response_impl(&digests, options)
        })
        .await
    }

    fn get_transaction_events_impl(
        &self,
        digest: TransactionDigest,
    ) -> Result<Vec<sui_json_rpc_types::SuiEvent>, IndexerError> {
        let (timestamp_ms, serialized_events) = self.run_query(|conn| {
            transactions::table
                .filter(transactions::transaction_digest.eq(digest.into_inner().to_vec()))
                .select((transactions::timestamp_ms, transactions::events))
                .first::<(i64, Vec<Option<Vec<u8>>>)>(conn)
        })?;

        let events = serialized_events
            .into_iter()
            .flatten()
            .map(|event| bcs::from_bytes::<sui_types::event::Event>(&event))
            .collect::<Result<Vec<_>, _>>()?;

        events
            .into_iter()
            .enumerate()
            .map(|(i, event)| {
                sui_json_rpc_types::SuiEvent::try_from(
                    event,
                    digest,
                    i as u64,
                    Some(timestamp_ms as u64),
                    self,
                )
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    fn query_events_by_tx_digest_query(
        &self,
        tx_digest: TransactionDigest,
        cursor: Option<EventID>,
        limit: usize,
        descending_order: bool,
    ) -> IndexerResult<String> {
        let cursor = if let Some(cursor) = cursor {
            if cursor.tx_digest != tx_digest {
                return Err(IndexerError::InvalidArgumentError(
                    "Cursor tx_digest does not match the tx_digest in the query.".into(),
                ));
            }
            if descending_order {
                format!("e.{EVENT_SEQUENCE_NUMBER_STR} < {}", cursor.event_seq)
            } else {
                format!("e.{EVENT_SEQUENCE_NUMBER_STR} > {}", cursor.event_seq)
            }
        } else if descending_order {
            format!("e.{EVENT_SEQUENCE_NUMBER_STR} <= {}", i64::MAX)
        } else {
            format!("e.{EVENT_SEQUENCE_NUMBER_STR} >= {}", 0)
        };

        let order_clause = if descending_order { "DESC" } else { "ASC" };
        Ok(format!(
            "SELECT * \
            FROM EVENTS e \
            JOIN TRANSACTIONS t \
            ON t.tx_sequence_number = e.tx_sequence_number \
            AND t.transaction_digest = '\\x{}'::bytea \
            WHERE {cursor} \
            ORDER BY e.{EVENT_SEQUENCE_NUMBER_STR} {order_clause} \
            LIMIT {limit}
            ",
            Hex::encode(tx_digest.into_inner()),
        ))
    }

    fn query_events_impl(
        &self,
        filter: EventFilter,
        cursor: Option<EventID>,
        limit: usize,
        descending_order: bool,
    ) -> IndexerResult<Vec<SuiEvent>> {
        let (tx_seq, event_seq) = if let Some(cursor) = cursor.clone() {
            let EventID {
                tx_digest,
                event_seq,
            } = cursor;
            (
                self.run_query(|conn| {
                    transactions::dsl::transactions
                        .select(transactions::tx_sequence_number)
                        .filter(
                            transactions::dsl::transaction_digest
                                .eq(tx_digest.into_inner().to_vec()),
                        )
                        .first::<i64>(conn)
                })?,
                event_seq,
            )
        } else if descending_order {
            let max_tx_seq: i64 = self.run_query(|conn| {
                events::dsl::events
                    .select(events::tx_sequence_number)
                    .order(events::dsl::tx_sequence_number.desc())
                    .first::<i64>(conn)
            })?;
            (max_tx_seq + 1, 0)
        } else {
            (-1, 0)
        };

        let query = if let EventFilter::Sender(sender) = &filter {
            // Need to remove ambiguities for tx_sequence_number column
            let cursor_clause = if descending_order {
                format!("(e.{TX_SEQUENCE_NUMBER_STR} < {} OR (e.{TX_SEQUENCE_NUMBER_STR} = {} AND e.{EVENT_SEQUENCE_NUMBER_STR} < {}))", tx_seq, tx_seq, event_seq)
            } else {
                format!("(e.{TX_SEQUENCE_NUMBER_STR} > {} OR (e.{TX_SEQUENCE_NUMBER_STR} = {} AND e.{EVENT_SEQUENCE_NUMBER_STR} > {}))", tx_seq, tx_seq, event_seq)
            };
            let order_clause = if descending_order {
                format!("e.{TX_SEQUENCE_NUMBER_STR} DESC, e.{EVENT_SEQUENCE_NUMBER_STR} DESC")
            } else {
                format!("e.{TX_SEQUENCE_NUMBER_STR} ASC, e.{EVENT_SEQUENCE_NUMBER_STR} ASC")
            };
            format!(
                "( \
                    SELECT *
                    FROM tx_senders s
                    JOIN events e
                    ON e.tx_sequence_number = s.tx_sequence_number
                    AND s.sender = '\\x{}'::bytea
                    WHERE {} \
                    ORDER BY {} \
                    LIMIT {}
                )",
                Hex::encode(sender.to_vec()),
                cursor_clause,
                order_clause,
                limit,
            )
        } else if let EventFilter::Transaction(tx_digest) = filter {
            self.query_events_by_tx_digest_query(tx_digest, cursor, limit, descending_order)?
        } else {
            let main_where_clause = match filter {
                EventFilter::Package(package_id) => {
                    format!("package = '\\x{}'::bytea", package_id.to_hex())
                }
                EventFilter::MoveModule { package, module } => {
                    format!(
                        "package = '\\x{}'::bytea AND module = '{}'",
                        package.to_hex(),
                        module,
                    )
                }
                EventFilter::MoveEventType(struct_tag) => {
                    format!("event_type = '{}'", struct_tag)
                }
                EventFilter::MoveEventModule { package, module } => {
                    let package_module_prefix = format!("{}::{}", package.to_hex_literal(), module);
                    format!("event_type LIKE '{package_module_prefix}::%'")
                }
                EventFilter::Sender(_) => {
                    // Processed above
                    unreachable!()
                }
                EventFilter::Transaction(_) => {
                    // Processed above
                    unreachable!()
                }
                EventFilter::MoveEventField { .. }
                | EventFilter::All(_)
                | EventFilter::Any(_)
                | EventFilter::And(_, _)
                | EventFilter::Or(_, _)
                | EventFilter::TimeRange { .. } => {
                    return Err(IndexerError::NotSupportedError(
                        "This type of EventFilter is not supported.".into(),
                    ));
                }
            };

            let cursor_clause = if descending_order {
                format!("AND ({TX_SEQUENCE_NUMBER_STR} < {} OR ({TX_SEQUENCE_NUMBER_STR} = {} AND {EVENT_SEQUENCE_NUMBER_STR} < {}))", tx_seq, tx_seq, event_seq)
            } else {
                format!("AND ({TX_SEQUENCE_NUMBER_STR} > {} OR ({TX_SEQUENCE_NUMBER_STR} = {} AND {EVENT_SEQUENCE_NUMBER_STR} > {}))", tx_seq, tx_seq, event_seq)
            };
            let order_clause = if descending_order {
                format!("{TX_SEQUENCE_NUMBER_STR} DESC, {EVENT_SEQUENCE_NUMBER_STR} DESC")
            } else {
                format!("{TX_SEQUENCE_NUMBER_STR} ASC, {EVENT_SEQUENCE_NUMBER_STR} ASC")
            };

            format!(
                "
                    SELECT * FROM events \
                    WHERE {} {} \
                    ORDER BY {} \
                    LIMIT {}
                ",
                main_where_clause, cursor_clause, order_clause, limit,
            )
        };
        tracing::debug!("query events: {}", query);
        let stored_events =
            self.run_query(|conn| diesel::sql_query(query).load::<StoredEvent>(conn))?;
        stored_events
            .into_iter()
            .map(|se| se.try_into_sui_event(self))
            .collect()
    }

}
