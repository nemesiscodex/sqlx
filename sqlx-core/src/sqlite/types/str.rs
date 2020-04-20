use std::borrow::Cow;

use crate::database::{Database, HasArguments};
use crate::decode::{Decode, Error};
use crate::encode::{Encode, IsNull};
use crate::sqlite::{Sqlite, SqliteArgumentValue, SqliteArguments, SqliteTypeInfo, SqliteValueRef};
use crate::types::Type;

impl Type<Sqlite> for &'_ str {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo::TEXT
    }
}

impl<'q> Encode<'q, Sqlite> for &'q str {
    fn encode(self, args: &mut SqliteArguments<'q>) -> IsNull {
        args.values
            .push(SqliteArgumentValue::Text(Cow::Borrowed(self)));

        IsNull::No
    }
}

impl<'r> Decode<'r, Sqlite> for &'r str {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, Error> {
        value.text()
    }
}

impl Type<Sqlite> for String {
    fn type_info() -> SqliteTypeInfo {
        <&str as Type<Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, Sqlite> for String {
    fn encode(self, args: &mut SqliteArguments<'q>) -> IsNull {
        args.values
            .push(SqliteArgumentValue::Text(Cow::Owned(self)));

        IsNull::No
    }
}

impl<'r> Decode<'r, Sqlite> for String {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, Error> {
        value.text().map(ToOwned::to_owned)
    }
}
