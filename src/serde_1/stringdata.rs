use serde_1 as serde;

use crate::StringData;

#[allow(clippy::multiple_inherent_impl)]
impl StringData<'static> {
    #[cfg_attr(docsrs, doc(cfg(feature = "serde_1")))]
    /// Deserialize a string to a shared/owned `StringData` using `serde`.
    ///
    /// The normal `Deserialize` implementation for `StringData` will deserialize to a borrowed `StringData`.
    /// The borrowed `StringData` will not be able to outlive a streaming deserialization process.
    /// Using this function in a `Deserialize` implementation will allow the `StringData` to have the static lifetime.
    ///
    /// ```rust
    /// # use serde_1::Deserialize;
    /// # use bytedata::StringData;
    /// #[derive(Deserialize)]
    /// # #[serde(crate = "serde_1")]
    /// struct Owned {
    ///     #[serde(deserialize_with = "StringData::deserialize_static")]
    ///     a: StringData<'static>,
    ///     b: u8,
    /// }
    /// ```
    #[inline]
    #[allow(clippy::missing_errors_doc)]
    pub fn deserialize_static<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        #[cfg(feature = "alloc")]
        {
            deserializer.deserialize_string(StaticStringDataVisitor)
        }
        #[cfg(not(feature = "alloc"))]
        {
            deserializer.deserialize_str(StaticStringDataVisitor)
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde_1")))]
impl serde::ser::Serialize for StringData<'_> {
    #[inline]
    fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

struct StaticStringDataVisitor;

impl serde::de::Visitor<'_> for StaticStringDataVisitor {
    type Value = StringData<'static>;

    #[inline]
    fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("a string")
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

    #[cfg(feature = "alloc")]
    #[allow(clippy::min_ident_chars)]
    #[inline]
    fn visit_string<E: serde::de::Error>(self, v: alloc::string::String) -> Result<Self::Value, E> {
        Ok(StringData::from_owned(v))
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

    #[cfg(feature = "alloc")]
    #[allow(clippy::min_ident_chars)]
    #[inline]
    fn visit_string<E: serde::de::Error>(self, v: alloc::string::String) -> Result<Self::Value, E> {
        Ok(StringData::from_owned(v))
    }
}

/// Deserialize a string to a borrowed `StringData` using `serde`.
///
/// ```rust
/// # use serde_1::Deserialize;
/// # use bytedata::StringData;
/// #[derive(Deserialize)]
/// # #[serde(crate = "serde_1")]
/// struct Borrowed<'a> {
///     #[serde(borrow)]
///     a: StringData<'a>,
///     b: u8,
/// }
/// ```
///
/// To deserialize to a shared/owned `StringData` with the static lifetime, use [`StringData::deserialize_static`].
#[cfg_attr(docsrs, doc(cfg(feature = "serde_1")))]
impl<'de: 'a, 'a> serde::de::Deserialize<'de> for StringData<'a> {
    #[inline]
    fn deserialize<D: serde::de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[cfg(feature = "alloc")]
        {
            deserializer.deserialize_string(StringDataVisitor)
        }
        #[cfg(not(feature = "alloc"))]
        {
            deserializer.deserialize_str(StringDataVisitor)
        }
    }
}
