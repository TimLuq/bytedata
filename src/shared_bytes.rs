use core::ops::{Bound, Deref, Index, RangeBounds};

type RefCounter = core::sync::atomic::AtomicU32;

use alloc::vec::Vec;

use crate::SharedBytesBuilder;

#[allow(clippy::redundant_pub_crate)]
#[repr(C)]
pub(crate) struct SharedBytesMeta {
    /// The reference count of the data.
    pub(crate) refcnt: RefCounter,
    /// The last 3 bits are used to store the alignment of the data.
    pub(crate) info: u8,
    pub(crate) _reserved: [u8; 3],
    /// The allocated length of the data, some of which may be uninitialized.
    pub(crate) len: u32,
}

impl SharedBytesMeta {
    #[inline]
    pub(crate) const fn new() -> Self {
        // SAFETY: `SharedBytesMeta` is a valid empty struct.
        unsafe { core::mem::zeroed::<Self>() }.with_align(4)
    }

    #[inline]
    pub(crate) const fn with_len(mut self, len: u32) -> Self {
        self.len = len;
        self
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    pub(crate) const fn with_align(mut self, align: usize) -> Self {
        const ALIGN: usize = core::mem::align_of::<SharedBytesMeta>();
        let align = if align < ALIGN { ALIGN } else { align };
        self.info = (align >> 2).ilog2() as u8;
        self
    }

    #[inline]
    pub(crate) const fn with_refcount(mut self, count: u32) -> Self {
        self.refcnt = RefCounter::new(count);
        self
    }

    #[inline]
    pub(crate) const fn align(&self) -> usize {
        4_usize << self.info
    }

    pub(crate) const fn compute_start_offset(align: usize) -> u32 {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::integer_division_remainder_used
        )]
        if align >= core::mem::size_of::<Self>() {
            align as u32
        } else {
            let diff = core::mem::size_of::<Self>() % align;
            if diff == 0 {
                core::mem::size_of::<Self>() as u32
            } else {
                (core::mem::size_of::<Self>() + (align - diff)) as u32
            }
        }
    }
}

/// A slice of a reference-counted byte buffer.
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[repr(C)]
pub struct SharedBytes {
    pub(crate) dat_addr: u64,
    pub(crate) len: u32,
    pub(crate) off: u32,
}

impl SharedBytes {
    /// An empty `SharedBytes`.
    pub const EMPTY: Self = Self {
        len: 0,
        off: 0,
        dat_addr: 0,
    };

    /// Creates an empty `SharedBytes`.
    #[inline]
    #[must_use]
    pub const fn empty() -> Self {
        Self::EMPTY
    }

    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub(crate) const fn dat(&self) -> *const u8 {
        self.dat_addr.to_le() as usize as *const u8
    }

    /// Creates a `SharedBytes` from a slice of bytes.
    #[inline]
    #[must_use]
    pub fn from_slice(dat: &[u8]) -> Self {
        Self::from_slice_aligned(dat, core::mem::align_of::<SharedBytesMeta>())
    }

    /// Creates a `SharedBytes` from a slice of bytes and a target alignment.
    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn from_slice_aligned(dat: &[u8], align: usize) -> Self {
        fn from_slice_aligned_inner(dat: &[u8], align: usize) -> SharedBytes {
            let align = core::mem::align_of::<SharedBytesMeta>().max(align);
            let off = SharedBytesMeta::compute_start_offset(align) as usize;
            let max_size: usize = 0xFFFF_FFFF - off;
            assert!(
                dat.len() <= max_size,
                "SharedBytes::from_slice: slice too large"
            );
            let len = dat.len() as u32;
            let alloc_size = dat.len() + off;
            #[allow(clippy::unwrap_used)]
            let layout = core::alloc::Layout::from_size_align(alloc_size, align).unwrap();
            let ptr = {
                // SAFETY: `layout` is a valid layout.
                let ptr = unsafe { alloc::alloc::alloc(layout) };
                if ptr.is_null() {
                    alloc::alloc::handle_alloc_error(layout);
                }
                #[allow(clippy::cast_ptr_alignment)]
                let meta = ptr.cast::<SharedBytesMeta>();
                let meta_item = SharedBytesMeta::new()
                    .with_len(alloc_size as u32)
                    .with_align(align)
                    .with_refcount(1);
                // SAFETY: `meta` is a valid pointer to the prefix `SharedBytesMeta`.
                unsafe {
                    meta.write(meta_item);
                };
                // SAFETY: `ptr` has a data section at `off` bytes from the start.
                let dat_ptr = unsafe { ptr.add(off) };
                // SAFETY: `dat_ptr` is a valid pointer and `dat` is a valid slice.
                unsafe {
                    core::ptr::copy_nonoverlapping(dat.as_ptr(), dat_ptr, dat.len());
                };
                ptr
            };
            let dat_addr = ptr as usize as u64;
            let dat_addr = dat_addr.to_le();
            SharedBytes {
                len,
                off: off as u32,
                dat_addr,
            }
        }
        if dat.is_empty() {
            return Self::EMPTY;
        }
        from_slice_aligned_inner(dat, align)
    }

