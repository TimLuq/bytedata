use core::{ops::RangeBounds, panic};

use crate::ByteData;

use super::byte_iter::ByteIter;
use super::linked_root::LinkedRoot;
use crate::queue::ChunkIter;

/// A queue of byte chunks.
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
#[derive(Clone, Default)]
#[allow(private_interfaces)]
pub struct ByteQueue<'a> {
    pub(crate) queue: LinkedRoot<'a>,
    pub(crate) remain: usize,
}

impl<'a> ByteQueue<'a> {
    /// Create a new empty `ByteQueue`.
    #[inline]
    pub const fn new() -> Self {
        Self {
            queue: LinkedRoot::new(),
            remain: 0,
        }
    }

    /// Create a new `ByteQueue` with a single chunk.
    #[inline]
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
    pub const fn is_full(&self) -> bool {
        false
    }

    #[cfg(not(feature = "alloc"))]
    /// Checks if the queue is full. When the feature `alloc` is enabled, this will always return `false`.
    #[inline]
    pub const fn is_full(&self) -> bool {
        self.queue.len() == self.queue.data.data.len()
    }

    /// Append bytes to the queue.
    pub fn push_back(&mut self, data: impl Into<ByteData<'a>>) {
        let data = data.into();
        if data.is_empty() {
            return;
        }
        self.remain += data.len();
        self.queue.push_back(data);
    }

    /// Prepend bytes into the queue.
    pub fn push_front(&mut self, data: impl Into<ByteData<'a>>) {
        let data = data.into();
        if data.is_empty() {
            return;
        }
        self.remain += data.len();
        self.queue.push_front(data);
    }

