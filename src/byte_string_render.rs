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
    #[must_use]
    pub const fn as_slice(&self) -> &'a [u8] {
        self.0
    }

    /// Create a new `ByteStringRender` from a byte slice.
    #[inline]
    #[must_use]
    pub const fn from_slice(slice: &'a [u8]) -> Self {
        Self(slice)
    }

    /// Create a new `ByteStringRender` from a string slice.
    #[inline]
    #[must_use]
    pub const fn from_str(slice: &'a str) -> Self {
        Self(slice.as_bytes())
    }

    /// Create a new `ByteStringRender` from a byte slice reference.
    #[inline]
    #[must_use]
    pub fn from_ref<B: AsRef<[u8]>>(slice: &'a B) -> Self {
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

impl core::fmt::Debug for ByteStringRender<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("b\"")?;
        core::fmt::Display::fmt(self, f)?;
        f.write_str("\"")
    }
}

impl core::fmt::Display for ByteStringRender<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for &by in self.0 {
            match by {
                b'\\' => f.write_str("\\\\")?,
                b'"' => f.write_str("\\\"")?,
                b'\n' => f.write_str("\\n")?,
                b'\r' => f.write_str("\\r")?,
                b'\t' => f.write_str("\\t")?,
                by if (32..127).contains(&by) => {
                    let by = [by];
                    // SAFETY: `by` is ASCII-7.
                    let dat = unsafe { core::str::from_utf8_unchecked(&by) };
                    f.write_str(dat)?;
                }
                by => f.write_fmt(format_args!("\\x{by:02x}"))?,
            }
        }
        Ok(())
    }
}

#[allow(clippy::redundant_pub_crate)]
pub(crate) fn lower_hex_slice(sl: &[u8], fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    if let Some(w) = fmt.width() {
        let mul = sl.len() << 1_u8;
        if w > mul {
            for _ in 0..w - mul {
                core::fmt::Write::write_str(fmt, "0")?;
            }
        }
    }
    let mut i = 0;
    while i < sl.len() {
        write!(fmt, "{:02x}", sl[i])?;
        i += 1;
    }
    Ok(())
}

impl core::fmt::LowerHex for ByteStringRender<'_> {
    #[allow(clippy::min_ident_chars)]
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        lower_hex_slice(self.as_slice(), f)
    }
}

#[allow(clippy::redundant_pub_crate)]
pub(crate) fn upper_hex_slice(sl: &[u8], fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    if let Some(w) = fmt.width() {
        let mul = sl.len() << 1_u8;
        if w > mul {
            for _ in 0..w - mul {
                core::fmt::Write::write_str(fmt, "0")?;
            }
        }
    }
    let mut i = 0;
    while i < sl.len() {
        write!(fmt, "{:02X}", sl[i])?;
        i += 1;
    }
    Ok(())
}

impl core::fmt::UpperHex for ByteStringRender<'_> {
    #[allow(clippy::min_ident_chars)]
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        upper_hex_slice(self.as_slice(), f)
    }
}

/// Helper wrapper to render a collection of byte slices as a singular bytestring similar to [`core::ascii::escape_default`].
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
    #[inline]
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
    #[inline]
    #[allow(clippy::min_ident_chars)]
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
    #[allow(clippy::min_ident_chars)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for by in self.inner {
            let rend = ByteStringRender::from_slice(by.as_ref());
            core::fmt::Display::fmt(&rend, f)?;
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
        {
            let data = b"Hello, World!";
            let rendered = ByteStringRender::from_slice(data);
            assert_eq!(alloc::format!("{}", &rendered), "Hello, World!");
            assert_eq!(alloc::format!("{:?}", &rendered), r#"b"Hello, World!""#);
        };

        {
            let data = b"Hello, \nWorld!";
            let rendered = ByteStringRender::from_slice(data);
            assert_eq!(alloc::format!("{}", &rendered), r"Hello, \nWorld!");
            assert_eq!(alloc::format!("{:?}", &rendered), r#"b"Hello, \nWorld!""#);
        };
    }
}