    /// Creates a new `SharedBytesBuilder`.
    #[inline]
    #[must_use]
    pub const fn builder() -> SharedBytesBuilder {
        SharedBytesBuilder::new()
    }

    /// Returns the number of bytes in the buffer.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns `true` if the buffer is empty.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns `true` if there is only a single owner of the data.
    #[inline]
    #[must_use]
    pub fn is_unique(&self) -> bool {
        let dat = self.dat();
        if dat.is_null() {
            return true;
        }
        #[allow(clippy::cast_ptr_alignment)]
        let meta = dat.cast::<SharedBytesMeta>();
        // SAFETY: `meta` is a valid pointer.
        let meta = unsafe { &*meta };
        meta.refcnt.load(core::sync::atomic::Ordering::Relaxed) == 1
    }

    /// Returns the bytes as a slice.
    #[inline]
    #[must_use]
    pub const fn as_slice(&self) -> &[u8] {
        if self.len == 0 {
            return &[];
        }
        let len = self.len as usize;
        let off = self.off as usize;
        let dat = self.dat();
        // SAFETY: `len` and `off` are within bounds.
        let dat = unsafe { dat.add(off) };
        // SAFETY: `dat` is a valid pointer.
        unsafe { core::slice::from_raw_parts(dat, len) }
    }

    /// Check if the underlying byte slice is equal to another. This can be used in a `const` context.
    #[inline]
    #[must_use]
    pub const fn eq_const(&self, other: &Self) -> bool {
        crate::const_eq(self.as_slice(), other.as_slice())
    }

    /// Check if the underlying byte slice is equal to another. This can be used in a `const` context.
    #[inline]
    #[must_use]
    pub const fn eq_slice(&self, other: &[u8]) -> bool {
        crate::const_eq(self.as_slice(), other)
    }

    /// Check if the ending of a `SharedBytes` matches the given bytes.
    #[inline]
    #[must_use]
    pub const fn ends_with(&self, needle: &[u8]) -> bool {
        crate::const_ends_with(self.as_slice(), needle)
    }

    /// Check if the beginning of a `SharedBytes` matches the given bytes.
    #[inline]
    #[must_use]
    pub const fn starts_with(&self, needle: &[u8]) -> bool {
        crate::const_starts_with(self.as_slice(), needle)
    }

    /// Returns a new subslice of the bytes.
    ///
    /// # Panics
    ///
    /// Panics if `offset` or `offset + len` are out of bounds.
    #[must_use]
    #[allow(clippy::missing_inline_in_public_items)]
    pub fn sliced(&self, offset: usize, len: usize) -> Self {
        assert!(
            offset <= self.len as usize,
            "SharedBytes::sliced: offset out of bounds"
        );
        assert!(
            offset + len <= self.len as usize,
            "SharedBytes::sliced: offset + len out of bounds"
        );
        if len == 0 {
            return Self::EMPTY;
        }
        #[allow(clippy::cast_possible_truncation)]
        let len = len as u32;
        #[allow(clippy::cast_possible_truncation)]
        let off = self.off + offset as u32;
        #[allow(clippy::cast_ptr_alignment)]
        let meta = self.dat().cast::<SharedBytesMeta>();
        // SAFETY: `meta` is a valid pointer.
        unsafe { &*meta }
            .refcnt
            .fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        Self {
            len,
            off,
            dat_addr: self.dat_addr,
        }
    }

