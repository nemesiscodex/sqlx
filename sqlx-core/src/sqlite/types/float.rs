use crate::decode::{Decode, Error};
use crate::encode::{Encode, IsNull};
use crate::sqlite::{Sqlite, SqliteArgumentValue, SqliteArguments, SqliteTypeInfo, SqliteValueRef};
use crate::types::Type;

impl Type<Sqlite> for f32 {
    #[inline]
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo::DOUBLE
    }
}

impl<'q> Encode<'q, Sqlite> for f32 {
    #[inline]
    fn encode(self, args: &mut SqliteArguments<'q>) -> IsNull {
        args.values.push(SqliteArgumentValue::Double(self.into()));

        IsNull::No
    }
}

impl<'r> Decode<'r, Sqlite> for f32 {
    #[inline]
    fn decode(value: SqliteValueRef<'r>) -> Result<f32, Error> {
        Ok(value.double() as f32)
    }
}

impl Type<Sqlite> for f64 {
    #[inline]
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo::DOUBLE
    }
}

impl<'q> Encode<'q, Sqlite> for f64 {
    #[inline]
    fn encode(self, args: &mut SqliteArguments<'q>) -> IsNull {
        args.values.push(SqliteArgumentValue::Double(self));

        IsNull::No
    }
}

impl<'r> Decode<'r, Sqlite> for f64 {
    #[inline]
    fn decode(value: SqliteValueRef<'r>) -> Result<f64, Error> {
        Ok(value.double())
    }
}
