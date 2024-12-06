use core::{ops::RangeBounds, panic};

use crate::ByteData;

use super::byte_iter::{ByteIter, OwnedByteIter};
use super::linked_root::LinkedRoot;
use crate::queue::ChunkIter;

/// A queue of byte chunks.
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
#[derive(Clone)]
#[allow(private_interfaces)]
pub struct ByteQueue<'a> {
    pub(crate) queue: LinkedRoot<'a>,
    pub(crate) remain: usize,
}

impl<'a> ByteQueue<'a> {
    /// Create a new empty `ByteQueue`.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            queue: LinkedRoot::new(),
            remain: 0,
        }
    }

    /// Create a new `ByteQueue` with a single chunk.
    #[inline]
    #[must_use]
    pub const fn with_item(data: ByteData<'a>) -> Self {
        let remain = data.len();
        Self {
            queue: LinkedRoot::with_item(data),
            remain,
        }
    }

    #[cfg(feature = "alloc")]
    /// Checks if the queue is full. When the feature `alloc` is enabled, this will always return `false`.
    #[inline]
    #[must_use]
    pub const fn is_full(&self) -> bool {
        false
    }

    #[cfg(not(feature = "alloc"))]
    /// Checks if the queue is full. When the feature `alloc` is enabled, this will always return `false`.
    #[inline]
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.queue.len() == self.queue.data.data.len()
    }

    /// Append bytes to the queue.
    #[inline]
    pub fn push_back<I: Into<ByteData<'a>>>(&mut self, data: I) {
        let data = data.into();
        if data.is_empty() {
            return;
        }
        self.remain += data.len();
        self.queue.push_back(data);
    }

    /// Prepend bytes into the queue.
    #[inline]
    pub fn push_front<I: Into<ByteData<'a>>>(&mut self, data: I) {
        let data = data.into();
        if data.is_empty() {
            return;
        }
        self.remain += data.len();
        self.queue.push_front(data);
    }

    /// Remove bytes from the front of the queue.
    #[inline]
    #[must_use]
    pub fn pop_front(&mut self) -> Option<ByteData<'a>> {
        let aa = self.queue.pop_front()?;
        self.remain -= aa.len();
        Some(aa)
    }

    /// Remove bytes from the back of the queue.
    #[inline]
    #[must_use]
    pub fn pop_back(&mut self) -> Option<ByteData<'a>> {
        let aa = self.queue.pop_back()?;
        self.remain -= aa.len();
        Some(aa)
    }

    /// Get the first chunk in the queue.
    #[inline]
    #[must_use]
    pub fn front(&self) -> Option<&ByteData<'a>> {
        self.queue.front()
    }

    /// Get the last chunk in the queue.
    #[inline]
    #[must_use]
    pub fn back(&self) -> Option<&ByteData<'a>> {
        self.queue.back()
    }

    /// Check if there are no bytes in the queue.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.remain == 0
    }

    /// The amount of bytes in the queue.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.remain
    }

    /// The amount of chunks in the queue.
    #[inline]
    #[must_use]
    pub const fn chunk_len(&self) -> usize {
        self.queue.len()
    }

    /// Iterates over each chunk in the queue.
    #[inline]
    #[must_use]
    pub fn chunks(&self) -> super::LinkedIter<'a, '_> {
        self.queue.iter()
    }

    /// Advance the queue by a certain amount of bytes.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    #[inline]
    pub fn consume(&mut self, mut cnt: usize) {
        assert!(cnt <= self.len(), "ByteData::advance: index out of bounds");
        while cnt > 0 {
            #[allow(clippy::unwrap_used)]
            let mut front = self.pop_front().unwrap();
            let len = front.len();
            if len > cnt {
                front.make_sliced(cnt..);
                self.push_front(front);
                return;
            }
            cnt -= len;
        }
    }

    /// Check if there are remaining bytes in the queue.
    ///
    /// # Panics
    ///
    /// Panics if the range is out of bounds.
    pub(super) fn check_range(&self, range: impl RangeBounds<usize>) -> (usize, usize) {
        let max = match range.end_bound() {
            core::ops::Bound::Excluded(&0) => 0,
            core::ops::Bound::Included(vv) if *vv < self.remain => *vv + 1,
            #[allow(clippy::panic)]
            core::ops::Bound::Included(vv) => panic!(
                "slicing outside of max bound `..={}` where the maximum is {}",
                *vv, self.remain
            ),
            core::ops::Bound::Excluded(vv) if *vv <= self.remain => *vv,
            #[allow(clippy::panic)]
            core::ops::Bound::Excluded(vv) => panic!(
                "slicing outside of max bound `..{}` where the maximum is {}",
                *vv, self.remain
            ),
            core::ops::Bound::Unbounded => self.remain,
        };
        let start = match range.start_bound() {
            core::ops::Bound::Included(vv) => *vv,
            core::ops::Bound::Excluded(vv) => *vv + 1,
            core::ops::Bound::Unbounded => 0,
        };
        assert!(
            start <= max,
            "slicing starting outside of maximum bound `{}..` where the maximum is {}",
            start,
            self.remain
        );
        (start, max)
    }

    /// Return a slice of the buffer.
    #[allow(clippy::missing_inline_in_public_items)]
    #[must_use]
    pub fn slice<R: RangeBounds<usize>>(&self, range: R) -> Self {
        let (mut start, mut max) = self.check_range(range);
        if max == 0 {
            return Self::new();
        }
        max -= start;
        let mut out = Self::new();
        for vv in self.chunks() {
            if max == 0 {
                return out;
            }
            let len = vv.len();
            if start >= len {
                start -= len;
                continue;
            }
            if start != 0 || max < len {
                let sl = vv.sliced(start..(max + start).min(len));
                let bb = sl.len();
                out.push_back(sl);
                max -= bb;
                start = 0;
                continue;
            }
            out.push_back(vv.clone());
            max -= len;
        }
        out
    }

    /// Check if the queue starts with a certain byte sequence.
    #[inline]
    #[must_use]
    pub fn starts_with(&self, data: &[u8]) -> bool {
        if data.len() > self.len() {
            return false;
        }
        self.exists_at(0, data)
    }

    /// Check if the queue ends with a certain byte sequence.
    #[inline]
    #[must_use]
    pub fn ends_with(&self, data: &[u8]) -> bool {
        if data.len() > self.len() {
            return false;
        }
        self.exists_at(self.len() - data.len(), data)
    }

    /// Check if the queue contains a certain byte sequence at a certain index.
    #[allow(clippy::missing_inline_in_public_items)]
    #[must_use]
    pub fn exists_at(&self, mut index: usize, mut data: &[u8]) -> bool {
        for chunk in self.chunks() {
            let chunk = chunk.as_slice();
            let len = chunk.len();
            if index >= len {
                index -= len;
                continue;
            }
            if len - index >= data.len() {
                return &chunk[index..index + data.len()] == data;
            }
            if chunk[index..] != data[..len - index] {
                return false;
            }
            data = &data[len - index..];
            index = 0;
        }
        index == 0 && data.is_empty()
    }

    /// Check if the queue contains a certain byte sequence and return its starting position.
    #[inline]
    pub fn find_byte<F: FnMut(u8) -> bool>(&self, predicate: F) -> Option<usize> {
        self.bytes().position(predicate)
    }

    /// Check if the queue contains a certain byte sequence and return its starting position.
    #[inline]
    #[must_use]
    pub fn find_slice(&self, data: &[u8]) -> Option<usize> {
        self.find_slice_after(data, 0)
    }

    /// Check if the queue contains a certain byte sequence and return its starting position.
    #[inline]
    #[must_use]
    pub fn find_slice_after(&self, data: &[u8], start: usize) -> Option<usize> {
        self.find_slice_pos(data, 0, start, start)
            .map(|(av, _, _)| av)
    }

    /// Check if the queue contains a certain byte sequence and return its starting position.
    #[inline]
    pub(crate) fn find_slice_pos(
        &self,
        data: &[u8],
        start_chunk: usize,
        start_offset: usize,
        start_byte: usize,
    ) -> Option<(usize, usize, usize)> {
        if data.is_empty() {
            return Some((0, 0, 0));
        }
        self.find_slice_pos_nonempty(data, start_chunk, start_offset, start_byte)
    }

    /// Check if the queue contains a certain byte sequence and return its starting position.
    fn find_slice_pos_nonempty(
        &self,
        data: &[u8],
        mut start_chunk: usize,
        mut start_offset: usize,
        mut start_byte: usize,
    ) -> Option<(usize, usize, usize)> {
        debug_assert!(!data.is_empty(), "data is empty");
        // the first byte in the sequence to match possible sequences
        let first_byte = data[0];

        'outer: loop {
            let mut chunks = self.chunks().skip(start_chunk);
            let mut sect = chunks.next()?;
            let len = sect.len();
            if start_offset >= len {
                start_offset -= len;
                start_chunk += 1;
                continue;
            }
            let marked_byte = loop {
                let sl = &sect.as_slice()[start_offset..];
                let Some(av) = sl.iter().position(|x| *x == first_byte) else {
                    sect = match chunks.next() {
                        Some(vv) => vv,
                        None => return None,
                    };
                    start_offset = 0;
                    start_chunk += 1;
                    continue;
                };
                start_offset += av;
                start_byte += av;
                break start_byte;
            };
            let marked_chunk = start_chunk;
            let marked_offset = start_offset;
            let mut found_in = false;
            let mut skipped_bytes = 0;
            let data = data.iter().skip(1);
            let mut sl = sect.as_slice()[(start_offset + 1)..].iter();
            for av in data {
                let bv = loop {
                    let nxt = sl.next();
                    let Some(bv) = nxt else {
                        sect = match chunks.next() {
                            Some(vv) => vv,
                            None => return None,
                        };
                        sl = sect.as_slice().iter();
                        continue;
                    };
                    break *bv;
                };
                if !found_in && bv == first_byte {
                    found_in = true;
                    start_byte += skipped_bytes;
                    start_offset += skipped_bytes;
                    start_offset -= 1;
                }
                if bv == *av {
                    skipped_bytes += 1;
                    continue;
                }
                if !found_in {
                    start_byte += skipped_bytes;
                    start_offset += skipped_bytes;
                }
                continue 'outer;
            }
            return Some((marked_byte, marked_chunk, marked_offset));
        }
    }

    /// Split the queue on a certain byte sequence.
    #[inline]
    #[must_use]
    pub const fn split_on<'b>(&'b self, needle: &'b [u8]) -> super::SplitOn<'a, 'b> {
        super::SplitOn::new(self, needle, 0)
    }

    /// Split the queue on a certain byte sequence.
    #[inline]
    #[must_use]
    pub const fn splitn_on<'b>(&'b self, needle: &'b [u8], max: usize) -> super::SplitOn<'a, 'b> {
        super::SplitOn::new(self, needle, max)
    }

    /// Iterates over each chunk of bytedata in the queue.
    #[inline]
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn into_iter(self) -> ChunkIter<'a> {
        self.queue.into_iter()
    }

    /// Iterates over each borrowed chunk of bytedata in the queue.
    #[inline]
    #[must_use]
    pub fn iter(&self) -> crate::queue::LinkedIter<'a, '_> {
        self.queue.iter()
    }

    /// Iterates over each byte in the queue.
    #[inline]
    #[must_use]
    pub fn bytes(&self) -> ByteIter<'a, '_> {
        ByteIter::new(self)
    }

    /// Iterates over each byte in the queue.
    #[inline]
    #[must_use]
    pub const fn into_bytes(self) -> OwnedByteIter<'a> {
        OwnedByteIter::new(self)
    }

    /// Adds another `ByteQueue`'s chunks to this queue. May be optimized in the future.
    #[inline]
    pub fn append(&mut self, other: Self) {
        // TODO: optimize by adding full regions instead of just chunks at to save on region allocations
        self.extend(other.into_iter());
    }

    /// Split the queue at a certain index.
    /// This will return the part of the queue after the index `[at, len)` and keep everything before the position in the original queue `[0, at)`.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    #[inline]
    #[must_use = "use `drain` if the result is unused"]
    pub fn split_off(&mut self, at: usize) -> Self {
        fn inner<'a>(this: &mut ByteQueue<'a>, at: usize) -> ByteQueue<'a> {
            let mut out = ByteQueue::new();
            let mut remain = this.len() - at;
            while let Some(av) = this.pop_back() {
                let len = av.len();
                if len > remain {
                    let (av, bv) = av.split_at(len - remain);
                    this.push_back(av);
                    out.push_front(bv);
                    return out;
                }
                remain -= len;
                out.push_front(av);
                if remain != 0 {
                    continue;
                }
                break;
            }
            out
        }
        if at == 0 {
            return core::mem::replace(self, Self::new());
        }
        let len = self.len();
        if at == len {
            return Self::new();
        }
        assert!(at <= len, "ByteQueue::split_off: index out of bounds");
        inner(self, at)
    }

    /// Drain a range of bytes from the queue. The returned iterator will remove the bytes from the queue when dropped.
    #[inline]
    pub fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> super::DrainBytes<'a, '_> {
        let (start, end) = self.check_range(range);
        super::DrainBytes::new(self, start, end)
    }

    /// Move data to the returned `ByteQueue` until the byte predicate returns `false`.
    #[inline]
    #[must_use]
    pub fn take_while<F: FnMut(u8) -> bool>(&mut self, mut fun: F) -> Self {
        let Some(position) = self.find_byte(|bte| !fun(bte)) else {
            return core::mem::replace(self, Self::new());
        };
        if position == 0 {
            return Self::new();
        }
        let mut ret: Self = self.split_off(position);
        core::mem::swap(self, &mut ret);
        ret
    }
}

