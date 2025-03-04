use core::{
    ops::{Bound, Deref, RangeBounds},
    slice::SliceIndex,
};

#[cfg(feature = "alloc")]
use alloc::{borrow::Cow, string::String, vec::Vec};

use crate::ByteData;

#[cfg(feature = "alloc")]
use crate::SharedBytes;

/// A wrapper around a [`ByteData`] that is guaranteed to be valid UTF-8.
///
/// `StringData<'a>` is to `ByteData<'a>` what `&'a str` is to `&'a [u8]`.
#[derive(Clone)]
#[repr(transparent)]
pub struct StringData<'a> {
    data: ByteData<'a>,
}

impl<'a> StringData<'a> {
    /// Returns an empty `StringData`.
    #[inline]
    #[must_use]
    pub const fn empty() -> Self {
        StringData {
            data: ByteData::empty(),
        }
    }

    /// Creates a `StringData` from a static str.
    #[inline]
    #[must_use]
    pub const fn from_static(dat: &'static str) -> Self {
        StringData {
            data: ByteData::from_static(dat.as_bytes()),
        }
    }

    /// Creates a `StringData` from a borrowed str.
    #[inline]
    #[must_use]
    pub const fn from_borrowed(dat: &'a str) -> Self {
        StringData {
            data: ByteData::from_borrowed(dat.as_bytes()),
        }
    }

