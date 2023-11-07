// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::anyhow;
use tokio::task::JoinHandle;

use crate::errors::IndexerError;
use crate::store::sqlite_store::SqliteIndexerStore;
use crate::{new_sqlite_connection_pool, Indexer, IndexerConfig};

/// Spawns an indexer thread with provided SQLite DB url
pub async fn start_test_indexer(
    config: IndexerConfig,
) -> Result<(SqliteIndexerStore, JoinHandle<Result<(), IndexerError>>), anyhow::Error> {
    let indexer_db = config
        .get_indexer_db()
        .to_str()
        .ok_or_else(|| anyhow!("Indexer_db doest not exist"))?
        .to_string();
    let blocking_pool = new_sqlite_connection_pool(indexer_db.as_str())
        .map_err(|e| anyhow!("Unable to connect to SQLite, is it running? {e}"))?;

    let store = SqliteIndexerStore::new(blocking_pool);
    let store_clone = store.clone();
    let handle = tokio::spawn(async move { Indexer::start(&config, store_clone).await });
    Ok((store, handle))
}
