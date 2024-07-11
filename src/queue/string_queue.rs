use super::ByteQueue;
use crate::StringData;

/// A queue of strings.
#[repr(transparent)]
#[derive(Clone)]
pub struct StringQueue<'a> {
    queue: ByteQueue<'a>,
}

impl<'a> StringQueue<'a> {
    /// Create a new empty `StringQueue`.
    #[inline]
    pub const fn new() -> Self {
        Self {
            queue: ByteQueue::new(),
        }
    }

    /// Create a new `StringQueue` with a single item.
    #[inline]
    pub const fn with_item(data: StringData<'a>) -> Self {
        Self {
            queue: ByteQueue::with_item(data.into_bytedata()),
        }
    }

    #[inline]
    pub(super) const unsafe fn from_bytequeue(q: ByteQueue<'a>) -> Self {
        StringQueue { queue: q }
    }

    /// Get the inner bytequeue.
    #[inline]
    pub const fn as_bytequeue(&self) -> &ByteQueue<'a> {
        &self.queue
    }

    #[inline]
    pub(super) fn as_bytequeue_mut(&mut self) -> &mut ByteQueue<'a> {
        &mut self.queue
    }

    /// Get the inner bytequeue.
    #[inline]
    pub const fn into_bytequeue(self) -> ByteQueue<'a> {
        unsafe { core::mem::transmute(self) }
    }

    /// Checks if the queue is full. When the feature `alloc` is enabled, this will always return `false`.
    #[inline]
    pub const fn is_full(&self) -> bool {
        self.queue.is_full()
    }

    /// Append string to the queue.
    #[inline]
    pub fn push_back(&mut self, data: impl Into<StringData<'a>>) {
        let data: StringData = data.into();
        self.queue.push_back(data.into_bytedata());
    }

    /// Prepend string into the queue.
    #[inline]
    pub fn push_front(&mut self, data: impl Into<StringData<'a>>) {
        let data = data.into();
        self.queue.push_front(data.into_bytedata());
    }

    /// Pop the first item from the queue.
    #[inline]
    pub fn pop_front(&mut self) -> Option<StringData<'a>> {
        match self.queue.pop_front() {
            Some(v) => Some(unsafe { StringData::from_bytedata_unchecked(v) }),
            None => None,
        }
    }

    /// Pop the last item from the queue.
    #[inline]
    pub fn pop_back(&mut self) -> Option<StringData<'a>> {
        match self.queue.pop_back() {
            Some(v) => Some(unsafe { StringData::from_bytedata_unchecked(v) }),
            None => None,
        }
    }

    /// Get the first chunk in the queue.
    #[inline]
    pub fn front(&self) -> Option<&crate::StringData<'a>> {
        self.queue.front().map(|v| unsafe {
            core::mem::transmute::<&crate::ByteData<'a>, &crate::StringData<'a>>(v)
        })
    }

    /// Get the last chunk in the queue.
    #[inline]
    pub fn back(&self) -> Option<&crate::StringData<'a>> {
        self.queue.back().map(|v| unsafe {
            core::mem::transmute::<&crate::ByteData<'a>, &crate::StringData<'a>>(v)
        })
    }

    /// Check if there are no bytes in the queue.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// The amount of utf-8 bytes in the queue.
    #[inline]
    pub const fn len(&self) -> usize {
        self.queue.len()
    }

    /// The amount of chunks in the queue.
    #[inline]
    pub const fn chunk_len(&self) -> usize {
        self.queue.chunk_len()
    }

    /// Check if the queue starts with the given bytes.
    #[inline]
    pub fn starts_with<S: AsRef<[u8]>>(&self, bytes: S) -> bool {
        self.queue.starts_with(bytes.as_ref())
    }

    /// Check if the queue ends with the given bytes.
    #[inline]
    pub fn ends_with<S: AsRef<[u8]>>(&self, bytes: S) -> bool {
        self.queue.ends_with(bytes.as_ref())
    }

    /// Iterates over each chunk of stringdata in the queue.
    #[inline]
    pub fn into_iter(self) -> super::StrChunkIter<'a> {
        super::StrChunkIter::new(self.queue.queue)
    }

    /// Slices the queue and returns a new queue that represents the given range.
    /// Panics if the range boundary is invalid UTF-8.
    pub fn slice(&self, range: impl core::ops::RangeBounds<usize>) -> Self {
        let slic = self.queue.slice(range);
        if slic.is_empty() {
            return Self::new();
        }
        let f = slic.front().unwrap();
        if f[0] & 0b1100_0000 == 0b1000_0000 {
            panic!("StringQueue: Invalid UTF-8 start in range");
        }
        let b = slic.back().unwrap();
        let end_byte = b[b.len() - 1];
        if end_byte & 0b1100_0000 == 0b1100_0000 {
            panic!("StringQueue: Invalid UTF-8 end in range");
        }
        if end_byte & 0b1100_0000 == 0b1000_0000 {
            // compute backwards to find the start of the char to see if the number of bytes is correct
            let mut i = b.len() - 2;
            while b[i] & 0b1100_0000 == 0b1000_0000 {
                i -= 1;
            }
            let char_len = b.len() - i;
            if char_len == 2 && b[i] & 0b1110_0000 != 0b1100_0000 {
                panic!("StringQueue: Invalid UTF-8 end in range");
            }
            if char_len == 3 && b[i] & 0b1111_0000 != 0b1110_0000 {
                panic!("StringQueue: Invalid UTF-8 end in range");
            }
            if char_len == 4 && b[i] & 0b1111_1000 != 0b1111_0000 {
                panic!("StringQueue: Invalid UTF-8 end in range");
            }
        }
        Self { queue: slic }
    }

    /// Iterates over each character in the queue.
    #[inline]
    pub fn chars(&self) -> super::CharIter<'a, '_> {
        super::char_iter::CharIter::new(self)
    }

    /// Iterates over each character in the queue.
    #[inline]
    pub fn into_chars(self) -> super::OwnedCharIter<'a> {
        super::char_iter::OwnedCharIter::new(self)
    }

    /// Iterates over each character in the queue.
    #[inline]
    pub fn chars_indecies(&self) -> super::CharIndecies<'a, '_> {
        super::char_iter::CharIndecies::new(self)
    }

    /// Iterates over each byte in the queue.
    #[inline]
    pub fn bytes(&self) -> super::ByteIter<'a, '_> {
        self.queue.bytes()
    }

    /// Iterates over each byte in the queue.
    #[inline]
    pub fn into_bytes(self) -> super::OwnedByteIter<'a> {
        self.into_bytequeue().into_bytes()
    }

    /// Iterates over each chunk of bytes in the queue.
    #[inline]
    pub fn byte_chunks(&self) -> super::LinkedIter<'a, '_> {
        self.queue.chunks()
    }

    /// Iterates over each chunk of string data in the queue.
    pub fn chunks(&self) -> impl Iterator<Item = &'_ StringData<'a>> + ExactSizeIterator + '_ {
        self.queue.chunks().map(|v| unsafe {
            core::mem::transmute::<&crate::ByteData<'a>, &crate::StringData<'a>>(v)
        })
    }

    /// Split the queue on a certain str sequence.
    pub const fn split_on<'b>(&'b self, needle: &'b str) -> super::SplitOnStr<'a, 'b> {
        super::SplitOnStr::new(self, needle, 0)
    }

    /// Split the queue on a certain str sequence.
    pub const fn splitn_on<'b>(&'b self, needle: &'b str, max: usize) -> super::SplitOnStr<'a, 'b> {
        super::SplitOnStr::new(self, needle, max)
    }

    /// Split the queue on a certain byte sequence.
    pub const fn split_on_bytes<'b>(&'b self, needle: &'b [u8]) -> super::SplitOn<'a, 'b> {
        super::SplitOn::new(self.as_bytequeue(), needle, 0)
    }

    /// Split the queue on a certain byte sequence.
    pub const fn splitn_on_bytes<'b>(
        &'b self,
        needle: &'b [u8],
        max: usize,
    ) -> super::SplitOn<'a, 'b> {
        super::SplitOn::new(self.as_bytequeue(), needle, max)
    }

    /// Split the queue on a certain byte position.
    /// `self` will contain the beginning `[0, at)`, and the returned queue will contain the end part `[at, len)`.
    /// If the position is in the middle of a multi-byte UTF-8 character, this will panic.
    pub fn split_off(&mut self, at: usize) -> Self {
        if at == self.len() {
            return Self::new();
        }
        if at == 0 {
            return core::mem::replace(self, Self::new());
        }
        // check if the split is in the middle of a char
        let b = self.queue.bytes().skip(at).next().unwrap();
        if b & 0b1100_0000 == 0b1000_0000 {
            panic!("StringQueue: Invalid UTF-8 split at index {}", at)
        }
        let out = self.queue.split_off(at);
        unsafe { Self::from_bytequeue(out) }
    }

    fn check_range(&self, range: impl core::ops::RangeBounds<usize>) -> (usize, usize) {
        let (start, end) = self.queue.check_range(range);
        if start == self.len() {
            return (start, end);
        }
        let mut len = end - start;
        let mut bytes = self.queue.bytes();
        if start != 0 {
            bytes = bytes.skip(start);
            let b = bytes.next().unwrap();
            if b & 0b1100_0000 == 0b1000_0000 {
                panic!("StringQueue: Invalid UTF-8 start in range");
            }
            if start == end || end == self.len() {
                return (start, end);
            }
            len -= 1;
        } else if end == self.len() {
            return (start, end);
        }
        let mut bytes = bytes.skip(len);
        let b = bytes.next().unwrap();
        if b & 0b1100_0000 == 0b1000_0000 {
            panic!("StringQueue: Invalid UTF-8 end in range");
        }
        (start, end)
    }

    /// Drain a range from the queue.
    /// Panics if the range boundary falls in the middle of a multi-byte UTF-8 character.
    pub fn drain(
        &mut self,
        range: impl core::ops::RangeBounds<usize>,
    ) -> super::DrainChars<'a, '_> {
        let (start, end) = self.check_range(range);
        unsafe { super::DrainChars::new(self, start, end) }
    }
}

