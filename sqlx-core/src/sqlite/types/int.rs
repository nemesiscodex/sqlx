use crate::database::HasValueRef;
use crate::decode::{Decode, Error};
use crate::encode::{Encode, IsNull};
use crate::sqlite::{Sqlite, SqliteArgumentValue, SqliteArguments, SqliteTypeInfo, SqliteValueRef};
use crate::types::Type;

impl Type<Sqlite> for i32 {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo::INT
    }
}

impl<'q> Encode<'q, Sqlite> for i32 {
    fn encode(self, args: &mut SqliteArguments<'q>) -> IsNull {
        args.values.push(SqliteArgumentValue::Int(self));

        IsNull::No
    }
}

impl<'r> Decode<'r, Sqlite> for i32 {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, Error> {
        Ok(value.int())
    }
}

impl Type<Sqlite> for i64 {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo::INT64
    }
}

impl<'q> Encode<'q, Sqlite> for i64 {
    fn encode(self, args: &mut SqliteArguments<'q>) -> IsNull {
        args.values.push(SqliteArgumentValue::Int64(self));

        IsNull::No
    }
}

impl<'r> Decode<'r, Sqlite> for i64 {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, Error> {
        Ok(value.int64())
    }
}
