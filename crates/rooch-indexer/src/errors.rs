// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

#[derive(Debug, Error)]
pub enum IndexerError {
    #[error("Indexer failed to convert timestamp to NaiveDateTime with error: `{0}`")]
    DateTimeParsingError(String),

    #[error("Indexer failed to deserialize event from events table with error: `{0}`")]
    EventDeserializationError(String),

    #[error("Indexer failed to transform data with error: `{0}`")]
    DataTransformationError(String),

    #[error("Indexer failed to convert structs to diesel Insertable with error: `{0}`")]
    InsertableParsingError(String),

    #[error("Indexer failed to build SQLite connection pool with error: `{0}`")]
    SqliteConnectionPoolInitError(String),

    #[error(
        "Indexer failed to get a pool connection from SQLite connection pool with error: `{0}`"
    )]
    SqlitePoolConnectionError(String),

    #[error("Indexer failed to read SQLiteDB with error: `{0}`")]
    SQLiteReadError(String),

    #[error("Indexer failed to reset SQLiteDB with error: `{0}`")]
    SQLiteResetError(String),

    #[error("Indexer failed to write SQLiteDB with error: `{0}`")]
    SQLiteWriteError(String),

    #[error(transparent)]
    SQLiteError(#[from] diesel::result::Error),

    #[error("Indexer failed to serialize/deserialize with error: `{0}`")]
    SerdeError(String),

    #[error("Indexer does not support the feature with error: `{0}`")]
    NotSupportedError(String),

    #[error("Indexer read corrupted/incompatible data from persistent storage: `{0}`")]
    PersistentStorageDataCorruptionError(String),

    #[error("Indexer generic error: `{0}`")]
    GenericError(String),

    #[error(transparent)]
    UncategorizedError(#[from] anyhow::Error),

    #[error("Invalid transaction digest with error: `{0}`")]
    InvalidTransactionDigestError(String),

    #[error(transparent)]
    BcsError(#[from] bcs::Error),

    #[error("Invalid argument with error: `{0}`")]
    InvalidArgumentError(String),

    #[error("`{0}`: `{1}`")]
    ErrorWithContext(String, Box<IndexerError>),
}

pub trait Context<T> {
    fn context(self, context: &str) -> Result<T, IndexerError>;
}

impl<T> Context<T> for Result<T, IndexerError> {
    fn context(self, context: &str) -> Result<T, IndexerError> {
        self.map_err(|e| IndexerError::ErrorWithContext(context.to_string(), Box::new(e)))
    }
}

impl From<tokio::task::JoinError> for IndexerError {
    fn from(value: tokio::task::JoinError) -> Self {
        IndexerError::UncategorizedError(anyhow::Error::from(value))
    }
}