impl<'a> From<StringData<'a>> for StringQueue<'a> {
    #[inline]
    fn from(data: StringData<'a>) -> Self {
        Self::with_item(data)
    }
}

impl<'a> From<&'a str> for StringQueue<'a> {
    #[inline]
    fn from(data: &'a str) -> Self {
        Self::with_item(data.into())
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<'a> From<alloc::string::String> for StringQueue<'a> {
    #[inline]
    fn from(data: alloc::string::String) -> Self {
        Self::with_item(data.into())
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<'a> From<alloc::borrow::Cow<'a, str>> for StringQueue<'a> {
    #[inline]
    fn from(data: alloc::borrow::Cow<'a, str>) -> Self {
        Self::with_item(match data {
            alloc::borrow::Cow::Borrowed(v) => StringData::from_borrowed(v),
            alloc::borrow::Cow::Owned(v) => StringData::from_owned(v),
        })
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<'a> From<StringQueue<'a>> for StringData<'a> {
    #[inline]
    fn from(data: StringQueue<'a>) -> Self {
        let out = From::from(data.queue);
        unsafe { StringData::from_bytedata_unchecked(out) }
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<'a> From<StringQueue<'a>> for alloc::string::String {
    #[inline]
    fn from(data: StringQueue<'a>) -> Self {
        let mut out = alloc::string::String::with_capacity(data.len());
        for c in data.chunks() {
            out.push_str(c.as_str());
        }
        out
    }
}

impl<'a> FromIterator<StringData<'a>> for StringQueue<'a> {
    fn from_iter<T: IntoIterator<Item = StringData<'a>>>(iter: T) -> Self {
        let mut out = Self::new();
        for i in iter {
            out.queue.push_back(i.into_bytedata());
        }
        out
    }
}

impl<'a> IntoIterator for StringQueue<'a> {
    type Item = StringData<'a>;
    type IntoIter = super::StrChunkIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        super::StrChunkIter::new(self.queue.queue)
    }
}

impl<'a> Extend<StringData<'a>> for StringQueue<'a> {
    fn extend<T: IntoIterator<Item = StringData<'a>>>(&mut self, iter: T) {
        for i in iter {
            self.queue.push_back(i.into_bytedata());
        }
    }
}

