use core::{
    ops::{Bound, Deref, RangeBounds},
    slice::SliceIndex,
};

use alloc::{borrow::Cow, string::String, vec::Vec};

use crate::{ByteData, SharedBytes};

#[repr(transparent)]
pub struct StringData<'a> {
    data: ByteData<'a>,
}

impl<'a> StringData<'a> {
    /// Returns an empty `StringData`.
    pub const fn empty() -> Self {
        StringData {
            data: ByteData::Static(&[]),
        }
    }
    /// Creates a `StringData` from a slice of bytes.
    #[inline]
    pub const fn from_static(dat: &'static str) -> Self {
        StringData {
            data: ByteData::Static(dat.as_bytes()),
        }
    }
    /// Creates a `StringData` from a borrowed slice of bytes.
    #[inline]
    pub const fn from_borrowed(dat: &'a str) -> Self {
        StringData {
            data: ByteData::Borrowed(dat.as_bytes()),
        }
    }
    /// Creates a `StringData` from a `SharedBytes`.
    pub const fn try_from_shared(dat: SharedBytes) -> Result<Self, SharedBytes> {
        if core::str::from_utf8(dat.as_slice()).is_err() {
            return Err(dat);
        }
        Ok(StringData {
            data: ByteData::Shared(dat),
        })
    }
    /// Creates a `StringData` from `ByteData`.
    pub const fn try_from_bytedata(dat: ByteData<'a>) -> Result<Self, ByteData> {
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
    /// Prefer `try_from_bytedata` if you are unsure.
    pub const unsafe fn from_bytedata_unchecked(dat: ByteData<'a>) -> Self {
        StringData { data: dat }
    }
    /// Creates a `StringData` from a `String`.
    pub fn from_owned(dat: String) -> Self {
        StringData {
            data: ByteData::Shared(dat.into_bytes().into()),
        }
    }
    /// Creates a `StringData` from a `Cow<'_, str>`.
    pub fn from_cow(dat: Cow<'a, str>) -> Self {
        match dat {
            Cow::Borrowed(b) => Self::from_borrowed(b),
            Cow::Owned(o) => Self::from_owned(o),
        }
    }
    /// Creates a `StringData` from a `Cow<'static, str>`.
    pub fn from_cow_static(dat: Cow<'static, str>) -> Self {
        match dat {
            Cow::Borrowed(b) => Self::from_static(b),
            Cow::Owned(o) => Self::from_owned(o),
        }
    }
    /// Returns the underlying byte slice.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        self.data.as_slice()
    }
    /// Returns the underlying `str`.
    #[inline]
    pub const fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(self.data.as_slice()) }
    }
    /// Returns the length of the underlying byte slice.
    #[inline]
    pub const fn len(&self) -> usize {
        match &self.data {
            ByteData::Static(dat) => dat.len(),
            ByteData::Borrowed(dat) => dat.len(),
            ByteData::Shared(dat) => dat.len(),
        }
    }
    /// Returns `true` if the underlying byte slice is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    /// Check if the underlying byte slice is equal to another. This can be used in a `const` context.
    #[inline]
    pub const fn eq_const(&self, other: &StringData<'_>) -> bool {
        crate::const_eq(self.as_bytes(), other.as_bytes())
    }
    /// Check if the underlying byte slice is equal to another. This can be used in a `const` context.
    #[inline]
    pub const fn eq_slice(&self, other: &[u8]) -> bool {
        crate::const_eq(self.as_bytes(), other)
    }
    /// Check if the underlying byte slice is equal to another. This can be used in a `const` context.
    #[inline]
    pub const fn eq_str(&self, other: &str) -> bool {
        crate::const_eq(self.as_bytes(), other.as_bytes())
    }

    /// Check if the ending of a `SharedBytes` matches the given bytes.
    pub const fn ends_with(&self, needle: &str) -> bool {
        crate::const_ends_with(self.as_bytes(), needle.as_bytes())
    }

    /// Check if the beginning of a `SharedBytes` matches the given bytes.
    pub const fn starts_with(&self, needle: &str) -> bool {
        crate::const_starts_with(self.as_bytes(), needle.as_bytes())
    }

    fn check_sliced<R: RangeBounds<usize> + SliceIndex<str, Output = str>>(
        &self,
        range: R,
    ) -> core::ops::Range<usize> {
        let b = self.data.as_slice();
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
        if end < start {
            panic!("StringData::sliced: end < start");
        }
        if start > b.len() {
            panic!("StringData::sliced: start > bytes.len()");
        }
        if end > b.len() {
            panic!("StringData::sliced: end > bytes.len()");
        }
        if end < b.len() && b[end] & 0b1100_0000 == 0b1000_0000 {
            panic!("StringData::sliced: end is not a char boundary");
        }
        if start != 0 && b[start] & 0b1100_0000 == 0b1000_0000 {
            panic!("StringData::sliced: start is not a char boundary");
        }
        start..end
    }

    /// Returns a `ByteData` with the given range of bytes.
    #[inline]
    pub fn sliced<R: RangeBounds<usize> + SliceIndex<str, Output = str>>(&self, range: R) -> Self {
        let range = self.check_sliced(range);
        let data = self.data.sliced(range);
        StringData { data }
    }
    /// Transform the range of bytes this `ByteData` represents.
    #[inline]
    pub fn into_sliced<R: RangeBounds<usize> + SliceIndex<str, Output = str>>(
        mut self,
        range: R,
    ) -> Self {
        let range = self.check_sliced(range);
        self.data.make_sliced(range);
        self
    }
    /// Transform the range of bytes this `ByteData` represents.
    #[inline]
    pub fn make_sliced<R: RangeBounds<usize> + SliceIndex<str, Output = str>>(
        &'_ mut self,
        range: R,
    ) {
        let range = self.check_sliced(range);
        self.data.make_sliced(range);
    }
    /// Transform any borrowed data into shared data. This is useful when you wish to change the lifetime of the data.
    #[inline]
    pub fn into_shared<'s>(self) -> StringData<'s> {
        let StringData { data } = self;
        StringData {
            data: data.into_shared(),
        }
    }
    /// Transform any borrowed data into shared data of a specific range. This is useful when you wish to change the lifetime of the data.
    ///
    /// This is essentially the same as `into_shared().into_sliced(range)`, but it is more efficient.
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
}

