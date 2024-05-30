use core::{mem::MaybeUninit, ops::RangeBounds, panic};

use crate::ByteData;

#[cfg(feature = "alloc")]
struct LinkedNodeLeaf<'a> {
    prev: *mut LinkedNodeLeaf<'a>,
    data: LinkedNodeData<'a>,
    next: *mut LinkedNodeLeaf<'a>,
}
#[cfg(feature = "alloc")]
unsafe impl Send for LinkedNodeLeaf<'_> {}

#[cfg(feature = "alloc")]
unsafe impl Sync for LinkedNodeLeaf<'_> {}

#[cfg(feature = "alloc")]
impl<'a> LinkedNodeLeaf<'a> {
    fn with_item(data: ByteData<'a>) -> Self {
        Self {
            prev: core::ptr::null_mut(),
            data: LinkedNodeData::with_item(data),
            next: core::ptr::null_mut(),
        }
    }
}

struct LinkedNodeData<'a> {
    data: [MaybeUninit<ByteData<'a>>; 8],
    beg: u8,
    len: u8,
}
impl<'a> LinkedNodeData<'a> {
    #[cfg(not(feature = "alloc"))]
    const fn new() -> Self {
        Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            beg: 0,
            len: 0,
        }
    }

    #[cfg(feature = "alloc")]
    fn with_item(data: ByteData<'a>) -> Self {
        let mut r = Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            beg: 0,
            len: 1,
        };
        r.data[0].write(data);
        r
    }

    fn push_back(&mut self, data: ByteData<'a>) -> Result<(), ByteData<'a>> {
        if self.len >= self.data.len() as u8 {
            return Err(data);
        }
        self.data[(self.beg as usize + self.len as usize) % self.data.len()].write(data);
        self.len += 1;
        Ok(())
    }

    fn push_front(&mut self, data: ByteData<'a>) -> Result<(), ByteData<'a>> {
        if self.len >= self.data.len() as u8 {
            return Err(data);
        }
        let i = (self.beg as usize + (self.data.len() - 1)) % self.data.len();
        self.data[i].write(data);
        self.beg = i as u8;
        self.len += 1;
        Ok(())
    }

    fn pop_back(&mut self) -> Option<ByteData<'a>> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let i = (self.beg as usize + self.len as usize) % self.data.len();
        Some(unsafe { self.data[i].as_mut_ptr().read() })
    }

    fn pop_front(&mut self) -> Option<ByteData<'a>> {
        if self.len == 0 {
            return None;
        }
        let i = self.beg as usize;
        self.beg = (self.beg + 1) % self.data.len() as u8;
        self.len -= 1;
        Some(unsafe { self.data[i].as_mut_ptr().read() })
    }

    fn front(&self) -> Option<&ByteData<'a>> {
        if self.len == 0 {
            return None;
        }
        Some(unsafe { self.data[self.beg as usize].assume_init_ref() })
    }

    fn back(&self) -> Option<&ByteData<'a>> {
        if self.len == 0 {
            return None;
        }
        let i = (self.beg as usize + self.len as usize - 1) % self.data.len();
        Some(unsafe { self.data[i].assume_init_ref() })
    }

    fn front_mut(&mut self) -> Option<&mut ByteData<'a>> {
        if self.len == 0 {
            return None;
        }
        Some(unsafe { self.data[self.beg as usize].assume_init_mut() })
    }

    fn back_mut(&mut self) -> Option<&mut ByteData<'a>> {
        if self.len == 0 {
            return None;
        }
        let i = (self.beg as usize + self.len as usize - 1) % self.data.len();
        Some(unsafe { self.data[i].assume_init_mut() })
    }
}

impl Drop for LinkedNodeData<'_> {
    fn drop(&mut self) {
        let mut b = self.beg as usize;
        for _ in 0..self.len {
            unsafe {
                self.data[b].as_mut_ptr().drop_in_place();
            }
            b = (b + 1) % self.data.len();
        }
    }
}

#[cfg(feature = "alloc")]
pub(crate) struct LinkedRoot<'a> {
    first: *mut LinkedNodeLeaf<'a>,
    last: *mut LinkedNodeLeaf<'a>,
    count: usize,
}

#[cfg(feature = "alloc")]
unsafe impl Send for LinkedRoot<'_> {}

