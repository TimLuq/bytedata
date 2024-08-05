use core::{ops::RangeBounds, slice::SliceIndex};

/// A chunk of bytes that is 14 bytes or less.
#[derive(Clone, Copy)]
pub struct ByteChunk {
    /// The length of the chunk.
    pub(crate) len: u8,
    /// The data of the chunk.
    pub(crate) data: [u8; 14],
}

impl ByteChunk {
    /// The maximum length of a `ByteChunk`.
    pub const LEN: usize = 14;

    /// Create a `ByteChunk` from a slice.
    #[inline]
    #[must_use]
    pub const fn from_slice(data: &[u8]) -> Self {
        let len = data.len();
        core::assert!(len <= Self::LEN, "chunk data too large");
        let mut chunk = Self {
            #[allow(clippy::cast_possible_truncation)]
            len: len as u8,
            // SAFETY: `ByteChunk` with all zeros is a valid empty chunk.
            data: unsafe { core::mem::zeroed() },
        };
        let mut i = 0;
        while i < len {
            chunk.data[i] = data[i];
            i += 1;
        }
        chunk
    }

    /// Create a `ByteChunk` from a fixed-size array.
    #[inline]
    #[must_use]
    pub const fn from_array<const L: usize>(data: &[u8; L]) -> Self {
        core::assert!(L <= Self::LEN, "chunk data too large");
        let mut chunk = Self {
            #[allow(clippy::cast_possible_truncation)]
            len: L as u8,
            // SAFETY: `ByteChunk` with all zeros is a valid empty chunk.
            data: unsafe { core::mem::zeroed() },
        };
        let mut i = 0;
        while i < L {
            chunk.data[i] = data[i];
            i += 1;
        }
        chunk
    }

    /// Create a `ByteChunk` from a fixed-size array.
    #[inline]
    #[must_use]
    pub const fn from_byte(data: u8) -> Self {
        let mut chunk: Self = Self {
            len: 1,
            // SAFETY: `ByteChunk` with all zeros is a valid empty chunk.
            data: unsafe { core::mem::zeroed() },
        };
        chunk.data[0] = data;
        chunk
    }

    /// Get the bytes of the `ByteChunk` as a slice.
    #[inline]
    #[must_use]
    pub const fn as_slice(&self) -> &[u8] {
        let len = self.len as usize;
        if len == 0 {
            return &[];
        }
        // SAFETY: `len` is within bounds.
        unsafe { core::slice::from_raw_parts(self.data.as_ptr(), len) }
    }

    /// Get the number bytes of the `ByteChunk`.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    /// Check if the `ByteChunk` is empty.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Slice the `ByteChunk` in place.
    fn slice(&mut self, start: usize, end: usize) {
        let curr_len = self.len as usize;
        assert!(
            end <= curr_len && start <= end,
            "ByteData: range out of bounds"
        );
        #[allow(clippy::cast_possible_truncation)]
        let len = (end - start) as u8;
        self.len = len;
        if len == 0 || start == 0 {
            return;
        }
        let len = len as usize;
        // SAFETY: `start` and `end` are within bounds.
        let sorc = unsafe { self.data.as_ptr().add(start) };
        let dest = self.data.as_mut_ptr();
        // SAFETY: `sorc` and `dest` are valid pointers. They may however overlap for `len` bytes.
        unsafe { core::ptr::copy(sorc, dest, len) };
    }

    /// Return as subslice of the `ByteChunk`.
    #[inline]
    #[must_use]
    pub fn sliced<R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>>(
        &'_ self,
        range: R,
    ) -> Self {
        let start = match range.start_bound() {
            core::ops::Bound::Included(&start) => start,
            core::ops::Bound::Excluded(&start) => start + 1,
            core::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            core::ops::Bound::Included(&end) => end + 1,
            core::ops::Bound::Excluded(&end) => end,
            core::ops::Bound::Unbounded => self.len as usize,
        };
        let mut ret = *self;
        Self::slice(&mut ret, start, end);
        ret
    }

    /// Return as subslice of the `ByteChunk`.
    #[inline]
    #[must_use]
    pub fn into_sliced<R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>>(
        mut self,
        range: R,
    ) -> Self {
        let start = match range.start_bound() {
            core::ops::Bound::Included(&start) => start,
            core::ops::Bound::Excluded(&start) => start + 1,
            core::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            core::ops::Bound::Included(&end) => end + 1,
            core::ops::Bound::Excluded(&end) => end,
            core::ops::Bound::Unbounded => self.len as usize,
        };
        Self::slice(&mut self, start, end);
        self
    }

    /// Slice the `ByteChunk` in place.
    #[inline]
    pub fn make_sliced<R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>>(
        &'_ mut self,
        range: R,
    ) {
        let start = match range.start_bound() {
            core::ops::Bound::Included(&start) => start,
            core::ops::Bound::Excluded(&start) => start + 1,
            core::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            core::ops::Bound::Included(&end) => end + 1,
            core::ops::Bound::Excluded(&end) => end,
            core::ops::Bound::Unbounded => self.len as usize,
        };
        Self::slice(self, start, end);
    }
}

impl Default for ByteChunk {
    #[inline]
    fn default() -> Self {
        // SAFETY: `ByteChunk` with all zeros is a valid empty chunk.
        unsafe { core::mem::zeroed() }
    }
}

impl core::fmt::Debug for ByteChunk {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let rend = crate::ByteStringRender::from_slice(self.as_slice());
        core::fmt::Debug::fmt(&rend, f)
    }
}
