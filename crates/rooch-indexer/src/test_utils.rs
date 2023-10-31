// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::anyhow;
use tokio::task::JoinHandle;

use crate::errors::IndexerError;
use crate::store::SqliteIndexerStore;
use crate::utils::reset_database;
use crate::{new_sqlite_connection_pool, Indexer, IndexerConfig};

/// Spawns an indexer thread with provided SQLite DB url
pub async fn start_test_indexer(
    config: IndexerConfig,
) -> Result<(SqliteIndexerStore, JoinHandle<Result<(), IndexerError>>), anyhow::Error> {
    let parsed_url = config.base_connection_url()?;
    let blocking_pool = new_sqlite_connection_pool(&parsed_url)
        .map_err(|e| anyhow!("unable to connect to SQLite, is it running? {e}"))?;
    if config.reset_db {
        reset_database(
            &mut blocking_pool
                .get()
                .map_err(|e| anyhow!("Fail to get sqlite_connection_pool {e}"))?,
            true,
        )?;
    }

    let store = SqliteIndexerStore::new(blocking_pool);
    let store_clone = store.clone();
    let handle = tokio::spawn(async move { Indexer::start(&config, store_clone, None).await });
    Ok((store, handle))
}
