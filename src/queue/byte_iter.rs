use crate::{ByteData, ByteQueue};

use super::LinkedIter;

/// An iterator over the bytes of a [`ByteQueue`].
pub struct ByteIter<'a, 'b> {
    inner: LinkedIter<'a, 'b>,
    chunk: Option<&'b ByteData<'a>>,
    offset: usize,
    len: usize,
}

impl<'a, 'b> ByteIter<'a, 'b> {
    #[inline]
    pub(super) fn new(queue: &'b ByteQueue<'a>) -> Self {
        Self {
            inner: queue.chunks(),
            chunk: None,
            offset: 0,
            len: queue.len(),
        }
    }

    /// Skip the next `n` bytes.
    #[inline]
    pub const fn skip(mut self, n: usize) -> Self {
        self.offset += n;
        if self.offset >= self.len {
            self.chunk = None;
            self.offset = 0;
            self.len = 0;
        }
        self
    }

    /// Limit the iterator to at most `n` bytes.
    #[inline]
    pub const fn take(mut self, n: usize) -> Self {
        if self.len() > n {
            self.len = n + self.offset;
        }
        self
    }

    /// Get the number of bytes remaining in the iterator.
    #[inline]
    pub const fn len(&self) -> usize {
        self.len - self.offset
    }
}

impl<'a, 'b> Iterator for ByteIter<'a, 'b> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.len {
            return None;
        }
        loop {
            let old_len = self.chunk.map(ByteData::len).unwrap_or_default();
            if old_len > self.offset {
                break;
            }
            self.chunk = match self.inner.next() {
                Some(v) => Some(v),
                None => return None,
            };
            self.offset -= old_len;
            self.len -= old_len;
        }

        let chunk = self.chunk?;
        let byte = chunk[self.offset];
        self.offset += 1;
        Some(byte)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        let l = self.len();
        if l == 0 {
            return None;
        }
        self.skip(l - 1).next()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.skip(n).next()
    }
}

impl<'a, 'b> core::iter::ExactSizeIterator for ByteIter<'a, 'b> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<'a, 'b> core::iter::FusedIterator for ByteIter<'a, 'b> {}
