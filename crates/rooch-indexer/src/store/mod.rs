// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub use traits::*;

pub mod indexer_store;
pub mod sqlite_store;
pub mod traits;

pub(crate) mod diesel_macro {
    macro_rules! read_only_blocking {
        ($pool:expr, $query:expr) => {{
            let mut sqlite_pool_conn = crate::get_sqlite_pool_connection($pool)?;
            sqlite_pool_conn
                .build_transaction()
                .read_only()
                .run($query)
                .map_err(|e| IndexerError::SQLiteReadError(e.to_string()))
        }};
    }

    macro_rules! transactional_blocking {
        ($pool:expr, $query:expr) => {{
            let mut sqlite_pool_conn = crate::get_sqlite_pool_connection($pool)?;
            sqlite_pool_conn
                .build_transaction()
                .serializable()
                .read_write()
                .run($query)
                .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
        }};
    }

    macro_rules! transactional_blocking_with_retry {
        ($pool:expr, $query:expr, $max_elapsed:expr) => {{
            let mut backoff = backoff::ExponentialBackoff::default();
            backoff.max_elapsed_time = Some($max_elapsed);

            let result = match backoff::retry(backoff, || {
                // let mut sqlite_pool_conn =
                //     crate::get_sqlite_pool_connection($pool).map_err(|e| {
                //         backoff::Error::Transient {
                //             err: IndexerError::SQLiteWriteError(e.to_string()),
                //             retry_after: None,
                //         }
                //     })?;
                // sqlite_pool_conn
                //     .build_transaction()
                //     .read_write()
                //     .run($query)
                //     .map_err(|e| {
                //         tracing::error!("Error with persisting data into DB: {:?}", e);
                //         backoff::Error::Transient {
                //             err: IndexerError::SQLiteWriteError(e.to_string()),
                //             retry_after: None,
                //         }
                //     })
                //TODO
                Err(backoff::Error::Permanent(IndexerError::SQLiteWriteError(
                    "TODO".to_string(),
                )))
            }) {
                Ok(v) => Ok(v),
                Err(backoff::Error::Transient { err, .. }) => Err(err),
                Err(backoff::Error::Permanent(err)) => Err(err),
            };

            result
        }};
    }

    pub(crate) use read_only_blocking;
    pub(crate) use transactional_blocking;
    pub(crate) use transactional_blocking_with_retry;
}