#[cfg(feature = "alloc")]
unsafe impl Sync for LinkedRoot<'_> {}

#[cfg(not(feature = "alloc"))]
#[allow(private_interfaces)]
struct LinkedRoot<'a> {
    data: LinkedNodeData<'a>,
}

impl<'a> LinkedRoot<'a> {
    #[cfg(feature = "alloc")]
    #[inline]
    const fn new() -> Self {
        Self {
            first: core::ptr::null_mut(),
            last: core::ptr::null_mut(),
            count: 0,
        }
    }
    #[cfg(not(feature = "alloc"))]
    #[inline]
    const fn new() -> Self {
        Self {
            data: LinkedNodeData::new(),
        }
    }
    #[inline]
    fn with_item(item: crate::ByteData<'a>) -> Self {
        // TODO: make const
        let mut me = Self::new();
        me.push_back(item);
        me
    }

    #[cfg(feature = "alloc")]
    #[inline]
    const fn len(&self) -> usize {
        self.count
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    const fn len(&self) -> usize {
        self.data.len as usize
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn first(&self) -> Option<&LinkedNodeData<'a>> {
        unsafe { self.first.as_ref().map(|x| &x.data) }
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn first_mut(&mut self) -> Option<&mut LinkedNodeData<'a>> {
        unsafe { self.first.as_mut().map(|x| &mut x.data) }
    }
    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn first_mut(&mut self) -> Option<&mut LinkedNodeData<'a>> {
        Some(&mut self.data)
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn last(&self) -> Option<&LinkedNodeData<'a>> {
        unsafe { self.first.as_ref().map(|x| &x.data) }
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn last_mut(&mut self) -> Option<&mut LinkedNodeData<'a>> {
        unsafe { self.first.as_mut().map(|x| &mut x.data) }
    }
    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn last_mut(&mut self) -> Option<&mut LinkedNodeData<'a>> {
        Some(&mut self.data)
    }

    fn push_back(&mut self, mut data: ByteData<'a>) {
        #[allow(unused_assignments)]
        if let Some(last) = self.last_mut() {
            data = match last.push_back(data) {
                Ok(()) => {
                    #[cfg(feature = "alloc")]
                    {
                        self.count += 1;
                    }
                    return;
                }
                Err(v) => v,
            };
        }
        #[cfg(not(feature = "alloc"))]
        panic!(
            "ByteQueue::push_back: out of space, use `alloc` feature to enable dynamic allocation"
        );
        #[cfg(feature = "alloc")]
        self.push_back_alloc(data)
    }

    #[cfg(feature = "alloc")]
    fn push_back_alloc(&mut self, data: ByteData<'a>) {
        let node = alloc::boxed::Box::new(LinkedNodeLeaf::with_item(data));
        let node = alloc::boxed::Box::into_raw(node);
        self.count += 1;
        if let Some(last) = unsafe { self.last.as_mut() } {
            last.next = node;
            unsafe { &mut *node }.prev = last;
            self.last = node;
            return;
        }
        self.first = node;
        self.last = node;
    }

    fn push_front(&mut self, mut data: ByteData<'a>) {
        #[allow(unused_assignments)]
        if let Some(first) = self.first_mut() {
            data = match first.push_front(data) {
                Ok(()) => {
                    #[cfg(feature = "alloc")]
                    {
                        self.count += 1;
                    }
                    return;
                }
                Err(v) => v,
            };
        }
        #[cfg(not(feature = "alloc"))]
        panic!(
            "ByteQueue::push_front: out of space, use `alloc` feature to enable dynamic allocation"
        );
        #[cfg(feature = "alloc")]
        self.push_front_alloc(data)
    }

    #[cfg(feature = "alloc")]
    fn push_front_alloc(&mut self, data: ByteData<'a>) {
        let node = alloc::boxed::Box::new(LinkedNodeLeaf::with_item(data));
        let node = alloc::boxed::Box::into_raw(node);
        self.count += 1;
        if let Some(first) = unsafe { self.first.as_mut() } {
            first.prev = node;
            unsafe { &mut *node }.next = first;
            self.first = node;
            return;
        }
        self.first = node;
        self.last = node;
    }

    #[cfg(feature = "alloc")]
    fn pop_back(&mut self) -> Option<ByteData<'a>> {
        let last = unsafe { self.last.as_mut()? };
        let r = last.data.pop_back()?;
        self.count -= 1;
        if last.data.len == 0 {
            if let Some(prev) = unsafe { last.prev.as_mut() } {
                prev.next = core::ptr::null_mut();
                self.last = prev;
                core::mem::drop(unsafe { alloc::boxed::Box::from_raw(last) });
            }
        }
        Some(r)
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn pop_back(&mut self) -> Option<ByteData<'a>> {
        self.data.pop_back()
    }

    #[cfg(feature = "alloc")]
    fn pop_front(&mut self) -> Option<ByteData<'a>> {
        let first = unsafe { self.first.as_mut()? };
        let r = first.data.pop_front();
        if first.data.len == 0 {
            if let Some(next) = unsafe { first.next.as_mut() } {
                next.prev = core::ptr::null_mut();
                self.first = next;
                core::mem::drop(unsafe { alloc::boxed::Box::from_raw(first) });
            }
        }
        r
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn pop_front(&mut self) -> Option<ByteData<'a>> {
        self.data.pop_front()
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn front(&self) -> Option<&ByteData<'a>> {
        self.first().and_then(|x| x.front())
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn front(&self) -> Option<&ByteData<'a>> {
        self.data.front()
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn front_mut(&mut self) -> Option<&mut ByteData<'a>> {
        self.first_mut().and_then(|x| x.front_mut())
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn front_mut(&mut self) -> Option<&mut ByteData<'a>> {
        self.data.front_mut()
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn back(&self) -> Option<&ByteData<'a>> {
        self.last().and_then(|x| x.back())
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn back(&self) -> Option<&ByteData<'a>> {
        self.data.back()
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn back_mut(&mut self) -> Option<&mut ByteData<'a>> {
        self.last_mut().and_then(|x| x.back_mut())
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn back_mut(&mut self) -> Option<&mut ByteData<'a>> {
        self.data.back_mut()
    }

    #[inline]
    fn from_iter<I: Iterator<Item = ByteData<'a>>>(iter: I) -> Self {
        let mut s = Self::new();
        s.extend(iter);
        s
    }

    fn extend<I: Iterator<Item = ByteData<'a>>>(&mut self, iter: I) {
        for i in iter {
            self.push_back(i);
        }
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn iter(&self) -> LinkedIter<'a, '_> {
        LinkedIter::new(unsafe { self.first.as_ref() })
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn iter(&self) -> LinkedIter<'a, '_> {
        LinkedIter::new(&self.data)
    }

    #[inline]
    fn into_iter(self) -> ChunkIter<'a> {
        ChunkIter(self)
    }
}

