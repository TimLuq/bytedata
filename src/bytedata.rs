use core::{
    ops::{Deref, Index, RangeBounds},
    slice::SliceIndex,
};

use alloc::vec::Vec;

use crate::SharedBytes;

/// A container of bytes that can be either static, borrowed, or shared.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ByteData<'a> {
    Static(&'static [u8]),
    Borrowed(&'a [u8]),
    Shared(SharedBytes),
}

impl<'a> ByteData<'a> {
    /// Returns an empty `ByteData`.
    pub const fn empty() -> Self {
        Self::Static(&[])
    }
    /// Creates a `ByteData` from a slice of bytes.
    pub const fn from_static(dat: &'static [u8]) -> Self {
        Self::Static(dat)
    }
    /// Creates a `ByteData` from a borrowed slice of bytes.
    pub const fn from_borrowed(dat: &'a [u8]) -> Self {
        Self::Borrowed(dat)
    }
    /// Creates a `ByteData` from a `SharedBytes`.
    pub const fn from_shared(dat: SharedBytes) -> Self {
        Self::Shared(dat)
    }
    /// Returns the underlying byte slice.
    pub const fn as_slice(&self) -> &[u8] {
        match self {
            Self::Static(dat) => dat,
            Self::Borrowed(dat) => dat,
            Self::Shared(dat) => dat.as_slice(),
        }
    }
    /// Returns the length of the underlying byte slice.
    pub const fn len(&self) -> usize {
        match self {
            Self::Static(dat) => dat.len(),
            Self::Borrowed(dat) => dat.len(),
            Self::Shared(dat) => dat.len(),
        }
    }
    /// Returns `true` if the underlying byte slice is empty.
    pub const fn is_empty(&self) -> bool {
        match self {
            Self::Static(dat) => dat.is_empty(),
            Self::Borrowed(dat) => dat.is_empty(),
            Self::Shared(dat) => dat.is_empty(),
        }
    }
    /// Check if the underlying byte slice is equal to another. This can be used in a `const` context.
    #[inline]
    pub const fn eq_const(&self, other: &ByteData<'_>) -> bool {
        crate::const_eq(self.as_slice(), other.as_slice())
    }
    /// Check if the underlying byte slice is equal to another. This can be used in a `const` context.
    #[inline]
    pub const fn eq_slice(&self, other: &[u8]) -> bool {
        crate::const_eq(self.as_slice(), other)
    }

    /// Check if the ending of a `SharedBytes` matches the given bytes.
    pub const fn ends_with(&self, needle: &[u8]) -> bool {
        crate::const_ends_with(self.as_slice(), needle)
    }

    /// Check if the beginning of a `SharedBytes` matches the given bytes.
    pub const fn starts_with(&self, needle: &[u8]) -> bool {
        crate::const_starts_with(self.as_slice(), needle)
    }

    /// Returns a `ByteData` with the given range of bytes.
    pub fn sliced<R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>>(
        &self,
        range: R,
    ) -> Self {
        match self {
            Self::Static(dat) => Self::Static(&dat[range]),
            Self::Borrowed(dat) => Self::Borrowed(&dat[range]),
            Self::Shared(dat) => Self::Shared(dat.sliced_range(range)),
        }
    }
    /// Transform the range of bytes this `ByteData` represents.
    pub fn into_sliced<R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>>(
        self,
        range: R,
    ) -> Self {
        match self {
            Self::Static(dat) => Self::Static(&dat[range]),
            Self::Borrowed(dat) => Self::Borrowed(&dat[range]),
            Self::Shared(dat) => Self::Shared(dat.into_sliced_range(range)),
        }
    }
    /// Transform the range of bytes this `ByteData` represents.
    pub fn make_sliced<R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>>(
        &'_ mut self,
        range: R,
    ) {
        match self {
            Self::Static(dat) => *dat = &dat[range],
            Self::Borrowed(dat) => *dat = &dat[range],
            Self::Shared(dat) => {
                dat.make_sliced_range(range);
            }
        }
    }
    /// Transform any borrowed data into shared data. This is useful when you wish to change the lifetime of the data.
    pub fn into_shared<'s>(self) -> ByteData<'s> {
        match self {
            Self::Borrowed(dat) => ByteData::Shared(SharedBytes::from_slice(dat)),
            Self::Static(dat) => ByteData::Static(dat),
            Self::Shared(dat) => ByteData::Shared(dat),
        }
    }
    /// Transform any borrowed data into shared data of a specific range. This is useful when you wish to change the lifetime of the data.
    ///
    /// This is essentially the same as `into_shared().into_sliced(range)`, but it is more efficient.
    pub fn into_shared_range<'s, R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>>(
        self,
        range: R,
    ) -> ByteData<'s> {
        match self {
            Self::Borrowed(dat) => ByteData::Shared(SharedBytes::from_slice(&dat[range])),
            Self::Shared(dat) => ByteData::Shared(dat.into_sliced_range(range)),
            Self::Static(dat) => ByteData::Static(&dat[range]),
        }
    }
}

