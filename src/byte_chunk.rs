use core::{ops::RangeBounds, slice::SliceIndex};

/// A chunk of bytes that is 12 bytes or less.
#[cfg_attr(docsrs, doc(cfg(feature = "chunk")))]
#[derive(Clone, Copy)]
pub struct ByteChunk {
    /// The length of the chunk.
    pub(crate) len: u8,
    /// The data of the chunk.
    pub(crate) data: [u8; Self::LEN],
}

impl ByteChunk {
    /// The maximum length of a `ByteChunk`.
    pub const LEN: usize = 12;

    /// Create a `ByteChunk` from a slice.
    pub const fn from_slice(data: &[u8]) -> Self {
        let len = data.len();
        core::assert!(len <= Self::LEN, "chunk data too large");
        let mut chunk = ByteChunk {
            len: len as u8,
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
    pub const fn from_array<const L: usize>(data: &[u8; L]) -> Self {
        core::assert!(L <= 12, "chunk data too large");
        let mut chunk = ByteChunk {
            len: L as u8,
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
    pub const fn from_byte(data: u8) -> Self {
        let mut chunk: ByteChunk = ByteChunk {
            len: 1,
            data: unsafe { core::mem::zeroed() },
        };
        chunk.data[0] = data;
        chunk
    }

    /// Get the bytes of the `ByteChunk` as a slice.
    #[inline]
    pub const fn as_slice(&self) -> &[u8] {
        let len = self.len as usize;
        if len == 0 {
            return &[];
        }
        unsafe { core::slice::from_raw_parts(&self.data[0], len) }
    }

    /// Get the number bytes of the `ByteChunk`.
    #[inline]
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    /// Check if the `ByteChunk` is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Slice the `ByteChunk` in place.
    fn slice(&mut self, start: usize, end: usize) {
        let len = self.len as usize;
        if end > len || start > end {
            panic!("ByteData: range out of bounds");
        }
        let len = (end - start) as u8;
        self.len = len;
        if len != 0 && start != 0 {
            let len = len as usize;
            let mut i = 0usize;
            while i < len {
                self.data[i] = self.data[start + i];
                i += 1;
            }
        }
    }

    /// Return as subslice of the `ByteChunk`.
    pub fn sliced<R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>>(
        &'_ self,
        range: R,
    ) -> Self {
        let start = match range.start_bound() {
            core::ops::Bound::Included(&s) => s,
            core::ops::Bound::Excluded(&s) => s + 1,
            core::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            core::ops::Bound::Included(&e) => e + 1,
            core::ops::Bound::Excluded(&e) => e,
            core::ops::Bound::Unbounded => self.len as usize,
        };
        let mut r = *self;
        Self::slice(&mut r, start, end);
        r
    }

    /// Return as subslice of the `ByteChunk`.
    pub fn into_sliced<R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>>(
        mut self,
        range: R,
    ) -> Self {
        let start = match range.start_bound() {
            core::ops::Bound::Included(&s) => s,
            core::ops::Bound::Excluded(&s) => s + 1,
            core::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            core::ops::Bound::Included(&e) => e + 1,
            core::ops::Bound::Excluded(&e) => e,
            core::ops::Bound::Unbounded => self.len as usize,
        };
        Self::slice(&mut self, start, end);
        self
    }

    /// Slice the `ByteChunk` in place.
    pub fn make_sliced<R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>>(
        &'_ mut self,
        range: R,
    ) {
        let start = match range.start_bound() {
            core::ops::Bound::Included(&s) => s,
            core::ops::Bound::Excluded(&s) => s + 1,
            core::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            core::ops::Bound::Included(&e) => e + 1,
            core::ops::Bound::Excluded(&e) => e,
            core::ops::Bound::Unbounded => self.len as usize,
        };
        Self::slice(self, start, end);
    }
}