    #[cfg(feature = "alloc")]
    /// Creates a `StringData` from a slice of borrowed `str`s.
    #[inline]
    #[must_use]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn from_concat(dat: &[&'a str]) -> Self {
        fn from_concat_inner<'a>(dat: &[&'a str]) -> StringData<'a> {
            let len = dat.iter().map(|x| x.len()).sum();
            if len <= crate::ByteChunk::LEN {
                let mut buf = [0_u8; crate::ByteChunk::LEN];
                let mut at = 0;
                for x in dat {
                    let by = x.as_bytes();
                    buf[at..at + by.len()].copy_from_slice(by);
                    at += by.len();
                }
                let data = crate::ByteData::from_chunk_slice(&buf[..at]);
                // SAFETY: `ByteData` is guaranteed to be valid UTF-8, if the input is valid UTF-8.
                return unsafe { StringData::from_bytedata_unchecked(data) };
            }
            let mut buf = crate::SharedStrBuilder::with_capacity(len);
            for x in dat {
                buf.push_str(x);
            }
            buf.build_str()
        }
        if dat.is_empty() {
            return Self::empty();
        }
        if dat.len() == 1 {
            return Self::from_borrowed(dat[0]);
        }
        from_concat_inner(dat)
    }

    #[cfg(feature = "alloc")]
    /// Creates a `StringData` from a `SharedBytes`.
    ///
    /// # Errors
    ///
    /// Returns the input if the data is not valid UTF-8.
    #[inline]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub const fn try_from_shared(dat: SharedBytes) -> Result<Self, SharedBytes> {
        if core::str::from_utf8(dat.as_slice()).is_err() {
            return Err(dat);
        }
        Ok(StringData {
            data: ByteData::from_shared(dat),
        })
    }

    /// Creates a `StringData` from `ByteData`.
    ///
    /// # Errors
    ///
    /// Returns the input if the data is not valid UTF-8.
    #[inline]
    pub const fn try_from_bytedata(dat: ByteData<'a>) -> Result<Self, ByteData<'a>> {
        if core::str::from_utf8(dat.as_slice()).is_err() {
            return Err(dat);
        }
        Ok(StringData { data: dat })
    }

    /// Creates a `StringData` from `ByteData`.
    ///
    /// # Safety
    ///
    /// The data must be valid UTF-8.
    /// Otherwise, the behavior is undefined for any context using the value.
    /// Prefer [`StringData::try_from_bytedata`] if you are unsure.
    #[inline]
    #[must_use]
    pub const unsafe fn from_bytedata_unchecked(dat: ByteData<'a>) -> Self {
        StringData { data: dat }
    }

    /// Returns the underlying [`ByteData`].
    #[inline]
    #[must_use]
    pub const fn into_bytedata(self) -> ByteData<'a> {
        // SAFETY: `StringData` is a transparent wrapper around `ByteData`.
        unsafe { core::mem::transmute(self) }
    }

    /// Returns a reference to the underlying [`ByteData`].
    #[inline]
    #[must_use]
    pub const fn as_bytedata(&self) -> &ByteData<'a> {
        &self.data
    }

    #[cfg(feature = "alloc")]
    /// Creates a `StringData` from a `String`.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[must_use]
    #[inline]
    pub fn from_owned(dat: String) -> Self {
        StringData { data: dat.into() }
    }

    #[cfg(feature = "alloc")]
    /// Creates a `StringData` from a `Cow<'_, str>`.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[must_use]
    #[inline]
    pub fn from_cow(dat: Cow<'a, str>) -> Self {
        match dat {
            Cow::Borrowed(borr) => Self::from_borrowed(borr),
            Cow::Owned(ow) => Self::from_owned(ow),
        }
    }

    #[cfg(feature = "alloc")]
    /// Creates a `StringData` from a `Cow<'static, str>`.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[must_use]
    #[inline]
    pub fn from_cow_static(dat: Cow<'static, str>) -> Self {
        match dat {
            Cow::Borrowed(borr) => Self::from_static(borr),
            Cow::Owned(ow) => Self::from_owned(ow),
        }
    }

    /// Returns the underlying byte slice.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        self.data.as_slice()
    }

    /// Returns the underlying string if it is borrowed or static
    #[inline]
    #[must_use]
    pub const fn as_borrowed(&self) -> Option<&'a str> {
        match self.data.as_borrowed() {
            // SAFETY: `StringData` is guaranteed to be valid UTF-8, unless the user has used `unsafe` methods.
            Some(buf) => Some(unsafe { core::str::from_utf8_unchecked(buf) }),
            None => None,
        }
    }

    /// Returns the underlying `str`.
    #[inline]
    #[must_use]
    pub const fn as_str(&self) -> &str {
        // SAFETY: `StringData` is guaranteed to be valid UTF-8, unless the user has used `unsafe` methods.
        unsafe { core::str::from_utf8_unchecked(self.data.as_slice()) }
    }

    /// Returns the length of the underlying byte slice.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the underlying byte slice is empty.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Check if the underlying byte slice is equal to another. This can be used in a `const` context.
    #[inline]
    #[must_use]
    pub const fn eq_const(&self, other: &StringData<'_>) -> bool {
        crate::const_eq(self.as_bytes(), other.as_bytes())
    }

    /// Check if the underlying byte slice is equal to another. This can be used in a `const` context.
    #[inline]
    #[must_use]
    pub const fn eq_slice(&self, other: &[u8]) -> bool {
        crate::const_eq(self.as_bytes(), other)
    }

    /// Check if the underlying byte slice is equal to another. This can be used in a `const` context.
    #[inline]
    #[must_use]
    pub const fn eq_str(&self, other: &str) -> bool {
        crate::const_eq(self.as_bytes(), other.as_bytes())
    }

    /// Check if the ending of a `StringData` matches the given str.
    #[inline]
    #[must_use]
    pub const fn ends_with(&self, needle: &str) -> bool {
        crate::const_ends_with(self.as_bytes(), needle.as_bytes())
    }

    /// Check if the beginning of a `StringData` matches the given str.
    #[inline]
    #[must_use]
    pub const fn starts_with(&self, needle: &str) -> bool {
        crate::const_starts_with(self.as_bytes(), needle.as_bytes())
    }

    /// Trim whitespace from the beginning and end of the `StringData`.
    #[inline]
    #[must_use]
    pub fn trim(&self) -> Self {
        let at = self.as_str().trim();
        let start = at.as_ptr() as usize - self.as_str().as_ptr() as usize;
        let end = start + at.len();
        self.sliced(start..end)
    }

    /// Trim whitespace from the beginning of the `StringData`.
    #[inline]
    #[must_use]
    pub fn trim_start(&self) -> Self {
        let at = self.as_str().trim_start();
        let start = at.as_ptr() as usize - self.as_str().as_ptr() as usize;
        let end = start + at.len();
        self.sliced(start..end)
    }

    /// Trim whitespace from the end of the `StringData`.
    #[inline]
    #[must_use]
    pub fn trim_end(&self) -> Self {
        let at = self.as_str().trim_end();
        self.sliced(0..at.len())
    }

    fn check_sliced<R: RangeBounds<usize> + SliceIndex<str, Output = str>>(
        &self,
        range: R,
    ) -> core::ops::Range<usize> {
        let by = self.data.as_slice();
        let start = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => *start + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(end) => end + 1,
            Bound::Excluded(end) => *end,
            Bound::Unbounded => self.len(),
        };
        assert!(end >= start, "StringData::check_sliced: end < start");
        assert!(
            start <= by.len(),
            "StringData::check_sliced: start > bytes.len()"
        );
        assert!(
            end <= by.len(),
            "StringData::check_sliced: end > bytes.len()"
        );
        assert!(
            end == by.len() || by[end] & 0b1100_0000 != 0b1000_0000,
            "StringData::check_sliced: end is not a char boundary"
        );
        assert!(
            start == 0 || start == end || by[start] & 0b1100_0000 != 0b1000_0000,
            "StringData::check_sliced: start is not a char boundary"
        );
        start..end
    }

    /// Returns a `ByteData` with the given range of bytes.
    #[inline]
    #[must_use]
    pub fn sliced<R: RangeBounds<usize> + SliceIndex<str, Output = str>>(&self, range: R) -> Self {
        let range = self.check_sliced(range);
        let data = self.data.sliced(range);
        StringData { data }
    }

    /// Transform the range of bytes this `ByteData` represents.
    #[inline]
    #[must_use]
    pub fn into_sliced<R: RangeBounds<usize> + SliceIndex<str, Output = str>>(
        mut self,
        range: R,
    ) -> Self {
        let range = self.check_sliced(range);
        self.data.make_sliced(range);
        self
    }

    /// Transform the range of bytes this `StringData` represents.
    #[inline]
    pub fn make_sliced<R: RangeBounds<usize> + SliceIndex<str, Output = str>>(
        &'_ mut self,
        range: R,
    ) {
        let range = self.check_sliced(range);
        self.data.make_sliced(range);
    }

    /// Consume the `StringData` until the char condition is triggered.
    #[inline]
    #[must_use]
    pub fn take_while<F: FnMut(char) -> bool>(&mut self, mut fun: F) -> Self {
        let Some(position) = self.as_str().find(|ch| !fun(ch)) else {
            return core::mem::replace(self, StringData::empty());
        };
        if position == 0 {
            return StringData::empty();
        }
        let av = self.sliced(0..position);
        self.make_sliced(position..);
        av
    }

    /// Takes and removes the first line from the string.
    /// If a newline (`'\n'`) is found, the returned string will contain all data up to, and including, the newline.
    /// If the queue does not contain a newline character, the returned string will contain all data currently in the queue.
    #[inline]
    #[must_use]
    pub fn take_line(&mut self) -> Self {
        let Some(position) = self.as_str().find('\n') else {
            return core::mem::replace(self, Self::empty());
        };
        let av = self.sliced(0..position);
        self.make_sliced(position..);
        av
    }

    /// Split the `StringData` at the given position.
    #[inline]
    #[must_use]
    pub fn split_at(mut self, position: usize) -> (Self, Self) {
        let av = self.sliced(0..position);
        self.make_sliced(position..);
        (av, self)
    }

    /// Split the `StringData` at the first occurrence of the given byte sequence.
    ///
    /// # Errors
    ///
    /// Returns the input if the `needle` is not found.
    #[inline]
    pub fn split_once_on(self, needle: &str) -> Result<(Self, Self), Self> {
        let aa = match crate::const_split_once_bytes(self.as_bytes(), needle.as_bytes()) {
            Some((aa, _)) => aa.len(),
            None => return Err(self),
        };
        Ok(self.split_at(aa))
    }

    /// Split the `StringData` at the first occurrence of the given str sequence.
    #[inline]
    pub fn split_on<'b>(self, needle: &'b str) -> impl Iterator<Item = Self> + Send + 'b
    where
        'a: 'b,
    {
        self.data
            .split_on(needle.as_bytes())
            // SAFETY: `ByteData` is guaranteed to be valid UTF-8, unless the user has used `unsafe` methods.
            .map(|x| unsafe { Self::from_bytedata_unchecked(x) })
    }

    #[cfg(feature = "alloc")]
    /// Transform any borrowed data into shared data. This is useful when you wish to change the lifetime of the data.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[inline]
    #[must_use]
    pub fn into_shared<'s>(self) -> StringData<'s> {
        let StringData { data } = self;
        StringData {
            data: data.into_shared(),
        }
    }

    #[cfg(feature = "alloc")]
    /// Transform any borrowed data into shared data of a specific range. This is useful when you wish to change the lifetime of the data.
    ///
    /// This is essentially the same as `into_shared().into_sliced(range)`, but it is more efficient.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[inline]
    pub fn into_shared_range<'s, R: RangeBounds<usize> + SliceIndex<str, Output = str>>(
        self,
        range: R,
    ) -> StringData<'s> {
        let range = self.check_sliced(range);
        let StringData { data } = self;
        StringData {
            data: data.into_shared_range(range),
        }
    }

    /// Iterate over the characters of the `StringData`.
    #[inline]
    pub fn iter(&self) -> core::str::Chars<'_> {
        self.as_str().chars()
    }
}