impl<'a> From<ByteData<'a>> for ByteQueue<'a> {
    #[inline]
    fn from(data: ByteData<'a>) -> Self {
        Self::with_item(data)
    }
}

impl<'a> From<&'a [u8]> for ByteQueue<'a> {
    #[inline]
    fn from(data: &'a [u8]) -> Self {
        Self::with_item(data.into())
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl From<alloc::vec::Vec<u8>> for ByteQueue<'_> {
    #[inline]
    fn from(data: alloc::vec::Vec<u8>) -> Self {
        Self::with_item(data.into())
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl From<alloc::string::String> for ByteQueue<'_> {
    #[inline]
    fn from(data: alloc::string::String) -> Self {
        Self::with_item(data.into())
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<'a> From<alloc::borrow::Cow<'a, [u8]>> for ByteQueue<'a> {
    #[inline]
    fn from(data: alloc::borrow::Cow<'a, [u8]>) -> Self {
        Self::with_item(ByteData::from_cow(data))
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<'a> From<alloc::borrow::Cow<'a, str>> for ByteQueue<'a> {
    #[inline]
    fn from(data: alloc::borrow::Cow<'a, str>) -> Self {
        Self::with_item(match data {
            alloc::borrow::Cow::Borrowed(val) => ByteData::from_borrowed(val.as_bytes()),
            alloc::borrow::Cow::Owned(val) => ByteData::from_owned(val.into_bytes()),
        })
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<'a> From<ByteQueue<'a>> for ByteData<'a> {
    #[inline]
    fn from(mut data: ByteQueue<'a>) -> Self {
        let Some(fst) = data.pop_front() else {
            return ByteData::empty();
        };
        if data.is_empty() {
            return fst;
        }
        let mut out = crate::SharedBytesBuilder::with_capacity(fst.len() + data.len());
        out.extend_from_slice(fst.as_slice());
        for i in data {
            out.extend_from_slice(i.as_slice());
        }
        ByteData::from_shared(out.build())
    }
}

impl<'a> FromIterator<ByteData<'a>> for ByteQueue<'a> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = ByteData<'a>>>(iter: T) -> Self {
        let mut out = Self::new();
        for i in iter {
            out.push_back(i);
        }
        out
    }
}

impl<'a> IntoIterator for ByteQueue<'a> {
    type Item = ByteData<'a>;
    type IntoIter = ChunkIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ChunkIter::new(self.queue)
    }
}

impl<'a: 'b, 'b> IntoIterator for &'b ByteQueue<'a> {
    type Item = &'b ByteData<'a>;
    type IntoIter = crate::queue::LinkedIter<'a, 'b>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.queue.iter()
    }
}

