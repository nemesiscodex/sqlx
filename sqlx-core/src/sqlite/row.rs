use crate::database::HasValueRef;
use crate::error::Error;
use crate::row::{ColumnIndex, Row};
use crate::sqlite::statement::StatementHandle;
use crate::sqlite::{Sqlite, SqliteValue, SqliteValueRef};
use crossbeam_channel::{bounded, Receiver, Sender};
use std::ptr::null_mut;
use std::slice;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, Weak};

/// Implementation of [`Row`] for SQLite.
pub struct SqliteRow {
    // Raw handle of the SQLite statement
    // This is valid to access IFF the atomic [values] is null
    // The way this works is that the executor retains a weak reference to
    // [values] after the Row is created and yielded downstream.
    // IF the user drops the Row before iterating the stream (so
    // nearly all of our internal stream iterators), the executor moves on; otherwise,
    // it actually inflates this row with a list of owned sqlite3 values.
    pub(crate) statement: StatementHandle,
    pub(crate) values: Arc<AtomicPtr<SqliteValue>>,
    pub(crate) size: usize,
}

impl crate::row::private_row::Sealed for SqliteRow {}

// Accessing values from the statement object is
// safe across threads as long as we don't call [sqlite3_step]

// we block ourselves from doing that by only exposing
// a set interface on [StatementHandle]

unsafe impl Send for SqliteRow {}
unsafe impl Sync for SqliteRow {}

impl SqliteRow {
    // creates a new row that is internally referencing the **current** state of the statement
    // returns a weak reference to an atomic list where the executor should inflate if its going
    // to increment the statement with [step]
    pub(crate) fn current(statement: StatementHandle) -> (Self, Weak<AtomicPtr<SqliteValue>>) {
        let values = Arc::new(AtomicPtr::new(null_mut()));
        let weak_values = Arc::downgrade(&values);
        let size = statement.column_count();
        let row = Self {
            statement,
            size,
            values,
        };

        (row, weak_values)
    }

    // inflates this Row into memory as a list of owned, protected SQLite value objects
    // this is called by the
    pub(crate) fn inflate(statement: &StatementHandle, values_ref: &AtomicPtr<SqliteValue>) {
        let size = statement.column_count();
        let mut values = Vec::with_capacity(size);

        for i in 0..size {
            values.push(statement.column_value(i));
        }

        // decay the array signifier and become just a normal, leaked array
        let values_ptr = Box::into_raw(values.into_boxed_slice()) as *mut SqliteValue;

        // store in the atomic ptr storage
        values_ref.store(values_ptr, Ordering::Release);
    }

    pub(crate) fn inflate_if_needed(
        statement: &StatementHandle,
        weak_values_ref: Option<Weak<AtomicPtr<SqliteValue>>>,
    ) {
        if let Some(v) = weak_values_ref.and_then(|v| v.upgrade()) {
            SqliteRow::inflate(statement, &v);
        }
    }
}

impl Row for SqliteRow {
    type Database = Sqlite;

    fn len(&self) -> usize {
        self.size
    }

    fn try_get_raw<I>(&self, index: I) -> Result<SqliteValueRef<'_>, Error>
    where
        I: ColumnIndex<Self>,
    {
        let index = index.index(self)?;

        let values_ptr = self.values.load(Ordering::Acquire);
        if !values_ptr.is_null() {
            // we have raw value data, we should use that
            let values: &[SqliteValue] = unsafe { slice::from_raw_parts(values_ptr, self.size) };

            Ok(SqliteValueRef::value(&values[index]))
        } else {
            Ok(SqliteValueRef::statement(&self.statement, index))
        }
    }
}

impl Drop for SqliteRow {
    fn drop(&mut self) {
        // if there is a non-null pointer stored here, we need to re-load and drop it
        let values_ptr = self.values.load(Ordering::Acquire);
        if !values_ptr.is_null() {
            let values: &mut [SqliteValue] =
                unsafe { slice::from_raw_parts_mut(values_ptr, self.size) };

            let _ = unsafe { Box::from_raw(values) };
        }
    }
}
