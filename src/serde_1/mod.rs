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