impl<'a> Extend<ByteData<'a>> for ByteQueue<'a> {
    #[inline]
    fn extend<T: IntoIterator<Item = ByteData<'a>>>(&mut self, iter: T) {
        for i in iter {
            self.push_back(i);
        }
    }
}

impl<'a> Extend<&'a [u8]> for ByteQueue<'a> {
    #[inline]
    fn extend<T: IntoIterator<Item = &'a [u8]>>(&mut self, iter: T) {
        for i in iter {
            self.push_back(i);
        }
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl Extend<alloc::vec::Vec<u8>> for ByteQueue<'_> {
    #[inline]
    fn extend<T: IntoIterator<Item = alloc::vec::Vec<u8>>>(&mut self, iter: T) {
        for i in iter {
            self.push_back(i);
        }
    }
}

impl<'a> Extend<crate::StringData<'a>> for ByteQueue<'a> {
    #[inline]
    fn extend<T: IntoIterator<Item = crate::StringData<'a>>>(&mut self, iter: T) {
        for i in iter {
            self.queue.push_back(i.into_bytedata());
        }
    }
}

impl<'a> Extend<&'a str> for ByteQueue<'a> {
    #[inline]
    fn extend<T: IntoIterator<Item = &'a str>>(&mut self, iter: T) {
        for i in iter {
            self.queue.push_back(ByteData::from_borrowed(i.as_bytes()));
        }
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl Extend<alloc::string::String> for ByteQueue<'_> {
    #[inline]
    fn extend<T: IntoIterator<Item = alloc::string::String>>(&mut self, iter: T) {
        for i in iter {
            self.queue.push_back(ByteData::from_owned(i.into_bytes()));
        }
    }
}

