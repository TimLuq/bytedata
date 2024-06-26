use core::{
    ops::{Bound, Deref, Index, RangeBounds},
    sync::atomic::AtomicU32,
};

use alloc::vec::Vec;

use crate::SharedBytesBuilder;

/// A slice of a reference-counted byte buffer.
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub struct SharedBytes {
    pub(crate) len: u32,
    pub(crate) off: u32,
    pub(crate) dat: *const u8,
}

unsafe impl Sync for SharedBytes {}
unsafe impl Send for SharedBytes {}

impl SharedBytes {
    /// An empty `SharedBytes`.
    pub const EMPTY: Self = Self {
        len: 0,
        off: 0,
        dat: core::ptr::null(),
    };
    /// Creates an empty `SharedBytes`.
    pub const fn empty() -> Self {
        Self::EMPTY
    }

    /// Creates a `SharedBytes` from a slice of bytes.
    pub fn from_slice(dat: &[u8]) -> Self {
        if dat.len() > 0xFFFF_FFF7 {
            panic!("SharedBytes::from_slice: slice too large");
        }
        let len = dat.len() as u32;
        let layout = alloc::alloc::Layout::from_size_align(dat.len() + 8, 4).unwrap();
        let ptr = unsafe {
            let ptr = alloc::alloc::alloc(layout);
            (ptr as *mut u32).write_volatile(len);
            (ptr.offset(4) as *mut u32).write_volatile(1);
            ptr.offset(8).copy_from(dat.as_ptr(), dat.len());
            ptr
        };
        Self {
            len,
            off: 8,
            dat: ptr,
        }
    }

    /// Creates a new `SharedBytesBuilder`.
    pub const fn builder() -> SharedBytesBuilder {
        SharedBytesBuilder::new()
    }

    /// Returns the number of bytes in the buffer.
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns `true` if the buffer is empty.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the bytes as a slice.
    pub const fn as_slice(&self) -> &[u8] {
        if self.len == 0 {
            return &[];
        }
        unsafe {
            core::slice::from_raw_parts(self.dat.offset(self.off as isize), self.len as usize)
        }
    }

    /// Check if the underlying byte slice is equal to another. This can be used in a `const` context.
    #[inline]
    pub const fn eq_const(&self, other: &SharedBytes) -> bool {
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

    /// Returns a new subslice of the bytes.
    pub fn sliced(&self, offset: usize, len: usize) -> Self {
        if offset > self.len as usize {
            panic!("SharedBytes::sliced: offset out of bounds");
        }
        if offset + len > self.len as usize {
            panic!("SharedBytes::sliced: offset + len out of bounds");
        }
        if len == 0 {
            return Self::EMPTY;
        }
        let len = len as u32;
        let off = self.off + offset as u32;
        unsafe { &mut *(self.dat.offset(4) as *mut AtomicU32) }
            .fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        Self {
            len,
            off,
            dat: self.dat,
        }
    }

    /// Returns a new subslice of the bytes.
    pub fn sliced_range<R: RangeBounds<usize>>(&self, range: R) -> Self {
        let start = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => *start + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(end) => end + 1,
            Bound::Excluded(end) => *end,
            Bound::Unbounded => self.len as usize,
        };
        if end < start {
            panic!("SharedBytes::sliced_range: end < start");
        }
        self.sliced(start, end - start)
    }

    /// Restrict the region of bytes this `SharedBytes` represents.
    pub const fn into_sliced(mut self, offset: usize, len: usize) -> Self {
        if offset > self.len as usize {
            panic!("SharedBytes::into_sliced: offset out of bounds");
        }
        if offset + len > self.len as usize {
            panic!("SharedBytes::into_sliced: offset + len out of bounds");
        }
        self.len = len as u32;
        self.off += offset as u32;
        self
    }

    /// Restrict the region of bytes this `SharedBytes` represents.
    pub fn make_sliced(&mut self, offset: usize, len: usize) -> &mut Self {
        if offset > self.len as usize {
            panic!("SharedBytes::into_sliced: offset out of bounds");
        }
        if offset + len > self.len as usize {
            panic!("SharedBytes::into_sliced: offset + len out of bounds");
        }
        self.len = len as u32;
        self.off += offset as u32;
        self
    }

    /// Restrict the region of bytes this `SharedBytes` represents.
    pub fn into_sliced_range<R: RangeBounds<usize>>(self, range: R) -> Self {
        let start = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => *start + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(end) => end + 1,
            Bound::Excluded(end) => *end,
            Bound::Unbounded => self.len as usize,
        };
        self.into_sliced(start, end - start)
    }

    /// Restrict the region of bytes this `SharedBytes` represents.
    pub fn make_sliced_range<R: RangeBounds<usize>>(&mut self, range: R) -> &mut Self {
        let start = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => *start + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(end) => end + 1,
            Bound::Excluded(end) => *end,
            Bound::Unbounded => self.len as usize,
        };
        self.make_sliced(start, end - start)
    }

    #[cfg(test)]
    pub(crate) fn ref_count(&self) -> u32 {
        if self.dat.is_null() {
            return 0;
        }
        unsafe { &*(self.dat.offset(4) as *mut AtomicU32) }
            .load(core::sync::atomic::Ordering::Relaxed)
    }
}

