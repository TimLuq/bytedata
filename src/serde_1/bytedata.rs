use serde_1 as serde;

use crate::ByteData;

#[allow(clippy::multiple_inherent_impl)]
impl ByteData<'static> {
    #[cfg_attr(docsrs, doc(cfg(feature = "serde_1")))]
    /// Deserialize a byte sequence to a shared/owned `ByteData` using `serde`.
    /// 
    /// The normal `Deserialize` implementation for `ByteData` will deserialize to a borrowed `ByteData`.
    /// The borrowed `ByteData` will not be able to outlive a streaming deserialization process.
    /// Using this function in a `Deserialize` implementation will allow the `ByteData` to have the static lifetime.
    /// 
    /// ```rust
    /// # use serde_1::Deserialize;
    /// # use bytedata::ByteData;
    /// #[derive(Deserialize)]
    /// # #[serde(crate = "serde_1")]
    /// struct Owned {
    ///     #[serde(deserialize_with = "ByteData::deserialize_static")]
    ///     a: ByteData<'static>,
    ///     b: u8,
    /// }
    /// ```
    #[inline]
    #[allow(clippy::missing_errors_doc)]
    pub fn deserialize_static<'de, D>(deserializer: D) -> Result<Self, D::Error> where D: serde::de::Deserializer<'de> {
        #[cfg(feature = "alloc")]
        {
            deserializer.deserialize_byte_buf(StaticByteDataVisitor)
        }
        #[cfg(not(feature = "alloc"))]
        {
            deserializer.deserialize_byte(StaticByteDataVisitor)
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde_1")))]
impl serde::ser::Serialize for ByteData<'_> {
    #[inline]
    fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.as_slice())
    }
}

struct StaticByteDataVisitor;

impl serde::de::Visitor<'_> for StaticByteDataVisitor {
    type Value = ByteData<'static>;

    #[inline]
    fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("a byte array")
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

    #[cfg(feature = "alloc")]
    #[allow(clippy::min_ident_chars)]
    #[inline]
    fn visit_byte_buf<E: serde::de::Error>(self, v: alloc::vec::Vec<u8>) -> Result<Self::Value, E> {
        Ok(ByteData::from_owned(v))
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

    #[cfg(feature = "alloc")]
    #[allow(clippy::min_ident_chars)]
    #[inline]
    fn visit_byte_buf<E: serde::de::Error>(self, v: alloc::vec::Vec<u8>) -> Result<Self::Value, E> {
        Ok(ByteData::from_owned(v))
    }
}

/// Deserialize a byte sequence to a borrowed `ByteData` using `serde`.
/// 
/// ```rust
/// # use serde_1::Deserialize;
/// # use bytedata::ByteData;
/// #[derive(Deserialize)]
/// # #[serde(crate = "serde_1")]
/// struct Borrowed<'a> {
///     #[serde(borrow)]
///     a: ByteData<'a>,
///     b: u8,
/// }
/// ```
/// 
/// To deserialize to a shared/owned `ByteData` with the static lifetime, use [`ByteData::deserialize_static`].
#[cfg_attr(docsrs, doc(cfg(feature = "serde_1")))]
impl<'de: 'a, 'a> serde::de::Deserialize<'de> for ByteData<'a> {
    #[inline]
    fn deserialize<D: serde::de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[cfg(feature = "alloc")]
        {
            deserializer.deserialize_byte_buf(ByteDataVisitor)
        }
        #[cfg(not(feature = "alloc"))]
        {
            deserializer.deserialize_byte(ByteDataVisitor)
        }
    }
}