impl<'b> PartialEq<ByteQueue<'b>> for ByteQueue<'_> {
    #[inline]
    fn eq(&self, other: &ByteQueue<'b>) -> bool {
        let mut ai = self.queue.iter().map(crate::ByteData::as_slice);
        let mut bi = other.queue.iter().map(crate::ByteData::as_slice);
        let mut ad = ai.next().unwrap_or_default();
        let mut bd = bi.next().unwrap_or_default();
        loop {
            match (ad, bd) {
                (b"", b"") => return true,
                (b"", _) | (_, b"") => return false,
                (mut av, mut bv) => {
                    let len = av.len().min(bv.len());
                    if av.len() == len {
                        ad = ai.next().unwrap_or_default();
                    } else {
                        ad = &av[len..];
                        av = &av[..len];
                    }
                    if bv.len() == len {
                        bd = bi.next().unwrap_or_default();
                    } else {
                        bd = &bv[len..];
                        bv = &bv[..len];
                    }
                    if av == bv {
                        continue;
                    }
                    return false;
                }
            }
        }
    }
}

impl PartialEq<[u8]> for ByteQueue<'_> {
    #[inline]
    fn eq(&self, mut other: &[u8]) -> bool {
        let mut ai = self.queue.iter().map(crate::ByteData::as_slice);
        let mut ad = ai.next().unwrap_or_default();
        loop {
            match (ad, other) {
                (b"", b"") => return true,
                (b"", _) | (_, b"") => return false,
                (mut av, mut bv) => {
                    let len = av.len().min(bv.len());
                    if av.len() == len {
                        ad = ai.next().unwrap_or_default();
                    } else {
                        ad = &av[len..];
                        av = &av[..len];
                    }
                    if bv.len() == len {
                        other = b"";
                    } else {
                        other = &bv[len..];
                        bv = &bv[..len];
                    }
                    if av == bv {
                        continue;
                    }
                    return false;
                }
            }
        }
    }
}

