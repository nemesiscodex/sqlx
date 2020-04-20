use crate::database::Database;
use crate::decode::Decode;
use crate::encode::Encode;
use crate::value::HasRawValue;
use serde_json::value::RawValue as JsonRawValue;
use serde_json::Value as JsonValue;
use std::ops::Deref;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Json<T>(pub T);

impl<T> Deref for Json<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> AsRef<T> for Json<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<DB> Encode<DB> for JsonValue
where
    for<'a> Json<&'a Self>: Encode<DB>,
    DB: Database,
{
    fn encode(&self, buf: &mut DB::RawBuffer) {
        <Json<&Self> as Encode<DB>>::encode(&Json(self), buf)
    }
}

impl<'de, DB> Decode<'de, DB> for JsonValue
where
    Json<Self>: Decode<'de, DB>,
    DB: Database,
{
    fn decode(value: <DB as HasRawValue<'de>>::RawValue) -> crate::Result<Self> {
        <Json<Self> as Decode<DB>>::decode(value).map(|item| item.0)
    }
}

// We don't have to implement Encode for JsonRawValue because that's covered by the default
// implementation for Encode
impl<'de, DB> Decode<'de, DB> for &'de JsonRawValue
where
    Json<Self>: Decode<'de, DB>,
    DB: Database,
{
    fn decode(value: <DB as HasRawValue<'de>>::RawValue) -> crate::Result<Self> {
        <Json<Self> as Decode<DB>>::decode(value).map(|item| item.0)
    }
}