impl AsRef<[u8]> for StringData<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<'a> Deref for StringData<'a> {
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

impl<'a> TryFrom<SharedBytes> for StringData<'a> {
    type Error = SharedBytes;
    #[inline]
    fn try_from(dat: SharedBytes) -> Result<Self, Self::Error> {
        Self::try_from_shared(dat)
    }
}

impl<'a> From<String> for StringData<'a> {
    #[inline]
    fn from(dat: String) -> Self {
        Self::from_owned(dat)
    }
}

impl<'a, 'b> PartialEq<StringData<'b>> for StringData<'a> {
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

impl PartialEq<Vec<u8>> for StringData<'_> {
    #[inline]
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.as_bytes().eq(other)
    }
}

impl PartialEq<String> for StringData<'_> {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.as_str().eq(other)
    }
}

impl PartialEq<StringData<'_>> for Vec<u8> {
    #[inline]
    fn eq(&self, other: &StringData<'_>) -> bool {
        self.eq(other.as_bytes())
    }
}

impl PartialEq<StringData<'_>> for String {
    #[inline]
    fn eq(&self, other: &StringData<'_>) -> bool {
        self.eq(other.as_str())
    }
}

impl<'a, 'b> PartialEq<ByteData<'b>> for StringData<'a> {
    #[inline]
    fn eq(&self, other: &ByteData<'b>) -> bool {
        self.as_bytes().eq(other.as_slice())
    }
}

impl<'a, 'b> PartialEq<StringData<'a>> for ByteData<'b> {
    #[inline]
    fn eq(&self, other: &StringData<'_>) -> bool {
        self.as_slice().eq(other.as_bytes())
    }
}

impl Eq for StringData<'_> {}

impl core::hash::Hash for StringData<'_> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl<'a, 'b> PartialOrd<StringData<'b>> for StringData<'a> {
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

impl PartialOrd<Vec<u8>> for StringData<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Vec<u8>) -> Option<core::cmp::Ordering> {
        self.as_bytes().partial_cmp(AsRef::<[u8]>::as_ref(other))
    }
}

impl PartialOrd<StringData<'_>> for Vec<u8> {
    #[inline]
    fn partial_cmp(&self, other: &StringData<'_>) -> Option<core::cmp::Ordering> {
        AsRef::<[u8]>::as_ref(self).partial_cmp(other.as_bytes())
    }
}

impl PartialOrd<String> for StringData<'_> {
    #[inline]
    fn partial_cmp(&self, other: &String) -> Option<core::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl PartialOrd<StringData<'_>> for String {
    #[inline]
    fn partial_cmp(&self, other: &StringData<'_>) -> Option<core::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl<'a, 'b> PartialOrd<ByteData<'b>> for StringData<'a> {
    #[inline]
    fn partial_cmp(&self, other: &ByteData<'b>) -> Option<core::cmp::Ordering> {
        self.as_bytes().partial_cmp(other.as_slice())
    }
}

impl<'a, 'b> PartialOrd<StringData<'a>> for ByteData<'b> {
    #[inline]
    fn partial_cmp(&self, other: &StringData<'_>) -> Option<core::cmp::Ordering> {
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.as_str().fmt(f)
    }
}
