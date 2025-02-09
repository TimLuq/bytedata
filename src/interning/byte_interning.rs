use crate::{ByteData, StringData};

/// Trait for interning byte data.
pub trait ByteInterning<'a> {
    /// Interns the given byte data using the rules for the interning implementation.
    ///
    /// The byte content returned in the `Ok` variant are guaranteed to be the same as the bytes in the input.
    #[allow(clippy::missing_errors_doc)]
    fn intern<'b>(&self, value: ByteData<'b>) -> Result<ByteData<'a>, ByteData<'b>>;
    /// Interns the given string data using the rules for the interning implementation.
    #[inline]
    #[allow(clippy::missing_errors_doc)]
    fn intern_str<'b>(&self, value: StringData<'b>) -> Result<StringData<'a>, StringData<'b>> {
        match self.intern(value.into_bytedata()) {
            // SAFETY: The implementation of `intern` is expected to always return the same byte data.
            Ok(value) => Ok(unsafe { StringData::from_bytedata_unchecked(value) }),
            // SAFETY: The implementation of `intern` is expected to always return the same byte data.
            Err(value) => Err(unsafe { StringData::from_bytedata_unchecked(value) }),
        }
    }

    /// Forcefully interns the given byte data, ignoring any rules for the interning implementation.
    ///
    /// The byte content returned in the `Ok` variant are guaranteed to be the same as the bytes in the input.
    #[allow(single_use_lifetimes)]
    fn intern_always<'b>(&self, value: ByteData<'b>) -> ByteData<'a>;
    /// Interns the given string data using the rules for the interning implementation.
    #[inline]
    #[allow(single_use_lifetimes)]
    fn intern_always_str<'b>(&self, value: StringData<'b>) -> StringData<'a> {
        // SAFETY: The implementation of `intern_always` is expected to always return the same byte data.
        unsafe { StringData::from_bytedata_unchecked(self.intern_always(value.into_bytedata())) }
    }

    /// Gets the interned value for the given byte data if it exists, otherwise returns the given value.
    ///
    /// The byte content returned in the `Ok` variant are guaranteed to be the same as the bytes in the input.
    #[allow(clippy::missing_errors_doc)]
    fn get<'b>(&self, value: ByteData<'b>) -> Result<ByteData<'a>, ByteData<'b>>;
    /// Gets the interned value for the given string data if it exists, otherwise returns the given value.
    #[inline]
    #[allow(clippy::missing_errors_doc)]
    fn get_str<'b>(&self, value: StringData<'b>) -> Result<StringData<'a>, StringData<'b>> {
        match self.get(value.into_bytedata()) {
            // SAFETY: The implementation of `get` is expected to always return the same byte data.
            Ok(value) => Ok(unsafe { StringData::from_bytedata_unchecked(value) }),
            // SAFETY: The implementation of `get` is expected to always return the same byte data.
            Err(value) => Err(unsafe { StringData::from_bytedata_unchecked(value) }),
        }
    }
}