    /// Returns a new subslice of the bytes.
    ///
    /// # Panics
    ///
    /// Panics if `range` are out of bounds.
    #[inline]
    #[must_use]
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
        assert!(end >= start, "SharedBytes::sliced_range: end < start");
        self.sliced(start, end - start)
    }

    /// Restrict the region of bytes this `SharedBytes` represents.
    ///
    /// # Panics
    ///
    /// Panics if `range` are out of bounds.
    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn into_sliced(mut self, offset: usize, len: usize) -> Self {
        assert!(
            offset <= self.len as usize,
            "SharedBytes::into_sliced: offset out of bounds"
        );
        assert!(
            offset + len <= self.len as usize,
            "SharedBytes::into_sliced: offset + len out of bounds"
        );
        self.len = len as u32;
        self.off += offset as u32;
        self
    }

    /// Restrict the region of bytes this `SharedBytes` represents.
    ///
    /// # Panics
    ///
    /// Panics if `range` are out of bounds.
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    pub fn make_sliced(&mut self, offset: usize, len: usize) -> &mut Self {
        assert!(
            offset <= self.len as usize,
            "SharedBytes::make_sliced: offset out of bounds"
        );
        assert!(
            offset + len <= self.len as usize,
            "SharedBytes::make_sliced: offset + len out of bounds"
        );
        self.len = len as u32;
        self.off += offset as u32;
        self
    }

    /// Restrict the region of bytes this `SharedBytes` represents.
    ///
    /// # Panics
    ///
    /// Panics if `range` are out of bounds.
    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
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
    ///
    /// # Panics
    ///
    /// Panics if `range` are out of bounds.
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
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
    #[inline]
    pub(crate) fn ref_count(&self) -> u32 {
        let dat = self.dat();
        if dat.is_null() {
            return 0;
        }
        #[allow(clippy::cast_ptr_alignment)]
        let meta = dat.cast::<SharedBytesMeta>();
        // SAFETY: `meta` is a valid pointer.
        unsafe { &*meta }
            .refcnt
            .load(core::sync::atomic::Ordering::Relaxed)
    }
}

impl Clone for SharedBytes {
    #[inline]
    fn clone(&self) -> Self {
        if self.len == 0 {
            return Self::EMPTY;
        }
        let dat = self.dat();
        if dat.is_null() {
            return Self::EMPTY;
        }
        #[allow(clippy::cast_ptr_alignment)]
        let dat = dat.cast::<SharedBytesMeta>().cast_mut();
        // SAFETY: `dat` is a valid pointer.
        unsafe { &mut *dat }
            .refcnt
            .fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        Self {
            len: self.len,
            off: self.off,
            dat_addr: self.dat_addr,
        }
    }
}

impl Drop for SharedBytes {
    #[inline]
    fn drop(&mut self) {
        fn drop_dat(dat: &u8) {
            let dat = dat as *const u8;
            #[allow(clippy::cast_ptr_alignment)]
            // SAFETY: `dat` should point to a `SharedBytesMeta`.
            let meta = unsafe { &*dat.cast::<SharedBytesMeta>() };
            let refcnt = &meta.refcnt;
            if refcnt.fetch_sub(1, core::sync::atomic::Ordering::Relaxed) == 1 {
                #[allow(clippy::unwrap_used)]
                let layout =
                    core::alloc::Layout::from_size_align(meta.len as usize, meta.align()).unwrap();
                let dat = dat.cast_mut();
                // SAFETY: `dat` is a valid pointer and the layout should be correct.
                unsafe {
                    alloc::alloc::dealloc(dat, layout);
                };
            }
        }
        let dat = self.dat();
        // SAFETY: `dat` is a valid pointer.
        let Some(dat) = (unsafe { dat.as_ref() }) else {
            return;
        };
        drop_dat(dat);
        self.dat_addr = 0;
    }
}

impl core::str::FromStr for SharedBytes {
    type Err = core::convert::Infallible;

