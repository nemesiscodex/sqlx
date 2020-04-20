use std::mem;

use crate::database::{Database, HasArguments};
use crate::types::Type;

/// The return type of [Encode::encode].
pub enum IsNull {
    /// The value is null; no data was written.
    Yes,

    /// The value is not null.
    ///
    /// This does not mean that data was written.
    No,
}

/// Encode a single value to be sent to the database.
pub trait Encode<'q, DB: Database>: Type<DB> {
    fn produces(&self) -> DB::TypeInfo {
        Self::type_info()
    }

    /// Writes the value of `self` into `buf` in the expected format for the database.
    #[must_use]
    fn encode(self, buf: &mut <DB as HasArguments<'q>>::Arguments) -> IsNull;

    #[inline]
    fn size_hint(&self) -> usize {
        mem::size_of_val(self)
    }
}

impl<'q, T: Encode<'q, DB>, DB: Database> Encode<'q, DB> for Option<T> {
    #[inline]
    fn produces(&self) -> DB::TypeInfo {
        if let Some(v) = self {
            v.produces()
        } else {
            T::type_info()
        }
    }

    #[inline]
    fn encode(self, buf: &mut <DB as HasArguments<'q>>::Arguments) -> IsNull {
        if let Some(v) = self {
            v.encode(buf);

            IsNull::No
        } else {
            IsNull::Yes
        }
    }

    #[inline]
    fn size_hint(&self) -> usize {
        self.as_ref().map_or(0, Encode::size_hint)
    }
}