impl<'a> Extend<&'a str> for StringQueue<'a> {
    fn extend<T: IntoIterator<Item = &'a str>>(&mut self, iter: T) {
        for i in iter {
            self.queue.push_back(StringData::from_borrowed(i));
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a> Extend<alloc::string::String> for StringQueue<'a> {
    fn extend<T: IntoIterator<Item = alloc::string::String>>(&mut self, iter: T) {
        for i in iter {
            self.queue.push_back(StringData::from_owned(i));
        }
    }
}

impl core::fmt::Display for crate::StringQueue<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        for c in self.chunks() {
            core::fmt::Display::fmt(c, f)?;
        }
        Ok(())
    }
}

impl core::fmt::Debug for crate::StringQueue<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut l = f.debug_list();
        l.entries(self.chunks());
        l.finish()
    }
}

impl<'a> Default for crate::StringQueue<'a> {
    #[inline]
    fn default() -> Self {
        Self {
            queue: crate::ByteQueue::new(),
        }
    }
}

impl Eq for crate::StringQueue<'_> {}
impl<'a, 'b> PartialEq<crate::StringQueue<'b>> for crate::StringQueue<'a> {
    #[inline]
    fn eq(&self, other: &crate::StringQueue<'b>) -> bool {
        self.queue == other.queue
    }
}

