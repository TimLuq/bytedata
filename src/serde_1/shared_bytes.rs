use serde_1 as serde;

use crate::SharedBytes;

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg_attr(docsrs, doc(cfg(feature = "serde_1")))]
impl serde::ser::Serialize for SharedBytes {
    #[inline]
    fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.as_slice())
    }
}

struct ByteDataVisitor;

impl serde::de::Visitor<'_> for ByteDataVisitor {
    type Value = SharedBytes;

    #[inline]
    fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("a byte array")
    }

    #[allow(clippy::min_ident_chars)]
    #[inline]
    fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
        Ok(v.into())
    }

    #[allow(clippy::min_ident_chars)]
    #[inline]
    fn visit_byte_buf<E>(self, v: alloc::vec::Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.into())
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg_attr(docsrs, doc(cfg(feature = "serde_1")))]
impl<'de> serde::de::Deserialize<'de> for SharedBytes {
    #[inline]
    fn deserialize<D: serde::de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_bytes(ByteDataVisitor)
    }
}
