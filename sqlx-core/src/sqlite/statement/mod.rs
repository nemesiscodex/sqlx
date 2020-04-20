use std::os::raw::c_char;
use std::ptr::{null, null_mut, NonNull};
use std::sync::{atomic::AtomicPtr, Weak};

use hashbrown::HashMap;
use libsqlite3_sys::{
    sqlite3_clear_bindings, sqlite3_finalize, sqlite3_prepare_v3, sqlite3_reset, sqlite3_stmt,
    SQLITE_OK, SQLITE_PREPARE_NO_VTAB, SQLITE_PREPARE_PERSISTENT,
};
use smallvec::SmallVec;

use crate::error::Error;
use crate::sqlite::{SqliteConnection, SqliteError, SqliteRow, SqliteValue};

mod handle;
mod maybe_owned;
mod worker;

pub(crate) use handle::StatementHandle;
pub(crate) use maybe_owned::MaybeOwnedStatement;
pub(crate) use worker::StatementWorker;

pub(crate) struct Statement {
    // underlying sqlite handles for each inner statement
    // a SQL query string in SQLite is broken up into N statements
    // we use a [`SmallVec`] to optimize for the most likely case of a single statement
    pub(crate) handles: SmallVec<[StatementHandle; 1]>,

    // column name -> index, for each statement
    column_names: SmallVec<[HashMap<String, usize>; 1]>,

    // weak reference to the previous row from this connection
    // we use the notice of a successful upgrade of this reference as an indicator that the
    // row is still around, in which we then inflate the row such that we can let SQLite
    // clobber the memory allocation for the row
    pub(crate) last_row_values: SmallVec<[Option<Weak<AtomicPtr<SqliteValue>>>; 1]>,
}

impl Statement {
    pub(crate) fn prepare(
        conn: &mut SqliteConnection,
        mut query: &str,
        persistent: bool,
    ) -> Result<Self, Error> {
        query = query.trim();

        if query.len() > i32::MAX as usize {
            return Err(err_protocol!(
                "query string must be smaller than {} bytes",
                i32::MAX
            ));
        }

        let mut handles: SmallVec<[StatementHandle; 1]> = SmallVec::with_capacity(1);
        let mut column_names: SmallVec<[HashMap<String, usize>; 1]> = SmallVec::with_capacity(1);

        let mut query_ptr = query.as_ptr() as *const c_char;
        let mut query_len = query.len() as i32;
        let mut flags = SQLITE_PREPARE_NO_VTAB;

        if persistent {
            // SQLITE_PREPARE_PERSISTENT
            //  The SQLITE_PREPARE_PERSISTENT flag is a hint to the query
            //  planner that the prepared statement will be retained for a long time
            //  and probably reused many times.
            flags |= SQLITE_PREPARE_PERSISTENT;
        }

        while query_len > 0 {
            let mut statement_handle: *mut sqlite3_stmt = null_mut();
            let mut tail: *const c_char = null();

            // <https://www.sqlite.org/c3ref/prepare.html>
            let status = unsafe {
                sqlite3_prepare_v3(
                    conn.handle.as_ptr(),
                    query_ptr,
                    query_len,
                    flags as u32,
                    &mut statement_handle,
                    &mut tail,
                )
            };

            if status != SQLITE_OK {
                return Err(SqliteError::new(conn.handle.as_ptr()).into());
            }

            // a null handle is generated when the sql statement contains nothing
            // interesting; like only a comment
            if let Some(handle) = NonNull::new(statement_handle) {
                let handle = StatementHandle(handle);

                // prepare a column hash map for use in pulling values from a column
                // by name

                let count = handle.column_count();
                let mut names = HashMap::with_capacity(count);

                for i in 0..count {
                    names.insert(handle.column_name(i).to_owned(), i);
                }

                handles.push(handle);
                column_names.push(names);
            }

            // tail should point to the first byte past the end of the first SQL
            // statement in zSql. these routines only compile the first statement,
            // so tail is left pointing to what remains un-compiled.

            query_len -= (tail as i32) - (query_ptr as i32);
            query_ptr = tail;
        }

        Ok(Self {
            handles,
            column_names,
            last_row_values: SmallVec::from([None; 1]),
        })
    }

    pub(crate) fn reset(&mut self) {
        for (i, handle) in self.handles.iter().enumerate() {
            SqliteRow::inflate_if_needed(&handle, self.last_row_values[i].take());

            unsafe {
                // Reset A Prepared Statement Object
                // https://www.sqlite.org/c3ref/reset.html
                // https://www.sqlite.org/c3ref/clear_bindings.html
                sqlite3_reset(handle.0.as_ptr());
                sqlite3_clear_bindings(handle.0.as_ptr());
            }
        }
    }
}

impl Drop for Statement {
    fn drop(&mut self) {
        for (i, handle) in self.handles.drain(..).enumerate() {
            SqliteRow::inflate_if_needed(&handle, self.last_row_values[i].take());

            unsafe {
                // https://sqlite.org/c3ref/finalize.html
                let _ = sqlite3_finalize(handle.0.as_ptr());
            }
        }
    }
}
