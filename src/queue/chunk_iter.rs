use crate::ByteData;

use super::linked_root::LinkedRoot;

/// An iterator over the chunks of a [`ByteQueue`].
///
/// [`ByteQueue`]: crate::ByteQueue
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
#[allow(missing_debug_implementations)]
pub struct ChunkIter<'a>(LinkedRoot<'a>);

impl<'a> ChunkIter<'a> {
    #[inline]
    pub(super) const fn new(data: LinkedRoot<'a>) -> Self {
        Self(data)
    }
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = ByteData<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.len(), Some(self.0.len()))
    }

    #[inline]
    fn count(self) -> usize {
        self.0.len()
    }
}

impl DoubleEndedIterator for ChunkIter<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.pop_back()
    }
}

impl ExactSizeIterator for ChunkIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl core::iter::FusedIterator for ChunkIter<'_> {}

/// An iterator over the chunks of a [`ByteQueue`].
///
/// [`ByteQueue`]: crate::ByteQueue
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
#[allow(missing_debug_implementations)]
pub struct StrChunkIter<'a>(LinkedRoot<'a>);

impl<'a> StrChunkIter<'a> {
    #[inline]
    pub(super) const fn new(data: LinkedRoot<'a>) -> Self {
        Self(data)
    }
}

impl<'a> Iterator for StrChunkIter<'a> {
    type Item = crate::StringData<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .pop_front()
            // Safety: The `ByteData` is guaranteed to be valid UTF-8.
            .map(|x| unsafe { crate::StringData::from_bytedata_unchecked(x) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.len(), Some(self.0.len()))
    }

    #[inline]
    fn count(self) -> usize {
        self.0.len()
    }
}

impl DoubleEndedIterator for StrChunkIter<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0
            .pop_back()
            // Safety: The `ByteData` is guaranteed to be valid UTF-8.
            .map(|x| unsafe { crate::StringData::from_bytedata_unchecked(x) })
    }
}

impl ExactSizeIterator for StrChunkIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl core::iter::FusedIterator for StrChunkIter<'_> {}
