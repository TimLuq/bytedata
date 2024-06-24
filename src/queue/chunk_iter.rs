use crate::ByteData;

use super::linked_root::LinkedRoot;

/// An iterator over the chunks of a [`ByteQueue`].
/// 
/// [`ByteQueue`]: crate::ByteQueue
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
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

impl<'a> DoubleEndedIterator for ChunkIter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.pop_back()
    }
}

impl<'a> ExactSizeIterator for ChunkIter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> core::iter::FusedIterator for ChunkIter<'a> {}

/// An iterator over the chunks of a [`ByteQueue`].
/// 
/// [`ByteQueue`]: crate::ByteQueue
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
pub struct StrChunkIter<'a>(LinkedRoot<'a>);

impl<'a> StrChunkIter<'a> {
    #[inline]
    pub(super) const fn new(data: LinkedRoot<'a>) -> Self {
        Self(data)
    }
}

impl<'a> Iterator for StrChunkIter<'a> {
    type Item = crate::StringData<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front().map(|x| unsafe { crate::StringData::from_bytedata_unchecked(x) })
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

impl<'a> DoubleEndedIterator for StrChunkIter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.pop_back().map(|x| unsafe { crate::StringData::from_bytedata_unchecked(x) })
    }
}

impl<'a> ExactSizeIterator for StrChunkIter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> core::iter::FusedIterator for StrChunkIter<'a> {}
