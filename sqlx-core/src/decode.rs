use std::error::Error as StdError;

use crate::database::{Database, HasValueRef};
use crate::types::Type;
use crate::value::Value;

// alias as this is a long type that's used everywhere
pub(crate) type Error = Box<dyn StdError + 'static + Send + Sync>;

/// A type that can be decoded from the database.
pub trait Decode<'r, DB: Database>: Sized + Type<DB> {
    /// Determines if a value of this type can be created from a value with the
    /// given type information.
    fn accepts(info: &DB::TypeInfo) -> bool {
        *info == Self::type_info()
    }

    /// Decode a new value of this type using a raw value from the database.
    fn decode(value: <DB as HasValueRef<'r>>::ValueRef) -> Result<Self, Error>;
}

/// An unexpected `NULL` was encountered during decoding.
///
/// Returned from [`Row::get`] if the value from the database is `NULL`,
/// and you are not decoding into an `Option`.
#[derive(thiserror::Error, Debug)]
#[error("unexpected null; try decoding as an `Option`")]
pub struct UnexpectedNullError;
