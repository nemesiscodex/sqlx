use std::fmt::{self, Debug, Formatter};
use std::ptr::NonNull;
use std::sync::atomic::AtomicPtr;
use std::sync::{Arc, Weak};

use futures_core::future::BoxFuture;
use futures_util::future;
use hashbrown::HashMap;

use crate::connection::{Connect, Connection};
use crate::error::Error;
use crate::executor::Executor;
use crate::sqlite::connection::establish::establish;
use crate::sqlite::statement::{Statement, StatementWorker};
use crate::sqlite::{Sqlite, SqliteConnectOptions, SqliteValue};

mod establish;
mod executor;
mod handle;

pub(crate) use handle::ConnectionHandle;

/// A connection to a [Sqlite] database.
pub struct SqliteConnection {
    pub(crate) handle: ConnectionHandle,
    pub(crate) statements: HashMap<String, Statement>,
    pub(crate) worker: Arc<StatementWorker>,
}

impl Debug for SqliteConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PgConnection").finish()
    }
}

impl Connection for SqliteConnection {
    type Database = Sqlite;

    fn close(mut self) -> BoxFuture<'static, Result<(), Error>> {
        todo!()
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        // For SQLite connections, PING does effectively nothing
        Box::pin(future::ok(()))
    }
}

impl Connect for SqliteConnection {
    type Options = SqliteConnectOptions;

    #[inline]
    fn connect_with(options: &Self::Options) -> BoxFuture<'_, Result<Self, Error>> {
        Box::pin(async move {
            let mut conn = establish(options).await?;

            // https://www.sqlite.org/wal.html

            // language=SQLite
            //             conn.execute(
            //                 r#"
            // PRAGMA journal_mode = WAL;
            // PRAGMA synchronous = NORMAL;
            //                 "#,
            //             )
            //             .await?;

            // conn.execute("PRAGMA journal_mode = WAL").await?;
            // conn.execute("PRAGMA synchronous = 0").await?;

            Ok(conn)
        })
    }
}

impl Drop for SqliteConnection {
    fn drop(&mut self) {
        // before the connection handle is dropped,
        // we must explicitly drop the statements as the drop-order in a struct is undefined
        self.statements.clear();
    }
}