pub struct ChunkIter<'a>(LinkedRoot<'a>);

impl<'a> Iterator for ChunkIter<'a> {
    type Item = ByteData<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.len(), Some(self.0.len()))
    }

    fn count(self) -> usize {
        self.0.len()
    }
}

impl<'a> DoubleEndedIterator for ChunkIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.pop_back()
    }
}

impl<'a> ExactSizeIterator for ChunkIter<'a> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> core::iter::FusedIterator for ChunkIter<'a> {}

pub struct LinkedIter<'a: 'b, 'b> {
    #[cfg(feature = "alloc")]
    node: Option<&'b LinkedNodeLeaf<'a>>,
    #[cfg(not(feature = "alloc"))]
    data: &'b LinkedNodeData<'a>,
    offset: usize,
}
impl<'a: 'b, 'b> LinkedIter<'a, 'b> {
    #[cfg(feature = "alloc")]
    fn new(node: Option<&'b LinkedNodeLeaf<'a>>) -> Self {
        Self { node, offset: 0 }
    }
    #[cfg(not(feature = "alloc"))]
    fn new(data: &'b LinkedNodeData<'a>) -> Self {
        Self { data, offset: 0 }
    }
}

impl<'a: 'b, 'b> Iterator for LinkedIter<'a, 'b> {
    type Item = &'b ByteData<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        #[cfg(feature = "alloc")]
        let node = self.node?;
        #[cfg(feature = "alloc")]
        let data = &node.data;
        #[cfg(not(feature = "alloc"))]
        let data = self.data;

        if self.offset < data.len as usize {
            let r: &MaybeUninit<ByteData<'a>> =
                &data.data[(data.beg as usize + self.offset) % data.data.len()];
            self.offset += 1;
            return Some(unsafe { r.assume_init_ref() });
        }

        #[cfg(not(feature = "alloc"))]
        return None;

        #[cfg(feature = "alloc")]
        {
            self.node = unsafe { node.next.as_ref() };
            self.offset = 0;
            self.next()
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        #[cfg(feature = "alloc")]
        let mut node = match self.node {
            Some(v) => v,
            None => return (0, Some(0)),
        };
        #[cfg(feature = "alloc")]
        let data = &node.data;
        #[cfg(not(feature = "alloc"))]
        let data = self.data;

        let len = data.len as usize - self.offset;
        #[cfg(feature = "alloc")]
        let mut len = len;
        #[cfg(feature = "alloc")]
        while let Some(a) = unsafe { node.next.as_ref() } {
            len += a.data.len as usize;
            node = a;
            continue;
        }

        (len, Some(len))
    }

    fn count(self) -> usize {
        self.size_hint().0
    }
}