impl<'b> PartialEq<crate::ByteData<'b>> for ByteQueue<'_> {
    #[inline]
    fn eq(&self, other: &crate::ByteData<'b>) -> bool {
        self.eq(other.as_slice())
    }
}

impl<'b> PartialEq<ByteQueue<'b>> for crate::ByteData<'_> {
    #[inline]
    fn eq(&self, other: &ByteQueue<'b>) -> bool {
        other.eq(self.as_slice())
    }
}

impl<'b> PartialEq<crate::StringData<'b>> for ByteQueue<'_> {
    #[inline]
    fn eq(&self, other: &crate::StringData<'b>) -> bool {
        self.eq(other.as_bytes())
    }
}

impl<'b> PartialEq<ByteQueue<'b>> for crate::StringData<'_> {
    #[inline]
    fn eq(&self, other: &ByteQueue<'b>) -> bool {
        other.eq(self.as_bytes())
    }
}

impl<'b> PartialEq<&'b [u8]> for ByteQueue<'_> {
    #[inline]
    fn eq(&self, other: &&'b [u8]) -> bool {
        self.eq(*other)
    }
}

impl PartialEq<ByteQueue<'_>> for [u8] {
    #[inline]
    fn eq(&self, other: &ByteQueue<'_>) -> bool {
        other.eq(self)
    }
}

