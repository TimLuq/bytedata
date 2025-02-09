use ::serde_1 as serde;

mod bytedata;
mod stringdata;

#[cfg(feature = "alloc")]
mod shared_bytes;

struct OptVisit<V>(V);

impl<
        'de,
        Value,
        V: serde::de::Visitor<'de, Value = Value>
            + serde::de::DeserializeSeed<'de, Value = Value>
            + Copy,
    > serde::de::Visitor<'de> for OptVisit<V>
{
    type Value = Option<Value>;

    #[inline]
    fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("a string or nothing")
    }

    #[inline]
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde_1::de::Error,
    {
        Ok(None)
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde_1::de::Error,
    {
        Ok(None)
    }

    #[inline]
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde_1::de::Deserializer<'de>,
    {
        self.0.deserialize(deserializer).map(Some)
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde_1::de::Error,
    {
        self.0.visit_bool(v).map(Some)
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde_1::de::Error,
    {
        self.0.visit_i8(v).map(Some)
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde_1::de::Error,
    {
        self.0.visit_i16(v).map(Some)
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde_1::de::Error,
    {
        self.0.visit_i32(v).map(Some)
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde_1::de::Error,
    {
        self.0.visit_i64(v).map(Some)
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde_1::de::Error,
    {
        self.0.visit_i128(v).map(Some)
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde_1::de::Error,
    {
        self.0.visit_u8(v).map(Some)
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde_1::de::Error,
    {
        self.0.visit_u16(v).map(Some)
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde_1::de::Error,
    {
        self.0.visit_u32(v).map(Some)
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde_1::de::Error,
    {
        self.0.visit_u64(v).map(Some)
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde_1::de::Error,
    {
        self.0.visit_u128(v).map(Some)
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: serde_1::de::Error,
    {
        self.0.visit_f32(v).map(Some)
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde_1::de::Error,
    {
        self.0.visit_f64(v).map(Some)
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: serde_1::de::Error,
    {
        self.0.visit_char(v).map(Some)
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde_1::de::Error,
    {
        self.0.visit_str(v).map(Some)
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde_1::de::Error,
    {
        self.0.visit_borrowed_str(v).map(Some)
    }

    #[cfg(feature = "alloc")]
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn visit_string<E>(self, v: alloc::string::String) -> Result<Self::Value, E>
    where
        E: serde_1::de::Error,
    {
        self.0.visit_string(v).map(Some)
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde_1::de::Error,
    {
        self.0.visit_bytes(v).map(Some)
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde_1::de::Error,
    {
        self.0.visit_borrowed_bytes(v).map(Some)
    }

    #[inline]
    #[allow(clippy::min_ident_chars)]
    #[cfg(feature = "alloc")]
    fn visit_byte_buf<E>(self, v: alloc::vec::Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde_1::de::Error,
    {
        self.0.visit_byte_buf(v).map(Some)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde_1::Deserializer<'de>,
    {
        self.0.visit_newtype_struct(deserializer).map(Some)
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde_1::de::SeqAccess<'de>,
    {
        self.0.visit_seq(seq).map(Some)
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: serde_1::de::MapAccess<'de>,
    {
        self.0.visit_map(map).map(Some)
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde_1::de::EnumAccess<'de>,
    {
        self.0.visit_enum(data).map(Some)
    }
}

#[cfg(feature = "alloc")]
struct VecVisit<V>(V);

#[cfg(feature = "alloc")]
impl<
        'de,
        Value,
        V: serde::de::Visitor<'de, Value = Value>
            + serde::de::DeserializeSeed<'de, Value = Value>
            + Copy,
    > serde::de::Visitor<'de> for VecVisit<V>
{
    type Value = alloc::vec::Vec<Value>;

    #[inline]
    fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("a sequence of items")
    }

    #[inline]
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde_1::de::SeqAccess<'de>,
    {
        let mut vec = alloc::vec::Vec::new();
        while let Some(value) = seq.next_element_seed(self.0)? {
            vec.push(value);
        }
        Ok(vec)
    }
}