impl StringData<'static> {
    /// Forces any borrowed str to be marked as static.
    #[inline]
    #[must_use]
    pub fn statically_borrowed(self) -> Self {
        StringData {
            data: self.data.statically_borrowed(),
        }
    }
}

impl AsRef<[u8]> for StringData<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsRef<str> for StringData<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for StringData<'_> {
    type Target = str;
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<'a> From<&'a str> for StringData<'a> {
    #[inline]
    fn from(dat: &'a str) -> Self {
        Self::from_borrowed(dat)
    }
}

impl<'a> From<StringData<'a>> for ByteData<'a> {
    #[inline]
    fn from(dat: StringData<'a>) -> Self {
        dat.into_bytedata()
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl TryFrom<SharedBytes> for StringData<'_> {
    type Error = SharedBytes;
    #[inline]
    fn try_from(dat: SharedBytes) -> Result<Self, Self::Error> {
        Self::try_from_shared(dat)
    }
}

impl<'a> TryFrom<ByteData<'a>> for StringData<'a> {
    type Error = ByteData<'a>;
    #[inline]
    fn try_from(dat: ByteData<'a>) -> Result<Self, Self::Error> {
        Self::try_from_bytedata(dat)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl From<String> for StringData<'_> {
    #[inline]
    fn from(dat: String) -> Self {
        Self::from_owned(dat)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<'a> From<alloc::borrow::Cow<'a, str>> for StringData<'a> {
    #[inline]
    fn from(data: alloc::borrow::Cow<'a, str>) -> Self {
        match data {
            alloc::borrow::Cow::Borrowed(borr) => Self::from_borrowed(borr),
            alloc::borrow::Cow::Owned(ow) => Self::from_owned(ow),
        }
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<'a> From<StringData<'a>> for String {
    #[inline]
    fn from(dat: StringData<'a>) -> Self {
        let dat = Vec::<u8>::from(dat.into_bytedata());
        // SAFETY: `StringData` is guaranteed to be valid UTF-8, unless the user has used `unsafe` methods.
        unsafe { Self::from_utf8_unchecked(dat) }
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<'a> From<StringData<'a>> for Cow<'a, str> {
    #[inline]
    fn from(dat: StringData<'a>) -> Self {
        let dat = Cow::<'a, [u8]>::from(dat.into_bytedata());
        match dat {
            Cow::Borrowed(borr) => {
                // SAFETY: `StringData` is guaranteed to be valid UTF-8, unless the user has used `unsafe` methods.
                Cow::Borrowed(unsafe { core::str::from_utf8_unchecked(borr) })
            }
            Cow::Owned(dat) => {
                // SAFETY: `StringData` is guaranteed to be valid UTF-8, unless the user has used `unsafe` methods.
                Cow::Owned(unsafe { String::from_utf8_unchecked(dat) })
            }
        }
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<'a> From<StringData<'a>> for alloc::vec::Vec<u8> {
    #[inline]
    fn from(dat: StringData<'a>) -> Self {
        Self::from(dat.into_bytedata())
    }
}

impl<'b> PartialEq<StringData<'b>> for StringData<'_> {
    #[inline]
    fn eq(&self, other: &StringData<'b>) -> bool {
        self.as_bytes().eq(other.as_bytes())
    }
}

impl PartialEq<str> for StringData<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}

impl PartialEq<[u8]> for StringData<'_> {
    #[inline]
    fn eq(&self, other: &[u8]) -> bool {
        self.as_bytes().eq(other)
    }
}

impl<'b> PartialEq<&'b str> for StringData<'_> {
    #[inline]
    fn eq(&self, other: &&'b str) -> bool {
        self.as_str().eq(*other)
    }
}

impl<'b> PartialEq<&'b [u8]> for StringData<'_> {
    #[inline]
    fn eq(&self, other: &&'b [u8]) -> bool {
        self.as_bytes().eq(*other)
    }
}

impl PartialEq<StringData<'_>> for [u8] {
    #[inline]
    fn eq(&self, other: &StringData<'_>) -> bool {
        self.eq(other.as_bytes())
    }
}

