use std::borrow::Cow;

use crate::database::{Database, HasArguments};
use crate::decode::{Decode, Error};
use crate::encode::{Encode, IsNull};
use crate::sqlite::{Sqlite, SqliteArgumentValue, SqliteArguments, SqliteTypeInfo, SqliteValueRef};
use crate::types::Type;

impl Type<Sqlite> for &'_ [u8] {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo::BLOB
    }
}

impl<'q> Encode<'q, Sqlite> for &'q [u8] {
    fn encode(self, args: &mut SqliteArguments<'q>) -> IsNull {
        args.values
            .push(SqliteArgumentValue::Blob(Cow::Borrowed(self)));

        IsNull::No
    }
}

impl<'r> Decode<'r, Sqlite> for &'r [u8] {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, Error> {
        Ok(value.blob())
    }
}

impl Type<Sqlite> for Vec<u8> {
    fn type_info() -> SqliteTypeInfo {
        <&[u8] as Type<Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, Sqlite> for Vec<u8> {
    fn encode(self, args: &mut SqliteArguments<'q>) -> IsNull {
        args.values
            .push(SqliteArgumentValue::Blob(Cow::Owned(self)));

        IsNull::No
    }
}

impl<'r> Decode<'r, Sqlite> for Vec<u8> {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, Error> {
        Ok(value.blob().to_owned())
    }
}
