use std::fmt::Debug;

use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::{future, pin_mut, FutureExt, StreamExt, TryFutureExt, TryStreamExt};

use crate::database::{Database, HasArguments};
use crate::error::Error;
use crate::value::Value;

/// A type that contains or can provide a database
/// connection to use for executing queries against the database.
///
/// No guarantees are provided that successive queries run on the same
/// physical database connection.
///
/// A [`Connection`](crate::connection::Connection) is an `Executor` that guarantees that
/// successive queries are ran on the same physical database connection.
///
/// Implemented for the following:
///
///  * [`&Pool`]
///  * [`&mut PoolConnection`]
///  * [`&mut Connection`]
///  * [`&mut Transaction`]
///
pub trait Executor<'c>: Send + Debug + Sized {
    type Database: Database;

    /// Execute the query and return the total number of rows affected.
    fn execute<'q: 'c, E>(self, query: E) -> BoxFuture<'c, Result<u64, Error>>
    where
        E: Execute<'q, Self::Database>,
    {
        self.execute_many(query)
            .try_fold(0, |acc, x| async move { Ok(acc + x) })
            .boxed()
    }

    /// Execute multiple queries and return the rows affected from each query, in a stream.
    fn execute_many<'q: 'c, E>(self, query: E) -> BoxStream<'c, Result<u64, Error>>
    where
        E: Execute<'q, Self::Database>,
    {
        self.fetch_many(query)
            .try_filter_map(|step| async move {
                Ok(match step {
                    Either::Left(rows) => Some(rows),
                    Either::Right(row) => None,
                })
            })
            .boxed()
    }

    /// Execute the query and return the generated results as a stream.
    fn fetch<'q: 'c, E>(
        self,
        query: E,
    ) -> BoxStream<'c, Result<<Self::Database as Database>::Row, Error>>
    where
        E: Execute<'q, Self::Database>,
    {
        self.fetch_many(query)
            .try_filter_map(|step| async move {
                Ok(match step {
                    Either::Left(_) => None,
                    Either::Right(row) => Some(row),
                })
            })
            .boxed()
    }

    /// Execute multiple queries and return the generated results as a stream
    /// from each query, in a stream.
    fn fetch_many<'q: 'c, E>(
        self,
        query: E,
    ) -> BoxStream<'c, Result<Either<u64, <Self::Database as Database>::Row>, Error>>
    where
        E: Execute<'q, Self::Database>;

    /// Execute the query and return all the generated results, collected into a [`Vec`].
    fn fetch_all<'q: 'c, E>(
        self,
        query: E,
    ) -> BoxFuture<'c, Result<Vec<<Self::Database as Database>::Row>, Error>>
    where
        E: Execute<'q, Self::Database>,
    {
        self.fetch(query).try_collect().boxed()
    }

    /// Execute the query and returns exactly one row.
    fn fetch_one<'q: 'c, E>(
        self,
        query: E,
    ) -> BoxFuture<'c, Result<<Self::Database as Database>::Row, Error>>
    where
        E: Execute<'q, Self::Database>,
    {
        self.fetch_optional(query)
            .and_then(|row| match row {
                Some(row) => future::ok(row),
                None => future::err(Error::RowNotFound),
            })
            .boxed()
    }

    /// Execute the query and returns at most one row.
    fn fetch_optional<'q: 'c, E>(
        self,
        query: E,
    ) -> BoxFuture<'c, Result<Option<<Self::Database as Database>::Row>, Error>>
    where
        E: Execute<'q, Self::Database>;
}

/// A type that may be executed against a database connection.
///
/// Implemented for the following:
///
///  * [`&str`]
///  * [`Query`]
///
pub trait Execute<'q, DB: Database>: Send {
    fn query(&self) -> &'q str;

    fn take_arguments(&mut self) -> Option<<DB as HasArguments<'q>>::Arguments>;
}

impl<'q, DB: Database> Execute<'q, DB> for &'q str {
    #[inline]
    fn query(&self) -> &'q str {
        self
    }

    #[inline]
    fn take_arguments(&mut self) -> Option<<DB as HasArguments<'q>>::Arguments> {
        None
    }
}
