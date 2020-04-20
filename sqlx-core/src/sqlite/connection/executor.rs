use std::borrow::Cow;
use std::sync::{atomic::AtomicPtr, Arc, Weak};

use libsqlite3_sys::SQLITE_ROW;

use async_stream::try_stream;
use crossbeam_channel as crossbeam;
use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_core::Stream;
use futures_util::{future, pin_mut};
use futures_util::{FutureExt, StreamExt, TryStreamExt};

use crate::error::Error;
use crate::executor::{Execute, Executor};
use crate::sqlite::statement::{MaybeOwnedStatement, Statement};
use crate::sqlite::{Sqlite, SqliteArguments, SqliteConnection, SqliteRow, SqliteValue};

impl SqliteConnection {
    fn prepare<'a: 'b, 'b>(
        &'a mut self,
        query: &str,
        persistent: bool,
    ) -> Result<MaybeOwnedStatement<'_>, Error> {
        if !persistent {
            return Ok(MaybeOwnedStatement::Owned(Statement::prepare(
                self, query, false,
            )?));
        }

        if !self.statements.contains_key(query) {
            let statement = Statement::prepare(self, query, true)?;
            self.statements.insert(query.to_owned(), statement);
        }

        let statement = self.statements.get_mut(query).unwrap();

        // as this statement has been executed before, we reset before continuing
        // this also causes any rows that are from the statement to be inflated
        statement.reset();

        Ok(MaybeOwnedStatement::Borrowed(statement))
    }

    fn run(
        &mut self,
        query: &str,
        arguments: Option<SqliteArguments<'_>>,
    ) -> Result<MaybeOwnedStatement<'_>, Error> {
        let mut statement = self.prepare(query, arguments.is_some())?;

        if let Some(arguments) = arguments {
            arguments.bind(&*statement)?;
        }

        Ok(statement)
    }
}

impl<'c> Executor<'c> for &'c mut SqliteConnection {
    type Database = Sqlite;

    fn fetch_many<'q: 'c, E>(
        mut self,
        mut query: E,
    ) -> BoxStream<'c, Result<Either<u64, SqliteRow>, Error>>
    where
        E: Execute<'q, Self::Database>,
    {
        let s = query.query();
        let arguments = query.take_arguments();

        Box::pin(try_stream! {
            let worker = Arc::clone(&self.worker);
            let mut stmt = self.run(s, arguments)?;

            // TODO: support N handles
            let handle = stmt.handles[0];
            worker.execute(&handle);

            loop {
                // save the rows from the _current_ position on the statement
                // and send them to the still-live row object
                SqliteRow::inflate_if_needed(&handle, stmt.last_row_values[0].take());

                match worker.step(&handle).await? {
                    Either::Left(changes) => {
                        let v = Either::Left(changes);
                        yield v;

                        // TODO: support N handles
                        break;
                    }

                    Either::Right(()) => {
                        let (row, weak_values_ref) = SqliteRow::current(handle);
                        let v = Either::Right(row);
                        stmt.last_row_values[0] = Some(weak_values_ref);

                        yield v;
                    }
                }
            }
        })
    }

    fn fetch_optional<'q: 'c, E>(
        self,
        mut query: E,
    ) -> BoxFuture<'c, Result<Option<SqliteRow>, Error>>
    where
        E: Execute<'q, Self::Database>,
    {
        let mut s = self.fetch_many(query);

        Box::pin(async move {
            pin_mut!(s);

            while let Some(v) = s.try_next().await? {
                if let Either::Right(r) = v {
                    return Ok(Some(r));
                }
            }

            Ok(None)
        })
    }
}
