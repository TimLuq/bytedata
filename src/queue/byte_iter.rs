use crate::{ByteData, ByteQueue};

use super::LinkedIter;

/// An iterator over the bytes of a [`ByteQueue`].
#[allow(missing_debug_implementations)]
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
    #[must_use]
    pub const fn skip(mut self, n: usize) -> Self {
        self.offset += n;
        if self.offset >= self.len {
            self.chunk = None;
            self.offset = 0;
            self.len = 0;
        }
        self
    }

    /// Skip the next `n` bytes.
    #[inline]
    pub(crate) fn skip_mut(&mut self, n: usize) -> &mut Self {
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
    #[must_use]
    pub const fn take(mut self, n: usize) -> Self {
        if self.len() > n {
            self.len = n + self.offset;
        }
        self
    }

    /// Get the number of bytes remaining in the iterator.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len - self.offset
    }

    /// Get the number of bytes remaining in the iterator.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[allow(single_use_lifetimes)]
impl<'a, 'b> Iterator for ByteIter<'a, 'b> {
    type Item = u8;

    #[allow(clippy::missing_inline_in_public_items)]
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
                Some(vv) => Some(vv),
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
    fn last(mut self) -> Option<Self::Item> {
        let len = self.len();
        if len == 0 {
            return None;
        }
        self.skip_mut(len - 1).next()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.skip_mut(n).next()
    }
}

#[allow(single_use_lifetimes)]
impl<'a, 'b> core::iter::ExactSizeIterator for ByteIter<'a, 'b> {
    #[inline]
    fn len(&self) -> usize {
        self.len()
    }
}

#[allow(single_use_lifetimes)]
impl<'a, 'b> core::iter::FusedIterator for ByteIter<'a, 'b> {}

/// An iterator over the bytes of a [`ByteQueue`].
#[allow(missing_debug_implementations)]
pub struct OwnedByteIter<'a> {
    inner: super::ByteQueue<'a>,
}

impl<'a> OwnedByteIter<'a> {
    #[inline]
    #[must_use]
    pub(super) const fn new(queue: ByteQueue<'a>) -> Self {
        Self { inner: queue }
    }

    /// Skip the next `n` bytes.
    #[inline]
    #[must_use]
    pub fn skip(mut self, n: usize) -> Self {
        self.skip_mut(n);
        self
    }

    /// Skip the next `n` bytes.
    #[inline]
    fn skip_mut(&mut self, mut n: usize) -> &mut Self {
        while let Some(mut av) = self.inner.pop_front() {
            if av.len() < n {
                n -= av.len();
                continue;
            }
            if av.len() == n {
                return self;
            }
            av.make_sliced(n..);
            self.inner.push_front(av);
            return self;
        }
        self
    }

    /// Limit the iterator to at most `n` bytes.
    #[inline]
    #[must_use]
    pub fn take(mut self, n: usize) -> Self {
        while self.len() > n {
            // SAFETY: `pop_back` is only called when `self.len() > n`.
            let mut av = unsafe { self.inner.pop_back().unwrap_unchecked() };
            if self.len() >= n {
                continue;
            }
            av.make_sliced(..av.len() - n);
            self.inner.push_back(av);
            break;
        }
        self
    }

    /// Get the number of bytes remaining in the iterator.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.inner.len()
    }

    /// Get the number of bytes remaining in the iterator.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Iterator for OwnedByteIter<'_> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut av = self.inner.pop_front()?;
        let bv = av.as_slice()[0];
        if av.len() > 1 {
            av.make_sliced(1..);
            self.inner.push_front(av);
        }
        Some(bv)
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
    fn last(mut self) -> Option<Self::Item> {
        let av = self.inner.pop_back()?;
        let bv = av.as_slice()[av.len() - 1];
        Some(bv)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.skip_mut(n).next()
    }
}

impl core::iter::ExactSizeIterator for OwnedByteIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.len()
    }
}

impl core::iter::FusedIterator for OwnedByteIter<'_> {}
