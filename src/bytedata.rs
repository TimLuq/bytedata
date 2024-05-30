use core::{
    ops::{Deref, Index, RangeBounds},
    slice::SliceIndex,
};

#[cfg(feature = "alloc")]
use alloc::{borrow::Cow, vec::Vec};

#[cfg(feature = "alloc")]
use crate::SharedBytes;

/// A container of bytes that can be either static, borrowed, or shared.
#[derive(Clone)]
#[non_exhaustive]
pub enum ByteData<'a> {
    /// A static byte slice.
    Static(&'static [u8]),
    /// A borrowed byte slice.
    Borrowed(&'a [u8]),
    #[cfg(feature = "chunk")]
    #[cfg_attr(docsrs, doc(cfg(feature = "chunk")))]
    /// A chunk of bytes that is 12 bytes or less.
    Chunk(crate::byte_chunk::ByteChunk),
    #[cfg(feature = "alloc")]
    /// A shared byte slice.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    Shared(SharedBytes),
}

impl<'a> ByteData<'a> {
    /// Returns an empty `ByteData`.
    #[inline]
    pub const fn empty() -> Self {
        Self::Static(&[])
    }

    /// Creates a `ByteData` from a slice of bytes.
    #[inline]
    pub const fn from_static(dat: &'static [u8]) -> Self {
        Self::Static(dat)
    }

    #[cfg(feature = "chunk")]
    /// Creates a `ByteData` from a slice of bytes. The slice must be 12 bytes or less. If the slice is larger, this will panic.
    #[cfg_attr(docsrs, doc(cfg(feature = "chunk")))]
    #[inline]
    pub const fn from_chunk_slice(dat: &[u8]) -> Self {
        Self::Chunk(crate::byte_chunk::ByteChunk::from_slice(dat))
    }

    #[cfg(feature = "chunk")]
    /// Creates a `ByteData` from a single byte.
    #[cfg_attr(docsrs, doc(cfg(feature = "chunk")))]
    #[inline]
    pub const fn from_byte(b0: u8) -> Self {
        Self::Chunk(crate::byte_chunk::ByteChunk::from_byte(b0))
    }

    #[cfg(feature = "chunk")]
    /// Creates a `ByteData` from an array of bytes. The array must be 12 bytes or less. If the array is larger, this will panic.
    #[cfg_attr(docsrs, doc(cfg(feature = "chunk")))]
    #[inline]
    pub const fn from_chunk<const L: usize>(dat: &[u8; L]) -> Self {
        Self::Chunk(crate::byte_chunk::ByteChunk::from_array(&dat))
    }

    /// Creates a `ByteData` from a borrowed slice of bytes.
    #[inline]
    pub const fn from_borrowed(dat: &'a [u8]) -> Self {
        Self::Borrowed(dat)
    }

    #[cfg(feature = "alloc")]
    /// Creates a `ByteData` from a `SharedBytes`.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[inline]
    pub const fn from_shared(dat: SharedBytes) -> Self {
        Self::Shared(dat)
    }

    #[cfg(feature = "alloc")]
    /// Creates a `ByteData` from a `Vec<u8>`.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[inline]
    pub fn from_owned(dat: Vec<u8>) -> Self {
        #[cfg(feature = "chunk")]
        if dat.len() <= 12 {
            return Self::Chunk(crate::byte_chunk::ByteChunk::from_slice(&dat));
        }
        Self::Shared(dat.into())
    }

    #[cfg(feature = "alloc")]
    /// Creates a `ByteData` from a `Cow<'_, [u8]>`.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn from_cow(dat: Cow<'a, [u8]>) -> Self {
        match dat {
            Cow::Borrowed(b) => Self::from_borrowed(b),
            Cow::Owned(o) => Self::from_owned(o),
        }
    }

    #[cfg(feature = "alloc")]
    /// Creates a `ByteData` from a `Cow<'static, [u8]>`.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn from_cow_static(dat: Cow<'static, [u8]>) -> Self {
        match dat {
            Cow::Borrowed(b) => Self::from_static(b),
            Cow::Owned(o) => Self::from_owned(o),
        }
    }

    /// Returns the underlying byte slice.
    pub const fn as_slice(&self) -> &[u8] {
        match self {
            Self::Static(dat) => dat,
            Self::Borrowed(dat) => dat,
            #[cfg(feature = "chunk")]
            Self::Chunk(dat) => dat.as_slice(),
            #[cfg(feature = "alloc")]
            Self::Shared(dat) => dat.as_slice(),
        }
    }

    /// Returns the length of the underlying byte slice.
    pub const fn len(&self) -> usize {
        match self {
            Self::Static(dat) => dat.len(),
            Self::Borrowed(dat) => dat.len(),
            #[cfg(feature = "chunk")]
            Self::Chunk(dat) => dat.len(),
            #[cfg(feature = "alloc")]
            Self::Shared(dat) => dat.len(),
        }
    }

    /// Returns `true` if the underlying byte slice is empty.
    pub const fn is_empty(&self) -> bool {
        match self {
            Self::Static(dat) => dat.is_empty(),
            Self::Borrowed(dat) => dat.is_empty(),
            #[cfg(feature = "chunk")]
            Self::Chunk(dat) => dat.is_empty(),
            #[cfg(feature = "alloc")]
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
    #[inline]
    pub const fn ends_with(&self, needle: &[u8]) -> bool {
        crate::const_ends_with(self.as_slice(), needle)
    }

    /// Check if the beginning of a `SharedBytes` matches the given bytes.
    #[inline]
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
            #[cfg(feature = "chunk")]
            Self::Chunk(dat) => Self::Chunk(dat.sliced(range)),
            #[cfg(feature = "alloc")]
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
            #[cfg(feature = "chunk")]
            Self::Chunk(dat) => Self::Chunk(dat.sliced(range)),
            #[cfg(feature = "alloc")]
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
            #[cfg(feature = "chunk")]
            Self::Chunk(dat) => dat.make_sliced(range),
            #[cfg(feature = "alloc")]
            Self::Shared(dat) => {
                dat.make_sliced_range(range);
            }
        }
    }

    #[cfg(feature = "alloc")]
    /// Transform any borrowed data into shared data. This is useful when you wish to change the lifetime of the data.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn into_shared<'s>(self) -> ByteData<'s> {
        match self {
            #[cfg(feature = "chunk")]
            Self::Borrowed(dat) if dat.len() <= 12 => {
                ByteData::Chunk(crate::byte_chunk::ByteChunk::from_slice(dat))
            }
            Self::Borrowed(dat) => ByteData::Shared(SharedBytes::from_slice(dat)),
            Self::Static(dat) => ByteData::Static(dat),
            #[cfg(feature = "chunk")]
            Self::Chunk(dat) => ByteData::Chunk(dat),
            Self::Shared(dat) => ByteData::Shared(dat),
        }
    }

    #[cfg(feature = "alloc")]
    /// Transform any borrowed data into shared data of a specific range. This is useful when you wish to change the lifetime of the data.
    ///
    /// This is essentially the same as `into_shared().into_sliced(range)`, but it is more efficient.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn into_shared_range<'s, R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>>(
        self,
        range: R,
    ) -> ByteData<'s> {
        match self {
            Self::Borrowed(dat) => {
                let dat = &dat[range];
                #[cfg(feature = "chunk")]
                if dat.len() <= 12 {
                    return ByteData::Chunk(crate::byte_chunk::ByteChunk::from_slice(dat));
                }
                ByteData::Shared(SharedBytes::from_slice(dat))
            }
            Self::Shared(dat) => ByteData::Shared(dat.into_sliced_range(range)),
            Self::Static(dat) => ByteData::Static(&dat[range]),
            #[cfg(feature = "chunk")]
            Self::Chunk(dat) => ByteData::Chunk(dat.into_sliced(range)),
        }
    }
}