impl<'a: 'b, 'b> ExactSizeIterator for LinkedIter<'a, 'b> {
    fn len(&self) -> usize {
        self.size_hint().0
    }
}

impl Clone for LinkedRoot<'_> {
    fn clone(&self) -> Self {
        Self::from_iter(self.iter().map(|x| x.clone()))
    }
}

impl Default for LinkedRoot<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for LinkedRoot<'_> {
    #[cfg(feature = "alloc")]
    fn drop(&mut self) {
        let mut node = self.first;
        while let Some(n) = unsafe { node.as_mut() } {
            let n = unsafe { alloc::boxed::Box::from_raw(n) };
            node = n.next;
            core::mem::drop(n);
        }
    }

    #[cfg(not(feature = "alloc"))]
    fn drop(&mut self) {
        unsafe { core::ptr::drop_in_place(&mut self.data) };
        self.data.len = 0;
    }
}

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

    fn with_item(data: ByteData<'a>) -> Self {
        let remain = data.len();
        // TODO: make const
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

    /// Get the first chunk in the queue.
    #[inline]
    pub fn front_mut(&mut self) -> Option<&mut ByteData<'a>> {
        self.queue.front_mut()
    }

    /// Get the last chunk in the queue.
    #[inline]
    pub fn back(&self) -> Option<&ByteData<'a>> {
        self.queue.back()
    }

    /// Get the last chunk in the queue.
    #[inline]
    pub fn back_mut(&mut self) -> Option<&mut ByteData<'a>> {
        self.queue.back_mut()
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
    pub fn chunk_len(&self) -> usize {
        self.queue.len()
    }

    /// Iterates over each chunk in the queue.
    #[inline]
    pub fn chunks(&self) -> impl Iterator<Item = &ByteData<'a>> {
        self.queue.iter()
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
        let mut it = self.chunks();
        while let Some(v) = it.next() {
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

    /// Iterates over each byte in the queue.
    #[inline]
    pub fn into_iter(self) -> impl Iterator<Item = ByteData<'a>> {
        self.queue.into_iter()
    }
}

impl<'a> From<ByteData<'a>> for ByteQueue<'a> {
    fn from(data: ByteData<'a>) -> Self {
        Self::with_item(data)
    }
}

impl<'a> From<&'a [u8]> for ByteQueue<'a> {
    fn from(data: &'a [u8]) -> Self {
        Self::with_item(data.into())
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<'a> From<alloc::vec::Vec<u8>> for ByteQueue<'a> {
    fn from(data: alloc::vec::Vec<u8>) -> Self {
        Self::with_item(data.into())
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<'a> From<alloc::string::String> for ByteQueue<'a> {
    fn from(data: alloc::string::String) -> Self {
        Self::with_item(data.into())
    }
}

impl<'a> IntoIterator for ByteQueue<'a> {
    type Item = ByteData<'a>;
    type IntoIter = ChunkIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ChunkIter(self.queue)
    }
}

impl<'a: 'b, 'b> IntoIterator for &'b ByteQueue<'a> {
    type Item = &'b ByteData<'a>;
    type IntoIter = LinkedIter<'a, 'b>;

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
