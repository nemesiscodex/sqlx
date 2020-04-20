use std::fmt::{self, Display, Formatter};

use crate::type_info::TypeInfo;

/// Type information for a PostgreSQL type.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SqliteTypeInfo {}

impl SqliteTypeInfo {
    // FIXME: Fill Out
    pub(crate) const BOOL: SqliteTypeInfo = SqliteTypeInfo {};
    pub(crate) const BLOB: SqliteTypeInfo = SqliteTypeInfo {};
    pub(crate) const INT: SqliteTypeInfo = SqliteTypeInfo {};
    pub(crate) const INT64: SqliteTypeInfo = SqliteTypeInfo {};
    pub(crate) const DOUBLE: SqliteTypeInfo = SqliteTypeInfo {};
    pub(crate) const TEXT: SqliteTypeInfo = SqliteTypeInfo {};
}

impl Display for SqliteTypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl TypeInfo for SqliteTypeInfo {}
