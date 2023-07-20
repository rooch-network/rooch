// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Error;
// use serde::{Deserialize, Serialize};
// use thiserror::Error;

// #[derive(Debug, Error)]
// pub enum RawStoreError {
//     #[error("Store check error {0:?}.")]
//     StoreCheckError(Error),
// }

// #[derive(Error, Debug, Serialize, Deserialize)]
#[derive(thiserror::Error, Debug)]
pub enum RawStoreError {
    #[error("Store check error {0:?}.")]
    StoreCheckError(Error),
    // #[error("rocksdb error: {0}")]
    // RocksDBError(String),
    // #[error("(de)serialization error: {0}")]
    // SerializationError(String),
    // #[error("the column family {0} was not registered with the database")]
    // UnregisteredColumn(String),
    // #[error("a batch operation can't operate across databases")]
    // CrossDBBatch,
    // #[error("Metric reporting thread failed with error")]
    // MetricsReporting,
    // #[error("Transaction should be retried")]
    // RetryableTransactionError,
}
