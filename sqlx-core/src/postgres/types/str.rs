use crate::decode::{Decode, Error};
use crate::encode::{Encode, IsNull};
use crate::postgres::{PgArguments, PgTypeInfo, PgValueRef, Postgres};
use crate::types::Type;

impl Type<Postgres> for &'_ str {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::TEXT
    }
}

impl Encode<'_, Postgres> for &'_ str {
    fn encode(self, buf: &mut PgArguments) -> IsNull {
        buf.extend(self.as_bytes());

        IsNull::No
    }
}

impl Encode<'_, Postgres> for String {
    fn encode(self, buf: &mut PgArguments) -> IsNull {
        <&str as Encode<Postgres>>::encode(&self, buf)
    }
}

impl<'r> Decode<'r, Postgres> for &'r str {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Error> {
        Ok(value.as_str()?)
    }
}

impl Type<Postgres> for String {
    fn type_info() -> PgTypeInfo {
        <&str as Type<Postgres>>::type_info()
    }
}

impl Decode<'_, Postgres> for String {
    fn decode(value: PgValueRef<'_>) -> Result<Self, Error> {
        Ok(value.as_str()?.to_owned())
    }
}