impl PartialEq<StringData<'_>> for str {
    #[inline]
    fn eq(&self, other: &StringData<'_>) -> bool {
        self.eq(other.as_str())
    }
}

impl<'a> PartialEq<StringData<'a>> for &'_ str {
    #[inline]
    fn eq(&self, other: &StringData<'a>) -> bool {
        (*self).eq(other.as_str())
    }
}

impl<'a> PartialEq<StringData<'a>> for &'_ [u8] {
    #[inline]
    fn eq(&self, other: &StringData<'a>) -> bool {
        (*self).eq(other.as_bytes())
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl PartialEq<Vec<u8>> for StringData<'_> {
    #[inline]
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.as_bytes().eq(other)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl PartialEq<String> for StringData<'_> {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.as_str().eq(other)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl PartialEq<StringData<'_>> for Vec<u8> {
    #[inline]
    fn eq(&self, other: &StringData<'_>) -> bool {
        self.eq(other.as_bytes())
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl PartialEq<StringData<'_>> for String {
    #[inline]
    fn eq(&self, other: &StringData<'_>) -> bool {
        self.eq(other.as_str())
    }
}

impl<'b> PartialEq<ByteData<'b>> for StringData<'_> {
    #[inline]
    fn eq(&self, other: &ByteData<'b>) -> bool {
        self.as_bytes().eq(other.as_slice())
    }
}

impl<'a> PartialEq<StringData<'a>> for ByteData<'_> {
    #[inline]
    fn eq(&self, other: &StringData<'a>) -> bool {
        self.as_slice().eq(other.as_bytes())
    }
}

