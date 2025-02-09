use super::shared_bytes_builder::SharedBytesBuilder;

/// A builder for a shared string.
/// This can be thought of as a `String` that can then be frozen into a `StringData`.
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[repr(transparent)]
pub struct SharedStrBuilder {
    inner: SharedBytesBuilder,
}

impl SharedStrBuilder {
    /// Creates a new shared string builder.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inner: SharedBytesBuilder::new(),
        }
    }

    /// Creates a new `SharedStrBuilder` with at least the specified capacity. The maximum capacity is `0xFFFF_FFF0` or `isize::MAX - 15`, whichever is lower.
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: SharedBytesBuilder::with_capacity(capacity),
        }
    }

    /// Appends the given byte to the shared string.
    #[inline]
    pub fn push(&mut self, ch: char) {
        #[allow(clippy::uninit_assumed_init, invalid_value)]
        // Safety: `buf` is a 4-byte buffer, which is enough to encode any Unicode scalar value.
        let mut buf: [u8; 4] = unsafe { core::mem::MaybeUninit::uninit().assume_init() };
        let buf = ch.encode_utf8(&mut buf);
        self.inner.extend_from_slice(buf.as_bytes());
    }

    /// Appends the given bytes to the shared string.
    #[inline]
    pub fn push_str(&mut self, str_data: &str) {
        self.inner.extend_from_slice(str_data.as_bytes());
    }

    /// Reserves capacity for at least `additional` more bytes to be written to the buffer.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    /// Clear the buffer.
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Truncates the buffer to the specified length.
    ///
    /// # Panics
    ///
    /// Panics if `len` is in the middle of a UTF-8 character.
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        if len < self.len() {
            assert!(
                self.inner.as_slice()[len] & 0b1100_0000 != 0b1000_0000,
                "Truncating in the middle of a UTF-8 character"
            );
            self.inner.truncate(len);
        }
    }

    /// Returns `true` if the buffer is empty.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the number of bytes currently written to in the buffer.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns total the number of bytes currently available in the buffer.
    #[inline]
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Gets the bytes currently in the shared string.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        self.inner.as_slice()
    }

    /// Gets the str currently in the shared string.
    #[inline]
    #[must_use]
    pub const fn as_str(&self) -> &str {
        let bytes = self.as_bytes();
        // Safety: `bytes` is a valid UTF-8 string.
        unsafe { core::str::from_utf8_unchecked(bytes) }
    }

    /// Builds the shared string.
    #[inline]
    #[must_use]
    pub fn build(self) -> super::SharedBytes {
        self.inner.build()
    }

    /// Builds the shared string.
    #[inline]
    #[must_use]
    pub fn build_str<'a>(self) -> super::StringData<'a> {
        let bytes = self.inner.into();
        // Safety: `bytes` is a valid UTF-8 string.
        unsafe { super::StringData::from_bytedata_unchecked(bytes) }
    }

    /// Converts the shared string builder into a shared bytes builder.
    #[inline]
    #[must_use]
    pub const fn into_shared_bytes_builder(self) -> SharedBytesBuilder {
        // Safety: The memory layout of `SharedStrBuilder` and `SharedBytesBuilder` is the same.
        unsafe { core::mem::transmute::<Self, SharedBytesBuilder>(self) }
    }
}

impl core::fmt::Debug for SharedStrBuilder {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SharedStrBuilder")
            .field("len", &self.len())
            .field("capacity", &self.capacity())
            .field("as_str", &self.as_str())
            .finish()
    }
}

impl core::fmt::Display for SharedStrBuilder {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self.as_str(), f)
    }
}

impl Default for SharedStrBuilder {
    #[inline]
    #[must_use]
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> From<&'a str> for SharedStrBuilder {
    #[inline]
    fn from(value: &'a str) -> Self {
        let mut builder = Self::with_capacity(value.len());
        builder.push_str(value);
        builder
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl From<SharedStrBuilder> for SharedBytesBuilder {
    #[inline]
    fn from(value: SharedStrBuilder) -> Self {
        value.into_shared_bytes_builder()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl From<SharedStrBuilder> for super::SharedBytes {
    #[inline]
    fn from(value: SharedStrBuilder) -> Self {
        value.inner.build()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl From<SharedStrBuilder> for super::ByteData<'_> {
    #[inline]
    fn from(value: SharedStrBuilder) -> Self {
        value.inner.into()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl From<SharedStrBuilder> for super::StringData<'_> {
    #[inline]
    fn from(value: SharedStrBuilder) -> Self {
        value.build_str()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl TryFrom<SharedBytesBuilder> for SharedStrBuilder {
    type Error = core::str::Utf8Error;
    #[inline]
    fn try_from(value: SharedBytesBuilder) -> Result<Self, Self::Error> {
        match core::str::from_utf8(value.as_slice()) {
            Ok(_) => Ok(Self { inner: value }),
            Err(err) => Err(err),
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl From<super::StringData<'_>> for SharedStrBuilder {
    #[inline]
    fn from(value: super::StringData<'_>) -> Self {
        let bytes = crate::SharedBytes::from(value.into_bytedata());
        Self {
            inner: SharedBytesBuilder::from(bytes),
        }
    }
}

impl core::str::FromStr for SharedStrBuilder {
    type Err = core::convert::Infallible;

    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.into())
    }
}

impl core::fmt::Write for SharedStrBuilder {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.push_str(s);
        Ok(())
    }
}

impl core::borrow::Borrow<str> for SharedStrBuilder {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}
