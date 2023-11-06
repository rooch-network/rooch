// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]
use diesel::backend::Backend;
use diesel::connection::{AnsiTransactionManager, TransactionManager};
use diesel::prelude::*;
use diesel::query_builder::{AstPass, QueryBuilder, QueryFragment};
use diesel::result::Error;

use diesel::sqlite::Sqlite;
use diesel::Connection;

/// Used to build a transaction, specifying additional details.
///
/// This struct is returned by [`.build_transaction`].
/// See the documentation for methods on this struct for usage examples.
/// for details on the behavior of each option.
///
/// [`.build_transaction`]: SqliteConnection::build_transaction()
/// [sqlite-docs]: https://www.sqlite.org/lang_transaction.html
#[allow(missing_debug_implementations)] // False positive. Connection isn't Debug.
#[must_use = "Transaction builder does nothing unless you call `run` on it"]
pub struct TransactionBuilder<'a, C> {
    connection: &'a mut C,
    read_mode: Option<ReadMode>,
    deferrable: Option<Deferrable>,
}

impl<'a, C> TransactionBuilder<'a, C>
where
    C: Connection<Backend = Sqlite, TransactionManager = AnsiTransactionManager>,
{
    /// Creates a new TransactionBuilder.
    #[diesel::diesel_derives::__diesel_public_if(
        feature = "i-implement-a-third-party-backend-and-opt-into-breaking-changes"
    )]
    pub(crate) fn new(connection: &'a mut C) -> Self {
        Self {
            connection,
            read_mode: None,
            deferrable: None,
        }
    }

    /// Makes the transaction `READ ONLY`
    ///
    /// # Example
    ///
    /// ```rust
    /// # include!("../doctest_setup.rs");
    /// # use diesel::sql_query;
    /// #
    /// # fn main() {
    /// #     run_test().unwrap();
    /// # }
    /// #
    /// # table! {
    /// #     users_for_read_only {
    /// #         id -> Integer,
    /// #         name -> Text,
    /// #     }
    /// # }
    /// #
    /// # fn run_test() -> QueryResult<()> {
    /// #     use users_for_read_only::table as users;
    /// #     use users_for_read_only::columns::*;
    /// #     let conn = &mut connection_no_transaction();
    /// #     sql_query("CREATE TABLE IF NOT EXISTS users_for_read_only (
    /// #       id SERIAL PRIMARY KEY,
    /// #       name TEXT NOT NULL
    /// #     )").execute(conn)?;
    /// conn.build_transaction()
    ///     .read_only()
    ///     .run::<_, diesel::result::Error, _>(|conn| {
    ///         let read_attempt = users.select(name).load::<String>(conn);
    ///         assert!(read_attempt.is_ok());
    ///
    ///         let write_attempt = diesel::insert_into(users)
    ///             .values(name.eq("Ruby"))
    ///             .execute(conn);
    ///         assert!(write_attempt.is_err());
    ///
    ///         Ok(())
    ///     })?;
    /// #     sql_query("DROP TABLE users_for_read_only").execute(conn)?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn read_only(mut self) -> Self {
        self.read_mode = Some(ReadMode::ReadOnly);
        self
    }

    /// Makes the transaction `READ WRITE`
    ///
    /// This is the default, unless you've changed the
    /// `default_transaction_read_only` configuration parameter.
    ///
    /// # Example
    ///
    /// ```rust
    /// # include!("../doctest_setup.rs");
    /// # use diesel::result::Error::RollbackTransaction;
    /// # use diesel::sql_query;
    /// #
    /// # fn main() {
    /// #     assert_eq!(run_test(), Err(RollbackTransaction));
    /// # }
    /// #
    /// # fn run_test() -> QueryResult<()> {
    /// #     use schema::users::dsl::*;
    /// #     let conn = &mut connection_no_transaction();
    /// conn.build_transaction()
    ///     .read_write()
    ///     .run(|conn| {
    /// #         sql_query("CREATE TABLE IF NOT EXISTS users (
    /// #             id SERIAL PRIMARY KEY,
    /// #             name TEXT NOT NULL
    /// #         )").execute(conn)?;
    ///         let read_attempt = users.select(name).load::<String>(conn);
    ///         assert!(read_attempt.is_ok());
    ///
    ///         let write_attempt = diesel::insert_into(users)
    ///             .values(name.eq("Ruby"))
    ///             .execute(conn);
    ///         assert!(write_attempt.is_ok());
    ///
    /// #       Err(RollbackTransaction)
    /// #       /*
    ///         Ok(())
    /// #       */
    ///     })
    /// # }
    /// ```
    pub fn read_write(mut self) -> Self {
        self.read_mode = Some(ReadMode::ReadWrite);
        self
    }

    /// Makes the transaction `DEFERRABLE`
    ///
    /// # Example
    ///
    /// ```rust
    /// # include!("../doctest_setup.rs");
    /// #
    /// # fn main() {
    /// #     run_test().unwrap();
    /// # }
    /// #
    /// # fn run_test() -> QueryResult<()> {
    /// #     use schema::users::dsl::*;
    /// #     let conn = &mut connection_no_transaction();
    /// conn.build_transaction()
    ///     .deferrable()
    ///     .run(|conn| Ok(()))
    /// # }
    /// ```
    pub fn deferrable(mut self) -> Self {
        self.deferrable = Some(Deferrable::Deferrable);
        self
    }

    /// Makes the transaction `NOT DEFERRABLE`
    ///
    /// This is the default, unless you've changed the
    /// `default_transaction_deferrable` configuration parameter.
    ///
    /// # Example
    ///
    /// ```rust
    /// # include!("../doctest_setup.rs");
    /// #
    /// # fn main() {
    /// #     run_test().unwrap();
    /// # }
    /// #
    /// # fn run_test() -> QueryResult<()> {
    /// #     use schema::users::dsl::*;
    /// #     let conn = &mut connection_no_transaction();
    /// conn.build_transaction()
    ///     .not_deferrable()
    ///     .run(|conn| Ok(()))
    /// # }
    /// ```
    pub fn not_deferrable(mut self) -> Self {
        self.deferrable = Some(Deferrable::NotDeferrable);
        self
    }

    /// Runs the given function inside of the transaction
    /// with the parameters given to this builder.
    ///
    /// This function executes the provided closure `f` inside a database
    /// transaction. If there is already an open transaction for the current
    /// connection it will return an error. The connection is committed if
    /// the closure returns `Ok(_)`, it will be rolled back if it returns `Err(_)`.
    /// For both cases the original result value will be returned from this function.
    ///
    /// If the transaction fails to commit and requires a rollback according to Postgres,
    /// (e.g. serialization failure) a rollback will be attempted.
    /// If the rollback fails, the error will be returned in a
    /// [`Error::RollbackErrorOnCommit`](crate::result::Error::RollbackErrorOnCommit),
    /// from which you will be able to extract both the original commit error and
    /// the rollback error.
    /// In addition, the connection will be considered broken
    /// as it contains a uncommitted unabortable open transaction. Any further
    /// interaction with the transaction system will result in an returned error
    /// in this case.
    pub fn run<T, E, F>(&mut self, f: F) -> Result<T, E>
    where
        F: FnOnce(&mut C) -> Result<T, E>,
        E: From<Error>,
    {
        let mut query_builder = <Sqlite as Backend>::QueryBuilder::default();
        self.to_sql(&mut query_builder, &Sqlite)?;
        let sql = query_builder.finish();

        AnsiTransactionManager::begin_transaction_sql(&mut *self.connection, &sql)?;
        match f(&mut *self.connection) {
            Ok(value) => {
                AnsiTransactionManager::commit_transaction(&mut *self.connection)?;
                Ok(value)
            }
            Err(user_error) => {
                match AnsiTransactionManager::rollback_transaction(&mut *self.connection) {
                    Ok(()) => Err(user_error),
                    Err(Error::BrokenTransactionManager) => {
                        // In this case we are probably more interested by the
                        // original error, which likely caused this
                        Err(user_error)
                    }
                    Err(rollback_error) => Err(rollback_error.into()),
                }
            }
        }
    }
}

