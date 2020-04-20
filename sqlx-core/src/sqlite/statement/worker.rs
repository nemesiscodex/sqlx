use std::ptr::{null_mut, NonNull};
use std::sync::atomic::{spin_loop_hint, AtomicI32, AtomicPtr, Ordering};
use std::sync::Arc;
use std::thread::{park, spawn, JoinHandle};

use either::Either;
use libsqlite3_sys::{sqlite3_step, sqlite3_stmt, SQLITE_DONE, SQLITE_ROW};
use sqlx_rt::yield_now;

use crate::error::Error;
use crate::sqlite::statement::StatementHandle;
use crate::sqlite::{SqliteError, SqliteRow};

// For async-std and actix, the worker maintains a dedicated thread for each SQLite connection
// All invocations of [sqlite3_step] are run on this thread

// For tokio, the worker is a thin wrapper around an invocation to [block_in_place]

#[cfg(not(feature = "runtime-tokio"))]
pub(crate) struct StatementWorker {
    statement: Arc<AtomicPtr<sqlite3_stmt>>,
    status: Arc<AtomicI32>,
    handle: Option<JoinHandle<()>>,
}

#[cfg(feature = "runtime-tokio")]
pub(crate) struct StatementWorker {}

#[cfg(not(feature = "runtime-tokio"))]
impl StatementWorker {
    pub(crate) fn new() -> Self {
        let statement = Arc::new(AtomicPtr::new(null_mut::<sqlite3_stmt>()));
        let status = Arc::new(AtomicI32::new(0));

        let handle = spawn({
            let status = Arc::clone(&status);
            let statement_v = Arc::clone(&statement);

            move || {
                'run: while status.load(Ordering::Acquire) >= 0 {
                    // wait for a statement to execute
                    park();

                    'statement: while status.load(Ordering::Acquire) >= 0 {
                        match status.load(Ordering::Acquire) {
                            0 => {
                                let statement = unsafe { statement_v.load(Ordering::Acquire) };
                                let v = unsafe { sqlite3_step(statement) };

                                status.store(v, Ordering::Release);

                                if v == SQLITE_DONE {
                                    // when a statement is _done_, we park the thread until
                                    // we need it again
                                    break 'statement;
                                }
                            }

                            v @ _ => {
                                // waits for the receiving end to be ready to receive the rows
                                // this should take less than 1 microsecond under most conditions
                                spin_loop_hint();
                            }
                        }
                    }
                }
            }
        });

        Self {
            handle: Some(handle),
            status,
            statement,
        }
    }

    pub(crate) fn execute(&self, statement: &StatementHandle) {
        // readies the worker to execute the statement
        // for async-std, this unparks our dedicated thread

        self.statement
            .store(statement.0.as_ptr(), Ordering::Release);

        if let Some(handle) = &self.handle {
            handle.thread().unpark();
        }
    }

    pub(crate) async fn step(&self, statement: &StatementHandle) -> Result<Either<u64, ()>, Error> {
        // storing <0> as a terminal in status releases the worker
        // to proceed to the next [sqlite3_step] invocation
        self.status.store(0, Ordering::Release);

        // we then use a spin loop to wait for this to finish
        // 99% of the time this should be < 1 μs
        let status = loop {
            let status = self.status.compare_and_swap(0, 0, Ordering::AcqRel);
            if status != 0 {
                break status;
            }

            yield_now().await;
        };

        match status {
            // a row was found
            SQLITE_ROW => Ok(Either::Right(())),

            // reached the end of the query results,
            // emit the # of changes
            SQLITE_DONE => Ok(Either::Left(statement.changes())),

            _ => Err(statement.last_error().into()),
        }
    }
}

#[cfg(feature = "runtime-tokio")]
impl StatementWorker {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {}
    }

    #[inline]
    pub(crate) fn execute(&self, _statement: &StatementHandle) {}

    #[inline]
    pub(crate) async fn step(&self, statement: &StatementHandle) -> Result<Either<u64, ()>, Error> {
        let statement = *statement;
        let status = sqlx_rt::blocking!({ unsafe { sqlite3_step(statement.0.as_ptr()) } });

        match status {
            // a row was found
            SQLITE_ROW => Ok(Either::Right(())),

            // reached the end of the query results,
            // emit the # of changes
            SQLITE_DONE => Ok(Either::Left(statement.changes())),

            _ => Err(statement.last_error().into()),
        }
    }
}

#[cfg(not(feature = "runtime-tokio"))]
impl Drop for StatementWorker {
    fn drop(&mut self) {
        // -1 will get the inner thread to stop
        self.status.store(-1, Ordering::Release);

        if let Some(handle) = self.handle.take() {
            handle.thread().unpark();
            handle.join().unwrap();
        }
    }
}