    /// Remove bytes from the front of the queue.
    #[inline]
    pub fn pop_front(&mut self) -> Option<ByteData<'a>> {
        let a = self.queue.pop_front()?;
        self.remain -= a.len();
        Some(a)
    }

    /// Remove bytes from the back of the queue.
    #[inline]
    pub fn pop_back(&mut self) -> Option<ByteData<'a>> {
        let a = self.queue.pop_back()?;
        self.remain -= a.len();
        Some(a)
    }

    /// Get the first chunk in the queue.
    #[inline]
    pub fn front(&self) -> Option<&ByteData<'a>> {
        self.queue.front()
    }

    /// Get the last chunk in the queue.
    #[inline]
    pub fn back(&self) -> Option<&ByteData<'a>> {
        self.queue.back()
    }

    /// Check if there are no bytes in the queue.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.remain == 0
    }

    /// The amount of bytes in the queue.
    #[inline]
    pub const fn len(&self) -> usize {
        self.remain
    }

    /// The amount of bytes in the queue.
    #[inline]
    pub const fn chunk_len(&self) -> usize {
        self.queue.len()
    }

    /// Iterates over each chunk in the queue.
    #[inline]
    pub fn chunks(&self) -> super::LinkedIter<'a, '_> {
        self.queue.iter()
    }

    /// Advance the queue by a certain amount of bytes.
    pub fn consume(&mut self, mut cnt: usize) {
        if cnt > self.len() {
            panic!("ByteData::advance: index out of bounds");
        }
        while cnt > 0 {
            let mut f = self.pop_front().unwrap();
            let len = f.len();
            if len > cnt {
                f.make_sliced(cnt..);
                self.push_front(f);
                return;
            }
            cnt -= len;
        }
    }

    /// Return a slice of the buffer.
    pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
        let mut max = match range.end_bound() {
            core::ops::Bound::Excluded(0) => return ByteQueue::new(),
            core::ops::Bound::Included(v) if *v < self.remain => *v + 1,
            core::ops::Bound::Included(v) => panic!(
                "slicing outside of max bound `..={}` where the maximum is {}",
                *v, self.remain
            ),
            core::ops::Bound::Excluded(v) if *v <= self.remain => *v,
            core::ops::Bound::Excluded(v) => panic!(
                "slicing outside of max bound `..{}` where the maximum is {}",
                *v, self.remain
            ),
            core::ops::Bound::Unbounded => self.remain,
        };
        let mut start = match range.start_bound() {
            core::ops::Bound::Included(v) => *v,
            core::ops::Bound::Excluded(v) => *v + 1,
            core::ops::Bound::Unbounded => 0,
        };
        if start > max {
            panic!(
                "slicing starting outside of maximum bound `{}..` where the maximum is {}",
                start, self.remain
            );
        }
        max -= start;
        let mut out = Self::new();
        for v in self.chunks() {
            if max == 0 {
                return out;
            }
            let l = v.len();
            if start >= l {
                start -= l;
                continue;
            }
            if start != 0 || max < l {
                let s = v.sliced(start..max.min(v.len() - start));
                let b = s.len();
                out.push_back(s);
                max -= b;
                start = 0;
                continue;
            }
            out.push_back(v.clone());
            max -= l;
        }
        out
    }

    /// Check if the queue starts with a certain byte sequence.
    #[inline]
    pub fn starts_with(&self, data: &[u8]) -> bool {
        if data.len() > self.len() {
            return false;
        }
        self.exists_at(0, data)
    }

    /// Check if the queue ends with a certain byte sequence.
    #[inline]
    pub fn ends_with(&self, data: &[u8]) -> bool {
        if data.len() > self.len() {
            return false;
        }
        self.exists_at(self.len() - data.len(), data)
    }

    /// Check if the queue contains a certain byte sequence at a certain index.
    pub fn exists_at(&self, mut index: usize, mut data: &[u8]) -> bool {
        for s in self.chunks() {
            let l = s.len();
            if index >= l {
                index -= l;
                continue;
            }
            let s = s.as_slice();
            let l = s.len();
            if l - index >= data.len() {
                return &s[index..index + data.len()] == data;
            }
            if &s[index..] != &data[..l - index] {
                return false;
            }
            data = &data[l - index..];
            index = 0;
        }
        index == 0 && data.is_empty()
    }

    /// Iterates over each chunk of bytedata in the queue.
    #[inline]
    pub fn into_iter(self) -> ChunkIter<'a> {
        self.queue.into_iter()
    }

    /// Iterates over each byte in the queue.
    #[inline]
    pub fn bytes(&self) -> ByteIter<'a, '_> {
        ByteIter::new(&self)
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
impl<'a> From<alloc::vec::Vec<u8>> for ByteQueue<'a> {
    #[inline]
    fn from(data: alloc::vec::Vec<u8>) -> Self {
        Self::with_item(data.into())
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<'a> From<alloc::string::String> for ByteQueue<'a> {
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
            alloc::borrow::Cow::Borrowed(v) => ByteData::from_borrowed(v.as_bytes()),
            alloc::borrow::Cow::Owned(v) => ByteData::from_owned(v.into_bytes()),
        })
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<'a> From<ByteQueue<'a>> for ByteData<'a> {
    fn from(mut data: ByteQueue<'a>) -> Self {
        let fst = match data.pop_front() {
            Some(v) => v,
            None => return ByteData::empty(),
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
    fn extend<T: IntoIterator<Item = ByteData<'a>>>(&mut self, iter: T) {
        for i in iter {
            self.push_back(i);
        }
    }
}

impl<'a> Extend<&'a [u8]> for ByteQueue<'a> {
    fn extend<T: IntoIterator<Item = &'a [u8]>>(&mut self, iter: T) {
        for i in iter {
            self.push_back(i);
        }
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<'a> Extend<alloc::vec::Vec<u8>> for ByteQueue<'a> {
    fn extend<T: IntoIterator<Item = alloc::vec::Vec<u8>>>(&mut self, iter: T) {
        for i in iter {
            self.push_back(i);
        }
    }
}

impl<'a, 'b> PartialEq<ByteQueue<'b>> for ByteQueue<'a> {
    #[inline]
    fn eq(&self, other: &ByteQueue<'b>) -> bool {
        let mut ai = self.queue.iter().map(|x| x.as_slice());
        let mut bi = other.queue.iter().map(|x| x.as_slice());
        let mut ad = ai.next().unwrap_or_default();
        let mut bd = bi.next().unwrap_or_default();
        loop {
            match (ad, bd) {
                (b"", b"") => return true,
                (b"", _) => return false,
                (_, b"") => return false,
                (mut a, mut b) => {
                    let l = a.len().min(b.len());
                    if a.len() == l {
                        ad = ai.next().unwrap_or_default();
                    } else {
                        ad = &a[l..];
                        a = &a[..l];
                    }
                    if b.len() == l {
                        bd = bi.next().unwrap_or_default();
                    } else {
                        bd = &b[l..];
                        b = &b[..l];
                    }
                    if a == b {
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
        let mut ai = self.queue.iter().map(|x| x.as_slice());
        let mut ad = ai.next().unwrap_or_default();
        loop {
            match (ad, other) {
                (b"", b"") => return true,
                (b"", _) => return false,
                (_, b"") => return false,
                (mut a, mut b) => {
                    let l = a.len().min(b.len());
                    if a.len() == l {
                        ad = ai.next().unwrap_or_default();
                    } else {
                        ad = &a[l..];
                        a = &a[..l];
                    }
                    if b.len() == l {
                        other = b"";
                    } else {
                        other = &b[l..];
                        b = &b[..l];
                    }
                    if a == b {
                        continue;
                    }
                    return false;
                }
            }
        }
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
        for s in self.chunks() {
            s.as_slice().hash(state);
        }
    }
}

impl<'a> core::cmp::Ord for ByteQueue<'a> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let mut ai = self.queue.iter().map(|x| x.as_slice());
        let mut bi = other.queue.iter().map(|x| x.as_slice());
        let mut ad = ai.next().unwrap_or_default();
        let mut bd = bi.next().unwrap_or_default();
        loop {
            match (ad, bd) {
                (b"", b"") => return core::cmp::Ordering::Equal,
                (b"", _) => return core::cmp::Ordering::Less,
                (_, b"") => return core::cmp::Ordering::Greater,
                (mut a, mut b) => {
                    let l = a.len().min(b.len());
                    if a.len() == l {
                        ad = ai.next().unwrap_or_default();
                    } else {
                        ad = &a[l..];
                        a = &a[..l];
                    }
                    if b.len() == l {
                        bd = bi.next().unwrap_or_default();
                    } else {
                        bd = &b[l..];
                        b = &b[..l];
                    }
                    match a.cmp(b) {
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
        let other = unsafe { core::mem::transmute::<&ByteQueue<'b>, &ByteQueue<'a>>(other) };
        Some(self.cmp(other))
    }
}

impl PartialOrd<[u8]> for ByteQueue<'_> {
    #[inline]
    fn partial_cmp(&self, mut other: &[u8]) -> Option<core::cmp::Ordering> {
        let mut ai = self.queue.iter().map(|x| x.as_slice());
        let mut ad = ai.next().unwrap_or_default();
        loop {
            match (ad, other) {
                (b"", b"") => return Some(core::cmp::Ordering::Equal),
                (b"", _) => return Some(core::cmp::Ordering::Less),
                (_, b"") => return Some(core::cmp::Ordering::Greater),
                (mut a, mut b) => {
                    let l = a.len().min(b.len());
                    if a.len() == l {
                        ad = ai.next().unwrap_or_default();
                    } else {
                        ad = &a[l..];
                        a = &a[..l];
                    }
                    if b.len() == l {
                        other = b"";
                    } else {
                        other = &b[l..];
                        b = &b[..l];
                    }
                    match a.cmp(b) {
                        core::cmp::Ordering::Equal => continue,
                        x => return Some(x),
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let r = crate::MultiByteStringRender::new(self);
        core::fmt::Debug::fmt(&r, f)
    }
}

impl core::fmt::LowerHex for ByteQueue<'_> {
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