impl<'a, C> QueryFragment<Sqlite> for TransactionBuilder<'a, C> {
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Sqlite>) -> QueryResult<()> {
        out.push_sql("BEGIN TRANSACTION");
        // if let Some(ref isolation_level) = self.isolation_level {
        //     isolation_level.walk_ast(out.reborrow())?;
        // }
        if let Some(ref read_mode) = self.read_mode {
            read_mode.walk_ast(out.reborrow())?;
        }
        if let Some(ref deferrable) = self.deferrable {
            deferrable.walk_ast(out.reborrow())?;
        }
        Ok(())
    }
}

// #[derive(Debug, Clone, Copy)]
// enum IsolationLevel {
//     ReadCommitted,
//     RepeatableRead,
//     Serializable,
// }
//
// impl QueryFragment<Sqlite> for IsolationLevel {
//     fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Sqlite>) -> QueryResult<()> {
//         out.push_sql(" ISOLATION LEVEL ");
//         match *self {
//             IsolationLevel::ReadCommitted => out.push_sql("READ COMMITTED"),
//             IsolationLevel::RepeatableRead => out.push_sql("REPEATABLE READ"),
//             IsolationLevel::Serializable => out.push_sql("SERIALIZABLE"),
//         }
//         Ok(())
//     }
// }

