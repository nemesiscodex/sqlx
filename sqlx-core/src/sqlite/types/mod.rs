//! Conversions between Rust and **SQLite** types.
//!
//! # Types
//!
//! | Rust type                             | SQLite type(s)                                       |
//! |---------------------------------------|------------------------------------------------------|
//! | `bool`                                | BOOLEAN                                              |
//! | `i16`                                 | INTEGER                                              |
//! | `i32`                                 | INTEGER                                              |
//! | `i64`                                 | INTEGER                                              |
//! | `f32`                                 | REAL                                                 |
//! | `f64`                                 | REAL                                                 |
//! | `&str`, `String`                      | TEXT                                                 |
//! | `&[u8]`, `Vec<u8>`                    | BLOB                                                 |
//!
//! # Nullable
//!
//! In addition, `Option<T>` is supported where `T` implements `Type`. An `Option<T>` represents
//! a potentially `NULL` value from SQLite.
//!

use crate::decode::{Decode, Error};
use crate::sqlite::{Sqlite, SqliteTypeInfo, SqliteValueRef};
use crate::types::Type;

mod bool;
mod bytes;
mod float;
mod int;
mod str;

impl<'r, T> Decode<'r, Sqlite> for Option<T>
where
    T: Decode<'r, Sqlite>,
{
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, Error> {
        if value.is_null() {
            Ok(None)
        } else {
            Ok(Some(T::decode(value)?))
        }
    }
}
