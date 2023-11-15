// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::SqlitePoolConnection;
use anyhow::anyhow;
use diesel::migration::MigrationSource;
use diesel::{RunQueryDsl, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tracing::info;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// creates all the tables by applying all migrations.
pub fn create_all_tables(conn: &mut SqlitePoolConnection) -> Result<(), anyhow::Error> {
    info!("Creates all tables in the database ...");
    let migration = MIGRATIONS;
    conn.run_migrations(&migration.migrations().unwrap())
        .map_err(|e| anyhow!("Failed to run migrations {e}"))?;
    info!("Creates all tables complete.");
    Ok(())
}

/// Resets the database by reverting all migrations and reapplying them.
///
/// If `drop_all` is set to `true`, the function will drop all tables in the database before
/// resetting the migrations. This option is destructive and will result in the loss of all
/// data in the tables. Use with caution, especially in production environments.
pub fn reset_database(
    conn: &mut SqlitePoolConnection,
    drop_all: bool,
) -> Result<(), anyhow::Error> {
    info!("Resetting database ...");
    let migration = MIGRATIONS;
    if drop_all {
        drop_all_tables(conn)
            .map_err(|e| anyhow!("Encountering error when dropping all tables {e}"))?;
    } else {
        conn.revert_all_migrations(migration)
            .map_err(|e| anyhow!("Error reverting all migrations {e}"))?;
    }
    let migration = MIGRATIONS;
    conn.run_migrations(&migration.migrations().unwrap())
        .map_err(|e| anyhow!("Failed to run migrations {e}"))?;
    info!("Reset database complete.");
    Ok(())
}

pub fn drop_all_tables(conn: &mut SqliteConnection) -> Result<(), diesel::result::Error> {
    info!("Dropping all tables in the database");
    let table_names: Vec<String> = diesel::dsl::sql::<diesel::sql_types::Text>(
        "
        SELECT tablename FROM sqlite_tables WHERE schemaname = 'public'
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
            version VARCHAR(50) PRIMARY KEY,
            run_on TIMESTAMP NOT NULL DEFAULT NOW()
        )
    ",
    )
    .execute(conn)?;
    info!("Dropped all tables in the database");
    Ok(())
}