impl<'b> PartialEq<&'b str> for ByteQueue<'_> {
    #[inline]
    fn eq(&self, other: &&'b str) -> bool {
        self.eq(other.as_bytes())
    }
}

impl PartialEq<str> for ByteQueue<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.eq(other.as_bytes())
    }
}

impl PartialEq<ByteQueue<'_>> for str {
    #[inline]
    fn eq(&self, other: &ByteQueue<'_>) -> bool {
        other.eq(self.as_bytes())
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
impl PartialEq<alloc::vec::Vec<u8>> for ByteQueue<'_> {
    #[inline]
    fn eq(&self, other: &alloc::vec::Vec<u8>) -> bool {
        self.eq(other.as_slice())
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
impl PartialEq<ByteQueue<'_>> for alloc::vec::Vec<u8> {
    #[inline]
    fn eq(&self, other: &ByteQueue<'_>) -> bool {
        other.eq(self.as_slice())
    }
}

impl Eq for ByteQueue<'_> {}

impl core::hash::Hash for ByteQueue<'_> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        for chunk in self.chunks() {
            chunk.as_slice().hash(state);
        }
    }
}

impl core::cmp::Ord for ByteQueue<'_> {
    #[allow(clippy::missing_inline_in_public_items)]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let mut ai = self.queue.iter().map(crate::ByteData::as_slice);
        let mut bi = other.queue.iter().map(crate::ByteData::as_slice);
        let mut ad = ai.next().unwrap_or_default();
        let mut bd = bi.next().unwrap_or_default();
        loop {
            match (ad, bd) {
                (b"", b"") => return core::cmp::Ordering::Equal,
                (b"", _) => return core::cmp::Ordering::Less,
                (_, b"") => return core::cmp::Ordering::Greater,
                (mut av, mut bv) => {
                    let len = av.len().min(bv.len());
                    if av.len() == len {
                        ad = ai.next().unwrap_or_default();
                    } else {
                        ad = &av[len..];
                        av = &av[..len];
                    }
                    if bv.len() == len {
                        bd = bi.next().unwrap_or_default();
                    } else {
                        bd = &bv[len..];
                        bv = &bv[..len];
                    }
                    #[allow(clippy::wildcard_enum_match_arm)]
                    match av.cmp(bv) {
                        core::cmp::Ordering::Equal => continue,
                        x => return x,
                    }
                }
            }
        }
    }
}

