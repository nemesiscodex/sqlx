use std::ffi::{c_void, CString};
use std::io;
use std::os::raw::c_char;
use std::ptr::{null, null_mut, NonNull};
use std::sync::Arc;

use hashbrown::HashMap;
use libsqlite3_sys::{
    sqlite3, sqlite3_extended_result_codes, sqlite3_open_v2, SQLITE_OK, SQLITE_OPEN_CREATE,
    SQLITE_OPEN_MEMORY, SQLITE_OPEN_NOMUTEX, SQLITE_OPEN_PRIVATECACHE, SQLITE_OPEN_READWRITE,
};
use sqlx_rt::blocking;

use crate::error::Error;
use crate::sqlite::connection::handle::ConnectionHandle;
use crate::sqlite::statement::StatementWorker;
use crate::sqlite::{SqliteConnectOptions, SqliteConnection, SqliteError};

pub(super) async fn establish(options: &SqliteConnectOptions) -> Result<SqliteConnection, Error> {
    let mut filename = options
        .filename
        .to_str()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "filename passed to SQLite must be valid UTF-8",
            )
        })?
        .to_owned();

    filename.push('\0');

    // By default, we connect to an in-memory database.
    // [SQLITE_OPEN_NOMUTEX] will instruct [sqlite3_open_v2] to return an error if it
    // cannot satisfy our wish for a thread-safe, lock-free connection object
    let mut flags =
        SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_NOMUTEX | SQLITE_OPEN_PRIVATECACHE;

    if options.in_memory {
        flags |= SQLITE_OPEN_MEMORY;
    }

    let handle = blocking!({
        let mut handle = null_mut();

        // <https://www.sqlite.org/c3ref/open.html>
        let status = unsafe {
            sqlite3_open_v2(
                filename.as_bytes().as_ptr() as *const _,
                &mut handle,
                flags,
                null(),
            )
        };

        if handle.is_null() {
            // Failed to allocate memory
            panic!("SQLite is unable to allocate memory to hold the sqlite3 object");
        }

        // SAFE: tested for NULL just above
        let handle = unsafe { ConnectionHandle::new(handle) };

        if status != SQLITE_OK {
            return Err(Error::Database(Box::new(SqliteError::new(handle.as_ptr()))));
        }

        // Enable extended result codes
        // https://www.sqlite.org/c3ref/extended_result_codes.html
        unsafe {
            sqlite3_extended_result_codes(handle.0.as_ptr(), 1);
        }

        Ok(handle)
    })?;

    Ok(SqliteConnection {
        handle,
        worker: Arc::new(StatementWorker::new()),
        statements: HashMap::new(),
    })
}
