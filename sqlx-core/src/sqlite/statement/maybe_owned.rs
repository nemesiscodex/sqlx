use std::ops::{Deref, DerefMut};

use crate::sqlite::statement::Statement;

pub(crate) enum MaybeOwnedStatement<'c> {
    Borrowed(&'c mut Statement),
    Owned(Statement),
}

impl Deref for MaybeOwnedStatement<'_> {
    type Target = Statement;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            MaybeOwnedStatement::Borrowed(v) => v,
            MaybeOwnedStatement::Owned(v) => v,
        }
    }
}

impl DerefMut for MaybeOwnedStatement<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            MaybeOwnedStatement::Borrowed(v) => v,
            MaybeOwnedStatement::Owned(v) => v,
        }
    }
}
