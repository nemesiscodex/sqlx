use std::ptr::NonNull;

use libsqlite3_sys::{
    sqlite3_value, sqlite3_value_blob, sqlite3_value_bytes, sqlite3_value_double,
    sqlite3_value_dup, sqlite3_value_int, sqlite3_value_int64, sqlite3_value_type, SQLITE_NULL,
};

use crate::database::HasValueRef;
use crate::decode::Error as DecodeError;
use crate::sqlite::statement::StatementHandle;
use crate::sqlite::{Sqlite, SqliteTypeInfo};
use crate::value::{Value, ValueRef};
use bitflags::_core::slice::from_raw_parts;
use bitflags::_core::str::from_utf8;

enum SqliteValueData<'r> {
    Statement {
        statement: &'r StatementHandle,
        index: usize,
    },

    Value(&'r SqliteValue),
}

pub struct SqliteValueRef<'r>(SqliteValueData<'r>);

impl<'r> SqliteValueRef<'r> {
    pub(crate) fn value(value: &'r SqliteValue) -> Self {
        Self(SqliteValueData::Value(value))
    }

    pub(crate) fn statement(statement: &'r StatementHandle, index: usize) -> Self {
        Self(SqliteValueData::Statement { statement, index })
    }

    pub(super) fn is_null(&self) -> bool {
        match self.0 {
            SqliteValueData::Statement { statement, index } => {
                statement.column_type(index) == SQLITE_NULL
            }

            SqliteValueData::Value(v) => v.is_null(),
        }
    }

    pub(super) fn int(&self) -> i32 {
        match self.0 {
            SqliteValueData::Statement { statement, index } => statement.column_int(index),
            SqliteValueData::Value(v) => v.int(),
        }
    }

    pub(super) fn int64(&self) -> i64 {
        match self.0 {
            SqliteValueData::Statement { statement, index } => statement.column_int64(index),
            SqliteValueData::Value(v) => v.int64(),
        }
    }

    pub(super) fn double(&self) -> f64 {
        match self.0 {
            SqliteValueData::Statement { statement, index } => statement.column_double(index),
            SqliteValueData::Value(v) => v.double(),
        }
    }

    pub(super) fn blob(&self) -> &'r [u8] {
        match self.0 {
            SqliteValueData::Statement { statement, index } => statement.column_blob(index),
            SqliteValueData::Value(v) => v.blob(),
        }
    }

    pub(super) fn text(&self) -> Result<&'r str, DecodeError> {
        match self.0 {
            SqliteValueData::Statement { statement, index } => statement.column_text(index),
            SqliteValueData::Value(v) => v.text(),
        }
    }
}

impl<'r> ValueRef<'r> for SqliteValueRef<'r> {
    type Database = Sqlite;

    fn to_owned(&self) -> SqliteValue {
        todo!()
    }

    fn type_info(&self) -> Option<&SqliteTypeInfo> {
        // TODO
        None
    }
}

pub struct SqliteValue(pub(crate) NonNull<sqlite3_value>);

// SAFE: only protected value objects are stored in SqliteValue
unsafe impl Send for SqliteValue {}
unsafe impl Sync for SqliteValue {}

impl SqliteValue {
    pub(crate) unsafe fn new(value: *mut sqlite3_value) -> Self {
        debug_assert!(!value.is_null());
        Self(NonNull::new_unchecked(sqlite3_value_dup(value)))
    }

    fn is_null(&self) -> bool {
        unsafe { sqlite3_value_type(self.0.as_ptr()) == SQLITE_NULL }
    }

    fn int(&self) -> i32 {
        unsafe { sqlite3_value_int(self.0.as_ptr()) }
    }

    fn int64(&self) -> i64 {
        unsafe { sqlite3_value_int64(self.0.as_ptr()) }
    }

    fn double(&self) -> f64 {
        unsafe { sqlite3_value_double(self.0.as_ptr()) }
    }

    fn blob(&self) -> &[u8] {
        let len = unsafe { sqlite3_value_bytes(self.0.as_ptr()) } as usize;

        if len == 0 {
            // empty blobs are NULL so just return an empty slice
            return &[];
        }

        let ptr = unsafe { sqlite3_value_blob(self.0.as_ptr()) } as *const u8;
        debug_assert!(!ptr.is_null());

        unsafe { from_raw_parts(ptr, len) }
    }

    fn text(&self) -> Result<&str, DecodeError> {
        Ok(from_utf8(self.blob())?)
    }
}

impl Value for SqliteValue {
    type Database = Sqlite;

    fn as_ref(&self) -> <Self::Database as HasValueRef>::ValueRef {
        todo!()
    }

    fn type_info(&self) -> Option<&SqliteTypeInfo> {
        todo!()
    }
}