    #[inline]
    #[allow(clippy::min_ident_chars)]
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

impl<'a> From<crate::ByteData<'a>> for SharedBytes {
    #[inline]
    fn from(dat: crate::ByteData<'a>) -> Self {
        if matches!(dat.kind(), crate::bytedata::Kind::Shared) {
            // SAFETY: `dat` is a `SharedBytes` according to the kind.
            return unsafe { core::mem::transmute::<crate::ByteData<'a>, Self>(dat) };
        }
        let ret = Self::from_slice(dat.as_slice());
        core::mem::drop(dat);
        ret
    }
}

#[cfg(feature = "queue")]
impl<'a> From<crate::ByteQueue<'a>> for SharedBytes {
    #[inline]
    fn from(dat: crate::ByteQueue<'a>) -> Self {
        crate::ByteData::from(dat).into()
    }
}

impl From<SharedBytesBuilder> for SharedBytes {
    #[inline]
    fn from(dat: SharedBytesBuilder) -> Self {
        dat.build()
    }
}

impl From<SharedBytes> for SharedBytesBuilder {
    #[inline]
    fn from(mut dat: SharedBytes) -> Self {
        let dat_dat = dat.dat();
        if dat_dat.is_null() {
            return Self::new();
        }
        // SAFETY: `dat_dat` should be a valid pointer.
        #[allow(clippy::cast_ptr_alignment)]
        let meta = unsafe { &*dat_dat.cast::<SharedBytesMeta>() };
        let align = meta.align();
        if meta.refcnt.load(core::sync::atomic::Ordering::Relaxed) == 1 {
            let len = meta.len;
            let dat_len = dat.len;
            let dataptr = dat_dat.cast_mut();
            dat.dat_addr = 0;
            let start_off = SharedBytesMeta::compute_start_offset(align);
            if dat_len != 0 && dat.off != start_off {
                // there is a prefix of unwanted data, so we move it to the beginning

                // SAFETY: `start_off` is the correct offset for the data, as determined by the alignment.
                let dest = unsafe { dataptr.add(start_off as usize) };
                // SAFETY: `dat.off` is the offset of the data in the buffer which is greater than `start_off` but regions may overlap.
                let sorc = unsafe { dataptr.add(dat.off as usize) };
                // SAFETY: `dat_len` is the length of the data in the buffer, `src` and `dst` are valid pointers but may overlap within `dat_len` bytes.
                unsafe {
                    dest.copy_from(sorc, dat_len as usize);
                };
            }
            core::mem::forget(dat);
            Self {
                align,
                off: start_off + dat_len,
                len,
                dat: dataptr,
            }
        } else {
            let mut ret = Self::with_alignment(align);
            ret.extend_from_slice(dat.as_slice());
            ret
        }
    }
}

impl Index<usize> for SharedBytes {
    type Output = u8;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        assert!(
            index < self.len as usize,
            "SharedBytes::index: index out of bounds"
        );
        // SAFETY: `index` is checked to be in bounds.
        let ptr = unsafe { self.dat().add(self.off as usize + index) };
        // SAFETY: `ptr` is a valid pointer.
        unsafe { &*ptr }
    }
}

impl Index<core::ops::RangeFull> for SharedBytes {
    type Output = [u8];

    #[inline]
    fn index(&self, index: core::ops::RangeFull) -> &Self::Output {
        self.as_slice().index(index)
    }
}

impl Index<core::ops::RangeTo<usize>> for SharedBytes {
    type Output = [u8];

    #[inline]
    fn index(&self, index: core::ops::RangeTo<usize>) -> &Self::Output {
        self.as_slice().index(index)
    }
}

impl Index<core::ops::RangeFrom<usize>> for SharedBytes {
    type Output = [u8];

    #[inline]
    fn index(&self, index: core::ops::RangeFrom<usize>) -> &Self::Output {
        self.as_slice().index(index)
    }
}

impl Index<core::ops::Range<usize>> for SharedBytes {
    type Output = [u8];

    #[inline]
    fn index(&self, index: core::ops::Range<usize>) -> &Self::Output {
        self.as_slice().index(index)
    }
}

impl PartialEq for SharedBytes {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl PartialEq<SharedBytes> for [u8] {
    #[inline]
    fn eq(&self, other: &SharedBytes) -> bool {
        self == other.as_slice()
    }
}

impl PartialEq<SharedBytes> for Vec<u8> {
    #[inline]
    fn eq(&self, other: &SharedBytes) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl PartialEq<[u8]> for SharedBytes {
    #[inline]
    fn eq(&self, other: &[u8]) -> bool {
        self.as_slice() == other
    }
}

impl PartialEq<Vec<u8>> for SharedBytes {
    #[inline]
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
        self.as_slice().hash(state);
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
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
    #[inline]
    fn partial_cmp(&self, other: &Vec<u8>) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl<'b> PartialOrd<&'b [u8]> for SharedBytes {
    #[inline]
    fn partial_cmp(&self, other: &&'b [u8]) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(*other)
    }
}

impl Ord for SharedBytes {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

#[allow(
    clippy::missing_inline_in_public_items,
    clippy::missing_fields_in_debug,
    clippy::min_ident_chars
)]
impl core::fmt::Debug for SharedBytes {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SharedBytes")
            .field("len", &self.len)
            .field("off", &self.off)
            .field("ptr", &self.dat())
            .field("dat", &crate::ByteStringRender::from_slice(self.as_slice()))
            .finish()
    }
}

impl core::fmt::LowerHex for SharedBytes {
    #[allow(clippy::min_ident_chars)]
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::byte_string_render::lower_hex_slice(self.as_slice(), f)
    }
}

impl core::fmt::UpperHex for SharedBytes {
    #[allow(clippy::min_ident_chars)]
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::byte_string_render::upper_hex_slice(self.as_slice(), f)
    }
}

impl Default for SharedBytes {
    #[inline]
    fn default() -> Self {
        Self::EMPTY
    }
}
