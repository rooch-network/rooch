// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::errors::{Context, IndexerError};
    use crate::indexer_reader::IndexerReader;
    use crate::models::events::StoredEvent;
    use crate::schema::events;
    use crate::{get_sqlite_pool_connection, IndexerStore, INDEXER_EVENTS_TABLE_NAME};
    use anyhow::Result;
    use diesel::RunQueryDsl;
    use move_core_types::account_address::AccountAddress;
    use moveos_types::moveos_std::tx_context::TxContext;
    use moveos_types::test_utils::random_event;
    use rooch_config::store_config::DEFAULT_DB_INDEXER_SUBDIR;
    use rooch_types::indexer::event::IndexerEvent;
    use rooch_types::test_utils::random_ledger_transaction;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_sqlite_writer_sqlite_reader_concurrence() {
        let count = sqlite_writer_sqlite_reader_concurrence().unwrap();
        assert_eq!(count, get_row_count());
    }

    fn get_row_count() -> i64 {
        1000
        // 10000
    }
    fn get_sleep_duration() -> Duration {
        Duration::from_millis(1)
    }

    /// Make tx_order using assigned tx_order and event_index to 0
    fn random_indexer_event(tx_order: u64) -> IndexerEvent {
        let mut random_event = random_event();
        random_event.event_index = 0;
        let mut random_transaction = random_ledger_transaction();
        random_transaction.sequence_info.tx_order = tx_order;
        let tx_context = TxContext::new_readonly_ctx(AccountAddress::random());

        IndexerEvent::new(random_event, random_transaction, tx_context)
    }

    fn sqlite_writer_sqlite_reader_concurrence() -> Result<i64> {
        let sleep_duration = get_sleep_duration();

        let tmpdir = moveos_config::temp_dir();
        let indexer_db = tmpdir.path().join(DEFAULT_DB_INDEXER_SUBDIR);
        let indexer_store = IndexerStore::new(indexer_db.clone())?;
        let indexer_reader = IndexerReader::new(indexer_db)?;

        let write_connection_pool = indexer_store
            .get_sqlite_store(INDEXER_EVENTS_TABLE_NAME)?
            .connection_pool;
        let inner_indexer_reader =
            indexer_reader.get_inner_indexer_reader(INDEXER_EVENTS_TABLE_NAME)?;

        // Spawn a thread to write to the DB
        let writer = thread::spawn(move || {
            let mut connection = get_sqlite_pool_connection(&write_connection_pool).unwrap();

            // Loop inserting rows
            for i in 0..get_row_count() {
                let events_data = vec![random_indexer_event(i as u64)];
                let events = events_data
                    .into_iter()
                    .map(StoredEvent::from)
                    .collect::<Vec<_>>();

                diesel::insert_into(events::table)
                    .values(events.as_slice())
                    .execute(&mut connection)
                    .map_err(|e| IndexerError::SQLiteWriteError(e.to_string()))
                    .context("Failed to write test case events to SQLiteDB")
                    .unwrap();

                thread::sleep(sleep_duration);
            }
        });

        // Give the writer time to get started
        thread::sleep(Duration::from_secs(2));

        // Spawn a thread to read from the DB
        let reader = thread::spawn(move || {
            let query = "SELECT * FROM events order by tx_order desc, event_index desc limit 1";
            tracing::debug!("query object test case events: {}", query);

            // Loop querying the number of rows
            loop {
                let result = inner_indexer_reader
                    .run_query(|conn| diesel::sql_query(query).load::<StoredEvent>(conn))
                    .unwrap();
                let count = if result.is_empty() {
                    0
                } else {
                    result[0].tx_order
                };

                if count % 100 == 0 {
                    println!("SQLite tx_order : {}", count);
                };
                if count >= get_row_count() {
                    return count;
                };
                thread::sleep(sleep_duration);
            }
        });

        writer.join().unwrap();
        Ok(reader.join().unwrap())
    }
}