impl<'a, 'b> PartialEq<crate::ByteQueue<'b>> for crate::StringQueue<'a> {
    #[inline]
    fn eq(&self, other: &crate::ByteQueue<'b>) -> bool {
        self.queue == *other
    }
}
impl<'a, 'b> PartialEq<crate::StringQueue<'b>> for crate::ByteQueue<'a> {
    #[inline]
    fn eq(&self, other: &crate::StringQueue<'b>) -> bool {
        *self == other.queue
    }
}

impl<'a, 'b> PartialEq<crate::StringData<'b>> for crate::StringQueue<'a> {
    #[inline]
    fn eq(&self, other: &crate::StringData<'b>) -> bool {
        self.queue == *other
    }
}
impl<'a, 'b> PartialEq<crate::StringQueue<'b>> for crate::StringData<'a> {
    #[inline]
    fn eq(&self, other: &crate::StringQueue<'b>) -> bool {
        *self == other.queue
    }
}

impl<'a, 'b> PartialEq<crate::ByteData<'b>> for crate::StringQueue<'a> {
    #[inline]
    fn eq(&self, other: &crate::ByteData<'b>) -> bool {
        self.queue == *other
    }
}
impl<'a, 'b> PartialEq<crate::StringQueue<'b>> for crate::ByteData<'a> {
    #[inline]
    fn eq(&self, other: &crate::StringQueue<'b>) -> bool {
        *self == other.queue
    }
}

impl<'a, 'b> PartialEq<&'b str> for crate::StringQueue<'a> {
    #[inline]
    fn eq(&self, other: &&'b str) -> bool {
        &self.queue == *other
    }
}
impl<'a> PartialEq<str> for crate::StringQueue<'a> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        &self.queue == other
    }
}
impl<'a> PartialEq<crate::StringQueue<'a>> for str {
    #[inline]
    fn eq(&self, other: &crate::StringQueue<'a>) -> bool {
        self == &other.queue
    }
}

impl<'a, 'b> PartialEq<[u8]> for crate::StringQueue<'a> {
    #[inline]
    fn eq(&self, other: &[u8]) -> bool {
        &self.queue == other
    }
}
impl<'a> PartialEq<crate::StringQueue<'a>> for [u8] {
    #[inline]
    fn eq(&self, other: &crate::StringQueue<'a>) -> bool {
        self == &other.queue
    }
}