impl Eq for StringData<'_> {}

impl core::hash::Hash for StringData<'_> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl<'b> PartialOrd<StringData<'b>> for StringData<'_> {
    #[inline]
    fn partial_cmp(&self, other: &StringData<'b>) -> Option<core::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl PartialOrd<[u8]> for StringData<'_> {
    #[inline]
    fn partial_cmp(&self, other: &[u8]) -> Option<core::cmp::Ordering> {
        self.as_bytes().partial_cmp(other)
    }
}

impl PartialOrd<str> for StringData<'_> {
    #[inline]
    fn partial_cmp(&self, other: &str) -> Option<core::cmp::Ordering> {
        self.as_str().partial_cmp(other)
    }
}

impl PartialOrd<StringData<'_>> for [u8] {
    #[inline]
    fn partial_cmp(&self, other: &StringData<'_>) -> Option<core::cmp::Ordering> {
        self.partial_cmp(other.as_bytes())
    }
}

impl PartialOrd<StringData<'_>> for str {
    #[inline]
    fn partial_cmp(&self, other: &StringData<'_>) -> Option<core::cmp::Ordering> {
        self.partial_cmp(other.as_str())
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl PartialOrd<Vec<u8>> for StringData<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Vec<u8>) -> Option<core::cmp::Ordering> {
        self.as_bytes().partial_cmp(AsRef::<[u8]>::as_ref(other))
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl PartialOrd<StringData<'_>> for Vec<u8> {
    #[inline]
    fn partial_cmp(&self, other: &StringData<'_>) -> Option<core::cmp::Ordering> {
        AsRef::<[u8]>::as_ref(self).partial_cmp(other.as_bytes())
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl PartialOrd<String> for StringData<'_> {
    #[inline]
    fn partial_cmp(&self, other: &String) -> Option<core::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl PartialOrd<StringData<'_>> for String {
    #[inline]
    fn partial_cmp(&self, other: &StringData<'_>) -> Option<core::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl<'b> PartialOrd<ByteData<'b>> for StringData<'_> {
    #[inline]
    fn partial_cmp(&self, other: &ByteData<'b>) -> Option<core::cmp::Ordering> {
        self.as_bytes().partial_cmp(other.as_slice())
    }
}

impl<'a> PartialOrd<StringData<'a>> for ByteData<'_> {
    #[inline]
    fn partial_cmp(&self, other: &StringData<'a>) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_bytes())
    }
}

impl Ord for StringData<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl core::fmt::Debug for StringData<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self.as_str(), f)
    }
}

impl core::fmt::Display for StringData<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self.as_str(), f)
    }
}

impl<'a> AsRef<crate::ByteData<'a>> for StringData<'a> {
    #[inline]
    fn as_ref(&self) -> &crate::ByteData<'a> {
        &self.data
    }
}

impl Iterator for StringData<'_> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            return None;
        }
        let mut ci = self.as_str().char_indices();
        let (_, ch) = ci.next()?;
        let idx = match ci.next() {
            Some((idx, _)) => idx,
            None => self.len(),
        };
        self.make_sliced(idx..);
        Some(ch)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.as_str().chars().size_hint()
    }
}

impl<'b> IntoIterator for &'b StringData<'_> {
    type Item = char;
    type IntoIter = core::str::Chars<'b>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.as_str().chars()
    }
}

impl core::borrow::Borrow<str> for StringData<'_> {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Default for StringData<'_> {
    #[inline]
    fn default() -> Self {
        StringData::empty()
    }
}
