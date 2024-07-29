/// Helper wrapper to render a byte slice as a bytestring similar to [`core::ascii::escape_default`].
///
#[cfg_attr(feature = "alloc", doc = "```")]
#[cfg_attr(not(feature = "alloc"), doc = "```ignore")]
/// # extern crate alloc;
/// # use alloc::format;
/// # use bytedata::ByteStringRender;
/// format!("{}", ByteStringRender::from_slice(b"Hello,\nWorld!"));
/// // => "Hello,\\nWorld!"
/// format!("{:?}", ByteStringRender::from_slice(b"Hello,\nWorld!"));
/// // => "b\"Hello,\\nWorld!\""
/// ```
#[repr(transparent)]
pub struct ByteStringRender<'a>(&'a [u8]);

impl<'a> ByteStringRender<'a> {
    /// Get the inner byte slice.
    #[inline]
    pub const fn as_slice(&self) -> &'a [u8] {
        self.0
    }

    /// Create a new `ByteStringRender` from a byte slice.
    pub const fn from_slice(slice: &'a [u8]) -> Self {
        Self(slice)
    }

    /// Create a new `ByteStringRender` from a string slice.
    pub const fn from_str(slice: &'a str) -> Self {
        Self(slice.as_bytes())
    }

    /// Create a new `ByteStringRender` from a byte slice reference.
    #[inline]
    pub fn from_ref(slice: &'a impl AsRef<[u8]>) -> Self {
        Self(slice.as_ref())
    }
}

impl<'a> From<&'a [u8]> for ByteStringRender<'a> {
    #[inline]
    fn from(value: &'a [u8]) -> Self {
        Self(value)
    }
}

impl<'a> From<&'a str> for ByteStringRender<'a> {
    #[inline]
    fn from(value: &'a str) -> Self {
        Self(value.as_bytes())
    }
}

impl<'a> core::fmt::Debug for ByteStringRender<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("b\"")?;
        core::fmt::Display::fmt(self, f)?;
        f.write_str("\"")
    }
}

impl<'a> core::fmt::Display for ByteStringRender<'a> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for &b in self.0 {
            match b {
                b'\\' => f.write_str("\\\\")?,
                b'"' => f.write_str("\\\"")?,
                b'\n' => f.write_str("\\n")?,
                b'\r' => f.write_str("\\r")?,
                b'\t' => f.write_str("\\t")?,
                b if (32..127).contains(&b) => {
                    f.write_str(unsafe { core::str::from_utf8_unchecked(&[b]) })?
                }
                b => f.write_fmt(format_args!("\\x{:02x}", b))?,
            }
        }
        Ok(())
    }
}

/// Helper wrapper to render a byte slice as a bytestring similar to [`core::ascii::escape_default`].
///
#[cfg_attr(feature = "alloc", doc = "```")]
#[cfg_attr(not(feature = "alloc"), doc = "```ignore")]
/// # extern crate alloc;
/// # use alloc::format;
/// # use bytedata::MultiByteStringRender;
/// format!("{}", MultiByteStringRender::new(&[b"Hello,\n".as_slice(), b"World!"]));
/// // => "Hello,\\nWorld!"
/// format!("{:?}", MultiByteStringRender::new(&[b"Hello,\n".as_slice(), b"World!"]));
/// // => "b\"Hello,\\nWorld!\""
/// ```
pub struct MultiByteStringRender<'a, T, R> {
    inner: &'a T,
    _phantom: core::marker::PhantomData<R>,
}

impl<'a, T, R: AsRef<[u8]>> MultiByteStringRender<'a, T, R>
where
    &'a T: IntoIterator<Item = R>,
{
    /// Create a new `MultiByteStringRender` from an iterator over byte slices.
    pub const fn new(inner: &'a T) -> Self {
        Self {
            inner,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<'a, T, R: AsRef<[u8]>> core::fmt::Debug for MultiByteStringRender<'a, T, R>
where
    &'a T: IntoIterator<Item = R>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("b\"")?;
        core::fmt::Display::fmt(self, f)?;
        f.write_str("\"")
    }
}

impl<'a, T, R: AsRef<[u8]>> core::fmt::Display for MultiByteStringRender<'a, T, R>
where
    &'a T: IntoIterator<Item = R>,
{
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for b in self.inner.into_iter() {
            let r = ByteStringRender::from_slice(b.as_ref());
            core::fmt::Display::fmt(&r, f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "alloc")]
mod tests {
    use super::*;

    #[test]
    fn test_byte_string_render() {
        let data = b"Hello, World!";
        let rendered = ByteStringRender::from_slice(data);
        assert_eq!(alloc::format!("{}", &rendered), r#"Hello, World!"#);
        assert_eq!(alloc::format!("{:?}", &rendered), r#"b"Hello, World!""#);

        let data = b"Hello, \nWorld!";
        let rendered = ByteStringRender::from_slice(data);
        assert_eq!(alloc::format!("{}", &rendered), r#"Hello, \nWorld!"#);
        assert_eq!(alloc::format!("{:?}", &rendered), r#"b"Hello, \nWorld!""#);
    }
}
