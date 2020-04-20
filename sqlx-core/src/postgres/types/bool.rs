use crate::database::Database;
use crate::decode::{Decode, Error};
use crate::encode::{Encode, IsNull};
use crate::postgres::{PgArguments, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use crate::types::Type;

impl Type<Postgres> for bool {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::BOOL
    }
}

impl Encode<'_, Postgres> for bool {
    fn encode(self, buf: &mut PgArguments) -> IsNull {
        buf.push(self as u8);
        IsNull::No
    }
}

impl Decode<'_, Postgres> for bool {
    fn decode(value: PgValueRef<'_>) -> Result<Self, Error> {
        Ok(match value.format() {
            PgValueFormat::Binary => value.as_bytes()?[0] != 0,

            PgValueFormat::Text => match value.as_str()? {
                "t" => true,
                "f" => false,

                s => {
                    return Err(format!("unexpected value {:?} for boolean", s).into());
                }
            },
        })
    }
}