#[derive(Debug, Clone, Copy)]
enum ReadMode {
    ReadOnly,
    ReadWrite,
}

impl QueryFragment<Sqlite> for ReadMode {
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Sqlite>) -> QueryResult<()> {
        match *self {
            ReadMode::ReadOnly => out.push_sql(" READ ONLY"),
            ReadMode::ReadWrite => out.push_sql(" READ WRITE"),
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum Deferrable {
    Deferrable,
    NotDeferrable,
}

impl QueryFragment<Sqlite> for Deferrable {
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Sqlite>) -> QueryResult<()> {
        match *self {
            Deferrable::Deferrable => out.push_sql(" DEFERRABLE"),
            Deferrable::NotDeferrable => out.push_sql(" NOT DEFERRABLE"),
        }
        Ok(())
    }
}

#[test]
fn test_transaction_builder_generates_correct_sql() {
    extern crate dotenvy;

    macro_rules! assert_sql {
        ($query:expr, $sql:expr) => {
            let mut query_builder = <Sqlite as Backend>::QueryBuilder::default();
            $query.to_sql(&mut query_builder, &Sqlite).unwrap();
            let sql = query_builder.finish();
            assert_eq!(sql, $sql);
        };
    }

    let database_url = dotenvy::var("PG_DATABASE_URL")
        .or_else(|_| dotenvy::var("DATABASE_URL"))
        .expect("DATABASE_URL must be set in order to run tests");
    let mut conn = SqliteConnection::establish(&database_url).unwrap();

    assert_sql!(conn.build_transaction(), "BEGIN TRANSACTION");
    assert_sql!(
        conn.build_transaction().read_only(),
        "BEGIN TRANSACTION READ ONLY"
    );
    assert_sql!(
        conn.build_transaction().read_write(),
        "BEGIN TRANSACTION READ WRITE"
    );
    assert_sql!(
        conn.build_transaction().deferrable(),
        "BEGIN TRANSACTION DEFERRABLE"
    );
    assert_sql!(
        conn.build_transaction().not_deferrable(),
        "BEGIN TRANSACTION NOT DEFERRABLE"
    );
    assert_sql!(
        conn.build_transaction().read_committed(),
        "BEGIN TRANSACTION ISOLATION LEVEL READ COMMITTED"
    );
    assert_sql!(
        conn.build_transaction().repeatable_read(),
        "BEGIN TRANSACTION ISOLATION LEVEL REPEATABLE READ"
    );
    assert_sql!(
        conn.build_transaction().serializable(),
        "BEGIN TRANSACTION ISOLATION LEVEL SERIALIZABLE"
    );
    assert_sql!(
        conn.build_transaction()
            .serializable()
            .deferrable()
            .read_only(),
        "BEGIN TRANSACTION ISOLATION LEVEL SERIALIZABLE READ ONLY DEFERRABLE"
    );
}
