use crate::decode::{Decode, Error};
use crate::encode::{Encode, IsNull};
use crate::sqlite::{Sqlite, SqliteArgumentValue, SqliteArguments, SqliteTypeInfo, SqliteValueRef};
use crate::types::Type;

impl Type<Sqlite> for bool {
    #[inline]
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo::BOOL
    }
}

impl<'q> Encode<'q, Sqlite> for bool {
    #[inline]
    fn encode(self, args: &mut SqliteArguments<'q>) -> IsNull {
        args.values.push(SqliteArgumentValue::Int(self.into()));

        IsNull::No
    }
}

impl<'r> Decode<'r, Sqlite> for bool {
    #[inline]
    fn decode(value: SqliteValueRef<'r>) -> Result<bool, Error> {
        Ok(value.int() != 0)
    }
}