impl ByteData<'static> {
    /// Returns a `ByteData` with the given range of bytes.
    #[inline]
    pub fn statically_borrowed(self) -> ByteData<'static> {
        match self {
            Self::Static(dat) => ByteData::Static(dat),
            Self::Borrowed(dat) => ByteData::Static(dat),
            #[cfg(feature = "alloc")]
            Self::Shared(dat) => ByteData::Shared(dat),
            #[cfg(feature = "chunk")]
            Self::Chunk(dat) => ByteData::Chunk(dat),
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

#[cfg(feature = "alloc")]
impl<'a> From<SharedBytes> for ByteData<'a> {
    #[inline]
    fn from(dat: SharedBytes) -> Self {
        Self::from_shared(dat)
    }
}

#[cfg(feature = "alloc")]
impl<'a> From<Vec<u8>> for ByteData<'a> {
    #[inline]
    fn from(dat: Vec<u8>) -> Self {
        Self::from_shared(dat.into())
    }
}

#[cfg(feature = "alloc")]
impl<'a> From<alloc::string::String> for ByteData<'a> {
    #[inline]
    fn from(dat: alloc::string::String) -> Self {
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

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl PartialEq<Vec<u8>> for ByteData<'_> {
    #[inline]
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.as_slice().eq(other)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
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

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl PartialOrd<Vec<u8>> for ByteData<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Vec<u8>) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(AsRef::<[u8]>::as_ref(other))
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
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

impl core::fmt::Debug for ByteData<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.as_slice(), f)
    }
}

impl core::fmt::LowerHex for ByteData<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = self.as_slice();
        if let Some(w) = f.width() {
            if w > s.len() * 2 {
                for _ in 0..w - s.len() * 2 {
                    core::fmt::Write::write_str(f, "0")?;
                }
            }
        }
        let mut i = 0;
        while i < s.len() {
            write!(f, "{:02x}", s[i])?;
            i += 1;
        }
        Ok(())
    }
}

impl core::fmt::UpperHex for ByteData<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = self.as_slice();
        if let Some(w) = f.width() {
            if w > s.len() * 2 {
                for _ in 0..w - s.len() * 2 {
                    core::fmt::Write::write_str(f, "0")?;
                }
            }
        }
        let mut i = 0;
        while i < s.len() {
            write!(f, "{:02X}", s[i])?;
            i += 1;
        }
        Ok(())
    }
}
