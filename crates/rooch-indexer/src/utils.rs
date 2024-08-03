// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::SqlitePoolConnection;
use anyhow::anyhow;
use diesel::{RunQueryDsl, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tracing::{debug, info};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// creates all the tables by applying all migrations.
pub fn create_all_tables_if_not_exists(
    conn: &mut SqlitePoolConnection,
    _table_name: String,
) -> Result<(), anyhow::Error> {
    debug!("Indexer creates all tables in the db ...");
    let migration = MIGRATIONS;

    // Create the __diesel_schema_migrations table if not exist
    diesel::sql_query(
        "
        CREATE TABLE IF NOT EXISTS __diesel_schema_migrations (
            version VARCHAR(50) PRIMARY KEY NOT NULL,
            run_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
    ",
    )
    .execute(conn)?;

    // TODO need filter by talbe name.
    // The fields and methods of EmbeddedMigrations in Diesel Lib are private.
    // There is no API support for filtering by tablename.
    // We may consider forking diesel to provide the corresponding API to implement the filtering function.

    // let match_migration = &MIGRATIONS
    //     .migrations()?
    //     .into_iter()
    //     .map(|v| {
    //         if v.name().version().to_string().ends_with(table_name) {
    //             v.clone()
    //         }
    //     })
    //     // .collect::<Vec<Box<dyn Migration<Sqlite>>>>();
    //     .collect::<Vec<_>>();

    conn.run_pending_migrations(migration)
        .map_err(|e| anyhow!("Failed to run migrations {e}"))?;
    debug!("Indexer creates all tables complete.");
    Ok(())
}

/// Resets the db by reverting all migrations and reapplying them.
///
/// If `drop_all` is set to `true`, the function will drop all tables in the db before
/// resetting the migrations. This option is destructive and will result in the loss of all
/// data in the tables. Use with caution, especially in production environments.
pub fn reset_db(conn: &mut SqlitePoolConnection, drop_all: bool) -> Result<(), anyhow::Error> {
    info!("Resetting db ...");
    let migration = MIGRATIONS;
    if drop_all {
        drop_all_tables(conn)
            .map_err(|e| anyhow!("Encountering error when dropping all tables {e}"))?;
    } else {
        conn.revert_all_migrations(migration)
            .map_err(|e| anyhow!("Error reverting all migrations {e}"))?;
    }
    let migration = MIGRATIONS;
    conn.run_pending_migrations(migration)
        .map_err(|e| anyhow!("Failed to run migrations {e}"))?;
    info!("Reset db complete.");
    Ok(())
}

pub fn drop_all_tables(conn: &mut SqliteConnection) -> Result<(), diesel::result::Error> {
    info!("Dropping all tables in the db ...");
    let table_names: Vec<String> = diesel::dsl::sql::<diesel::sql_types::Text>(
        "
        SELECT name FROM sqlite_schema WHERE type = 'table'
    ",
    )
    .load(conn)?;

    for table_name in table_names {
        let drop_table_query = format!("DROP TABLE IF EXISTS {} CASCADE", table_name);
        diesel::sql_query(drop_table_query).execute(conn)?;
    }

    // Recreate the __diesel_schema_migrations table
    diesel::sql_query(
        "
        CREATE TABLE __diesel_schema_migrations (
            version VARCHAR(50) PRIMARY KEY NOT NULL,
            run_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
    ",
    )
    .execute(conn)?;
    info!("Dropped all tables complete.");
    Ok(())
}

pub fn escape_sql_string(value: String) -> String {
    // In SQLite, replace single quotes with two single quotes
    value.replace(['\''], "''")
}
