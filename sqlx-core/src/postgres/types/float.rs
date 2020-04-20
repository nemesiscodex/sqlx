use byteorder::{BigEndian, ByteOrder};

use crate::database::Database;
use crate::decode::{Decode, Error};
use crate::encode::{Encode, IsNull};
use crate::postgres::{PgArguments, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use crate::types::Type;

impl Type<Postgres> for f32 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::FLOAT4
    }
}

impl Encode<'_, Postgres> for f32 {
    fn encode(self, buf: &mut PgArguments) -> IsNull {
        buf.extend(&self.to_be_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for f32 {
    fn decode(value: PgValueRef<'_>) -> Result<Self, Error> {
        Ok(match value.format() {
            PgValueFormat::Binary => BigEndian::read_f32(value.as_bytes()?),
            PgValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}

impl Type<Postgres> for f64 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::FLOAT8
    }
}

impl Encode<'_, Postgres> for f64 {
    fn encode(self, buf: &mut PgArguments) -> IsNull {
        buf.extend(&self.to_be_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for f64 {
    fn decode(value: PgValueRef<'_>) -> Result<Self, Error> {
        Ok(match value.format() {
            PgValueFormat::Binary => BigEndian::read_f64(value.as_bytes()?),
            PgValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}
