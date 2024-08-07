use serde_1 as serde;

use crate::ByteData;

#[cfg_attr(docsrs, doc(cfg(feature = "serde_1")))]
impl serde::ser::Serialize for ByteData<'_> {
    #[inline]
    fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.as_slice())
    }
}

struct ByteDataVisitor;

impl<'de> serde::de::Visitor<'de> for ByteDataVisitor {
    type Value = ByteData<'de>;

    #[inline]
    fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("a byte array")
    }

    #[allow(clippy::min_ident_chars)]
    #[inline]
    fn visit_borrowed_bytes<E: serde::de::Error>(self, v: &'de [u8]) -> Result<Self::Value, E> {
        Ok(ByteData::from_borrowed(v))
    }

    #[allow(clippy::min_ident_chars)]
    #[inline]
    fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
        if v.len() <= crate::ByteChunk::LEN {
            return Ok(ByteData::from_chunk_slice(v));
        }
        #[cfg(feature = "alloc")]
        {
            Ok(ByteData::from_shared(v.into()))
        }
        #[cfg(not(feature = "alloc"))]
        {
            Err(serde::de::Error::custom(
                "the `alloc` or `std` feature is required in `bytedata` for ephemeral byte data",
            ))
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde_1")))]
impl<'de> serde::de::Deserialize<'de> for ByteData<'de> {
    #[inline]
    fn deserialize<D: serde::de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_bytes(ByteDataVisitor)
    }
}
