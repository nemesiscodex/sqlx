use byteorder::{BigEndian, ByteOrder};

use crate::database::Database;
use crate::decode::{Decode, Error};
use crate::encode::{Encode, IsNull};
use crate::postgres::{PgArguments, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use crate::types::Type;

impl Type<Postgres> for i8 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::CHAR
    }
}

impl Encode<'_, Postgres> for i8 {
    fn encode(self, buf: &mut PgArguments) -> IsNull {
        buf.extend(&self.to_be_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for i8 {
    fn decode(value: PgValueRef<'_>) -> Result<Self, Error> {
        // note: in the TEXT encoding, a value of "0" here is encoded as an empty string
        Ok(value.as_bytes()?.get(0).copied().unwrap_or_default() as i8)
    }
}

impl Type<Postgres> for i16 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::INT2
    }
}

impl Encode<'_, Postgres> for i16 {
    fn encode(self, buf: &mut PgArguments) -> IsNull {
        buf.extend(&self.to_be_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for i16 {
    fn decode(value: PgValueRef<'_>) -> Result<Self, Error> {
        Ok(match value.format() {
            PgValueFormat::Binary => BigEndian::read_i16(value.as_bytes()?),
            PgValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}

impl Type<Postgres> for u32 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::OID
    }
}

impl Encode<'_, Postgres> for u32 {
    fn encode(self, buf: &mut PgArguments) -> IsNull {
        buf.extend(&self.to_be_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for u32 {
    fn decode(value: PgValueRef<'_>) -> Result<Self, Error> {
        Ok(match value.format() {
            PgValueFormat::Binary => BigEndian::read_u32(value.as_bytes()?),
            PgValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}

impl Type<Postgres> for i32 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::INT4
    }
}

impl Encode<'_, Postgres> for i32 {
    fn encode(self, buf: &mut PgArguments) -> IsNull {
        buf.extend(&self.to_be_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for i32 {
    fn decode(value: PgValueRef<'_>) -> Result<Self, Error> {
        Ok(match value.format() {
            PgValueFormat::Binary => BigEndian::read_i32(value.as_bytes()?),
            PgValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}

impl Type<Postgres> for i64 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::INT8
    }
}

impl Encode<'_, Postgres> for i64 {
    fn encode(self, buf: &mut PgArguments) -> IsNull {
        buf.extend(&self.to_be_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for i64 {
    fn decode(value: PgValueRef<'_>) -> Result<Self, Error> {
        Ok(match value.format() {
            PgValueFormat::Binary => BigEndian::read_i64(value.as_bytes()?),
            PgValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}
