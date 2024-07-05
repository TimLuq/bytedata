use serde_1 as serde;

use crate::StringData;

#[cfg_attr(docsrs, doc(cfg(feature = "serde_1")))]
impl serde::ser::Serialize for StringData<'_> {
    fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde_1")))]
impl<'de> serde::de::Deserialize<'de> for StringData<'de> {
    fn deserialize<D: serde::de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct StringDataVisitor;

        impl<'de> serde::de::Visitor<'de> for StringDataVisitor {
            type Value = StringData<'de>;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_borrowed_str<E: serde::de::Error>(
                self,
                v: &'de str,
            ) -> Result<Self::Value, E> {
                Ok(StringData::from_borrowed(v))
            }

            fn visit_str<E: serde::de::Error>(self, _v: &str) -> Result<Self::Value, E> {
                #[cfg(feature = "chunk")]
                if _v.len() <= 12 {
                    return Ok(unsafe { StringData::from_bytedata_unchecked(crate::ByteData::from_chunk_slice(_v.as_bytes())) });
                }
                #[cfg(feature = "alloc")]
                {
                    Ok(StringData::from_borrowed(_v).into_shared())
                }
                #[cfg(not(feature = "alloc"))]
                {
                    Err(serde::de::Error::custom("the `alloc` or `std` feature is required in `stringdata` for ephemeral string data"))
                }
            }
        }

        deserializer.deserialize_str(StringDataVisitor)
    }
}