impl AsRef<[u8]> for ByteData<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<'a> Deref for ByteData<'a> {
    type Target = [u8];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<'a> From<&'a [u8]> for ByteData<'a> {
    #[inline]
    fn from(dat: &'a [u8]) -> Self {
        Self::from_borrowed(dat)
    }
}

impl<'a> From<SharedBytes> for ByteData<'a> {
    #[inline]
    fn from(dat: SharedBytes) -> Self {
        Self::from_shared(dat)
    }
}

impl<'a> From<Vec<u8>> for ByteData<'a> {
    #[inline]
    fn from(dat: Vec<u8>) -> Self {
        Self::from_shared(dat.into())
    }
}

impl Index<usize> for ByteData<'_> {
    type Output = u8;
    fn index(&self, idx: usize) -> &Self::Output {
        let sl = self.as_slice();
        if idx >= sl.len() {
            panic!("ByteData::index: index out of bounds");
        }
        unsafe { &*sl.as_ptr().add(idx) }
    }
}

impl<'a, 'b> PartialEq<ByteData<'b>> for ByteData<'a> {
    #[inline]
    fn eq(&self, other: &ByteData<'b>) -> bool {
        self.as_slice().eq(other.as_slice())
    }
}

impl PartialEq<[u8]> for ByteData<'_> {
    #[inline]
    fn eq(&self, other: &[u8]) -> bool {
        self.as_slice().eq(other)
    }
}

impl<'b> PartialEq<&'b [u8]> for ByteData<'_> {
    #[inline]
    fn eq(&self, other: &&'b [u8]) -> bool {
        self.as_slice().eq(*other)
    }
}

impl PartialEq<ByteData<'_>> for [u8] {
    #[inline]
    fn eq(&self, other: &ByteData<'_>) -> bool {
        self.eq(other.as_slice())
    }
}

impl PartialEq<Vec<u8>> for ByteData<'_> {
    #[inline]
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.as_slice().eq(other)
    }
}

impl PartialEq<ByteData<'_>> for Vec<u8> {
    #[inline]
    fn eq(&self, other: &ByteData<'_>) -> bool {
        self.eq(other.as_slice())
    }
}

impl Eq for ByteData<'_> {}

impl core::hash::Hash for ByteData<'_> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state)
    }
}

impl<'a, 'b> PartialOrd<ByteData<'b>> for ByteData<'a> {
    #[inline]
    fn partial_cmp(&self, other: &ByteData<'b>) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl PartialOrd<[u8]> for ByteData<'_> {
    #[inline]
    fn partial_cmp(&self, other: &[u8]) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(other)
    }
}

impl PartialOrd<ByteData<'_>> for [u8] {
    #[inline]
    fn partial_cmp(&self, other: &ByteData<'_>) -> Option<core::cmp::Ordering> {
        self.partial_cmp(other.as_slice())
    }
}

impl PartialOrd<Vec<u8>> for ByteData<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Vec<u8>) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(AsRef::<[u8]>::as_ref(other))
    }
}

impl PartialOrd<ByteData<'_>> for Vec<u8> {
    #[inline]
    fn partial_cmp(&self, other: &ByteData<'_>) -> Option<core::cmp::Ordering> {
        AsRef::<[u8]>::as_ref(self).partial_cmp(other.as_slice())
    }
}

impl Ord for ByteData<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}