impl<'a, 'b> PartialOrd<ByteQueue<'b>> for ByteQueue<'a> {
    #[inline]
    fn partial_cmp(&self, other: &ByteQueue<'b>) -> Option<core::cmp::Ordering> {
        // SAFETY: for the purpose of comparison, the lifetime of the other queue is temporarily extended to the lifetime of the self queue
        let other = unsafe { core::mem::transmute::<&ByteQueue<'b>, &ByteQueue<'a>>(other) };
        Some(self.cmp(other))
    }
}

impl PartialOrd<[u8]> for ByteQueue<'_> {
    #[inline]
    fn partial_cmp(&self, mut other: &[u8]) -> Option<core::cmp::Ordering> {
        let mut ai = self.queue.iter().map(crate::ByteData::as_slice);
        let mut ad = ai.next().unwrap_or_default();
        loop {
            match (ad, other) {
                (b"", b"") => return Some(core::cmp::Ordering::Equal),
                (b"", _) => return Some(core::cmp::Ordering::Less),
                (_, b"") => return Some(core::cmp::Ordering::Greater),
                (mut av, mut bv) => {
                    let len = av.len().min(bv.len());
                    if av.len() == len {
                        ad = ai.next().unwrap_or_default();
                    } else {
                        ad = &av[len..];
                        av = &av[..len];
                    }
                    if bv.len() == len {
                        other = b"";
                    } else {
                        other = &bv[len..];
                        bv = &bv[..len];
                    }
                    #[allow(clippy::wildcard_enum_match_arm)]
                    match av.cmp(bv) {
                        core::cmp::Ordering::Equal => continue,
                        xy => return Some(xy),
                    }
                }
            }
        }
    }
}

impl PartialOrd<ByteQueue<'_>> for [u8] {
    #[inline]
    fn partial_cmp(&self, other: &ByteQueue<'_>) -> Option<core::cmp::Ordering> {
        match other.partial_cmp(self) {
            Some(core::cmp::Ordering::Less) => Some(core::cmp::Ordering::Greater),
            Some(core::cmp::Ordering::Greater) => Some(core::cmp::Ordering::Less),
            x => x,
        }
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
impl PartialOrd<alloc::vec::Vec<u8>> for ByteQueue<'_> {
    #[inline]
    fn partial_cmp(&self, other: &alloc::vec::Vec<u8>) -> Option<core::cmp::Ordering> {
        self.partial_cmp(other.as_slice())
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
impl PartialOrd<ByteQueue<'_>> for alloc::vec::Vec<u8> {
    #[inline]
    fn partial_cmp(&self, other: &ByteQueue<'_>) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(other)
    }
}

impl core::fmt::Debug for ByteQueue<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let r = crate::MultiByteStringRender::new(self);
        core::fmt::Debug::fmt(&r, f)
    }
}

impl core::fmt::LowerHex for ByteQueue<'_> {
    #[allow(clippy::missing_inline_in_public_items)]
    #[allow(clippy::min_ident_chars)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(w) = f.width() {
            if w > self.len() * 2 {
                for _ in 0..w - self.len() * 2 {
                    core::fmt::Write::write_str(f, "0")?;
                }
            }
        }
        for s in self.chunks() {
            let s = s.as_slice();
            let mut i = 0;
            while i < s.len() {
                write!(f, "{:02x}", s[i])?;
                i += 1;
            }
        }
        Ok(())
    }
}

impl core::fmt::UpperHex for ByteQueue<'_> {
    #[allow(clippy::missing_inline_in_public_items)]
    #[allow(clippy::min_ident_chars)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(w) = f.width() {
            if w > self.len() * 2 {
                for _ in 0..w - self.len() * 2 {
                    core::fmt::Write::write_str(f, "0")?;
                }
            }
        }
        for s in self.chunks() {
            let s = s.as_slice();
            let mut i = 0;
            while i < s.len() {
                write!(f, "{:02X}", s[i])?;
                i += 1;
            }
        }
        Ok(())
    }
}

impl Default for ByteQueue<'_> {
    #[inline]
    fn default() -> Self {
        ByteQueue::new()
    }
}