impl Clone for SharedBytes {
    fn clone(&self) -> Self {
        if self.len == 0 {
            return Self::EMPTY;
        }
        unsafe { &mut *(self.dat.offset(4) as *mut AtomicU32) }
            .fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        Self {
            len: self.len,
            off: self.off,
            dat: self.dat,
        }
    }
}

impl Drop for SharedBytes {
    fn drop(&mut self) {
        if self.dat.is_null() {
            return;
        }
        unsafe {
            let refcnt = &mut *(self.dat.offset(4) as *mut AtomicU32);
            if refcnt.fetch_sub(1, core::sync::atomic::Ordering::Relaxed) == 1 {
                let layout = alloc::alloc::Layout::from_size_align(
                    *(self.dat.offset(4) as *mut u32) as usize,
                    4,
                )
                .unwrap();
                alloc::alloc::dealloc(self.dat as *mut u8, layout);
            }
            self.dat = core::ptr::null();
        }
    }
}

impl core::str::FromStr for SharedBytes {
    type Err = core::convert::Infallible;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_slice(s.as_bytes()))
    }
}

impl AsRef<[u8]> for SharedBytes {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl Deref for SharedBytes {
    type Target = [u8];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl From<&[u8]> for SharedBytes {
    #[inline]
    fn from(dat: &[u8]) -> Self {
        Self::from_slice(dat)
    }
}

impl From<Vec<u8>> for SharedBytes {
    #[inline]
    fn from(dat: Vec<u8>) -> Self {
        Self::from_slice(&dat)
    }
}

impl From<alloc::string::String> for SharedBytes {
    #[inline]
    fn from(dat: alloc::string::String) -> Self {
        Self::from_slice(dat.as_bytes())
    }
}

impl Index<usize> for SharedBytes {
    type Output = u8;
    fn index(&self, idx: usize) -> &Self::Output {
        if idx >= self.len as usize {
            panic!("SharedBytes::index: index out of bounds");
        }
        unsafe { &*self.dat.offset(self.off as isize + idx as isize) }
    }
}

impl Index<core::ops::RangeFull> for SharedBytes {
    type Output = [u8];
    fn index(&self, range: core::ops::RangeFull) -> &Self::Output {
        self.as_slice().index(range)
    }
}

impl Index<core::ops::RangeTo<usize>> for SharedBytes {
    type Output = [u8];
    fn index(&self, range: core::ops::RangeTo<usize>) -> &Self::Output {
        self.as_slice().index(range)
    }
}

impl Index<core::ops::RangeFrom<usize>> for SharedBytes {
    type Output = [u8];
    fn index(&self, range: core::ops::RangeFrom<usize>) -> &Self::Output {
        self.as_slice().index(range)
    }
}

impl Index<core::ops::Range<usize>> for SharedBytes {
    type Output = [u8];
    fn index(&self, range: core::ops::Range<usize>) -> &Self::Output {
        self.as_slice().index(range)
    }
}

impl PartialEq for SharedBytes {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl PartialEq<SharedBytes> for [u8] {
    fn eq(&self, other: &SharedBytes) -> bool {
        self == other.as_slice()
    }
}

impl PartialEq<SharedBytes> for Vec<u8> {
    fn eq(&self, other: &SharedBytes) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl PartialEq<[u8]> for SharedBytes {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_slice() == other
    }
}

impl PartialEq<Vec<u8>> for SharedBytes {
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<'b> PartialEq<&'b [u8]> for SharedBytes {
    #[inline]
    fn eq(&self, other: &&'b [u8]) -> bool {
        self.as_slice().eq(*other)
    }
}

impl Eq for SharedBytes {}

impl core::hash::Hash for SharedBytes {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state)
    }
}

impl PartialOrd for SharedBytes {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.as_slice().cmp(other.as_slice()))
    }
}

impl PartialOrd<SharedBytes> for [u8] {
    #[inline]
    fn partial_cmp(&self, other: &SharedBytes) -> Option<core::cmp::Ordering> {
        self.partial_cmp(other.as_slice())
    }
}

impl PartialOrd<SharedBytes> for Vec<u8> {
    #[inline]
    fn partial_cmp(&self, other: &SharedBytes) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl PartialOrd<[u8]> for SharedBytes {
    #[inline]
    fn partial_cmp(&self, other: &[u8]) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(other)
    }
}

impl PartialOrd<Vec<u8>> for SharedBytes {
    fn partial_cmp(&self, other: &Vec<u8>) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl<'b> PartialOrd<&'b [u8]> for SharedBytes {
    fn partial_cmp(&self, other: &&'b [u8]) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(*other)
    }
}

impl Ord for SharedBytes {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl core::fmt::Debug for SharedBytes {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SharedBytes")
            .field("len", &self.len)
            .field("off", &self.off)
            .field("ptr", &self.dat)
            .field("dat", &self.as_slice())
            .finish()
    }
}

impl core::fmt::LowerHex for SharedBytes {
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

impl core::fmt::UpperHex for SharedBytes {
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
