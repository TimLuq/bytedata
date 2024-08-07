use serde_1 as serde;

use crate::StringData;

#[cfg_attr(docsrs, doc(cfg(feature = "serde_1")))]
impl serde::ser::Serialize for StringData<'_> {
    #[inline]
    fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

struct StringDataVisitor;

impl<'de> serde::de::Visitor<'de> for StringDataVisitor {
    type Value = StringData<'de>;

    #[inline]
    fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("a string")
    }

    #[allow(clippy::min_ident_chars)]
    #[inline]
    fn visit_borrowed_str<E: serde::de::Error>(self, v: &'de str) -> Result<Self::Value, E> {
        Ok(StringData::from_borrowed(v))
    }

    #[allow(clippy::min_ident_chars)]
    #[inline]
    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        if v.len() <= crate::ByteChunk::LEN {
            // SAFETY: `v` is a valid UTF-8 string
            return Ok(unsafe {
                StringData::from_bytedata_unchecked(crate::ByteData::from_chunk_slice(v.as_bytes()))
            });
        }
        #[cfg(feature = "alloc")]
        {
            Ok(StringData::from_borrowed(v).into_shared())
        }
        #[cfg(not(feature = "alloc"))]
        {
            Err(serde::de::Error::custom(
                "the `alloc` or `std` feature is required in `bytedata` for ephemeral string data",
            ))
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde_1")))]
impl<'de> serde::de::Deserialize<'de> for StringData<'de> {
    #[inline]
    fn deserialize<D: serde::de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(StringDataVisitor)
    }
}
