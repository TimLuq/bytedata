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
    #[must_use]
    pub const fn new() -> Self {
        Self {
            queue: ByteQueue::new(),
        }
    }

    /// Create a new `StringQueue` with a single item.
    #[inline]
    #[must_use]
    pub const fn with_item(data: StringData<'a>) -> Self {
        Self {
            queue: ByteQueue::with_item(data.into_bytedata()),
        }
    }

    #[inline]
    #[must_use]
    pub(super) const unsafe fn from_bytequeue(queue: ByteQueue<'a>) -> Self {
        StringQueue { queue }
    }

    /// Get the inner bytequeue.
    #[inline]
    #[must_use]
    pub const fn as_bytequeue(&self) -> &ByteQueue<'a> {
        &self.queue
    }

    #[inline]
    pub(super) fn as_bytequeue_mut(&mut self) -> &mut ByteQueue<'a> {
        &mut self.queue
    }

    /// Get the inner bytequeue.
    #[inline]
    #[must_use]
    pub const fn into_bytequeue(self) -> ByteQueue<'a> {
        // SAFETY: `StringQueue` is a transparent wrapper around `ByteQueue`.
        unsafe { core::mem::transmute(self) }
    }

    /// Checks if the queue is full. When the feature `alloc` is enabled, this will always return `false`.
    #[inline]
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.queue.is_full()
    }

    /// Append string to the queue.
    #[inline]
    pub fn push_back<S: Into<StringData<'a>>>(&mut self, data: S) {
        let data = data.into();
        self.queue.push_back(data.into_bytedata());
    }

    /// Prepend string into the queue.
    #[inline]
    pub fn push_front<S: Into<StringData<'a>>>(&mut self, data: S) {
        let data = data.into();
        self.queue.push_front(data.into_bytedata());
    }

    /// Pop the first item from the queue.
    #[inline]
    pub fn pop_front(&mut self) -> Option<StringData<'a>> {
        let val = self.queue.pop_front()?;
        // SAFETY: The queue only contains valid utf-8 strings.
        Some(unsafe { StringData::from_bytedata_unchecked(val) })
    }

    /// Pop the last item from the queue.
    #[inline]
    pub fn pop_back(&mut self) -> Option<StringData<'a>> {
        let val = self.queue.pop_back()?;
        // SAFETY: The queue only contains valid utf-8 strings.
        Some(unsafe { StringData::from_bytedata_unchecked(val) })
    }

    /// Get the first chunk in the queue.
    #[inline]
    #[must_use]
    pub fn front(&self) -> Option<&crate::StringData<'a>> {
        self.queue
            .front()
            // SAFETY: The queue only contains valid utf-8 strings.
            .map(|val| unsafe { &*(val as *const crate::ByteData<'a>).cast::<StringData<'a>>() })
    }

    /// Get the last chunk in the queue.
    #[inline]
    #[must_use]
    pub fn back(&self) -> Option<&crate::StringData<'a>> {
        self.queue
            .back()
            // SAFETY: The queue only contains valid utf-8 strings.
            .map(|val| unsafe { &*(val as *const crate::ByteData<'a>).cast::<StringData<'a>>() })
    }

    /// Check if there are no bytes in the queue.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// The amount of utf-8 bytes in the queue.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.queue.len()
    }

    /// The amount of chunks in the queue.
    #[inline]
    #[must_use]
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
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn into_iter(self) -> super::StrChunkIter<'a> {
        super::StrChunkIter::new(self.queue.queue)
    }

    /// Slices the queue and returns a new queue that represents the given range.
    ///
    /// # Panics
    ///
    /// Panics if the range boundary is invalid UTF-8.
    #[inline]
    #[must_use]
    pub fn slice<R: core::ops::RangeBounds<usize>>(&self, range: R) -> Self {
        fn inner(slic: ByteQueue<'_>) -> StringQueue<'_> {
            #[allow(clippy::expect_used)]
            let front = slic.front().expect("not empty");
            assert!(
                front[0] & 0b1100_0000 != 0b1000_0000,
                "StringQueue: Invalid UTF-8 start in range"
            );
            #[allow(clippy::expect_used)]
            let by = slic.back().expect("not empty");
            let end_byte = by[by.len() - 1];
            assert!(
                end_byte & 0b1100_0000 != 0b1100_0000,
                "StringQueue: Invalid UTF-8 end in range"
            );
            if end_byte & 0b1100_0000 == 0b1000_0000 {
                // compute backwards to find the start of the char to see if the number of bytes is correct
                let mut i = by.len() - 2;
                while by[i] & 0b1100_0000 == 0b1000_0000 {
                    i -= 1;
                }
                let char_len = by.len() - i;
                assert!(
                    !(char_len == 2 && by[i] & 0b1110_0000 != 0b1100_0000),
                    "StringQueue: Invalid UTF-8 end in range"
                );
                assert!(
                    !(char_len == 3 && by[i] & 0b1111_0000 != 0b1110_0000),
                    "StringQueue: Invalid UTF-8 end in range"
                );
                assert!(
                    !(char_len == 4 && by[i] & 0b1111_1000 != 0b1111_0000),
                    "StringQueue: Invalid UTF-8 end in range"
                );
            }
            StringQueue { queue: slic }
        }
        let slic = self.queue.slice(range);
        if slic.is_empty() {
            return Self::new();
        }
        inner(slic)
    }

    /// Iterates over each character in the queue.
    #[inline]
    #[must_use]
    pub fn chars(&self) -> super::CharIter<'a, '_> {
        super::char_iter::CharIter::new(self)
    }

    /// Iterates over each character in the queue.
    #[inline]
    #[must_use]
    pub const fn into_chars(self) -> super::OwnedCharIter<'a> {
        super::char_iter::OwnedCharIter::new(self)
    }

    /// Iterates over each character in the queue.
    #[inline]
    #[must_use]
    pub fn chars_indecies(&self) -> super::CharIndecies<'a, '_> {
        super::char_iter::CharIndecies::new(self)
    }

    /// Iterates over each byte in the queue.
    #[inline]
    #[must_use]
    pub fn bytes(&self) -> super::ByteIter<'a, '_> {
        self.queue.bytes()
    }

    /// Iterates over each byte in the queue.
    #[inline]
    #[must_use]
    pub const fn into_bytes(self) -> super::OwnedByteIter<'a> {
        self.into_bytequeue().into_bytes()
    }

    /// Iterates over each chunk of bytes in the queue.
    #[inline]
    #[must_use]
    pub fn byte_chunks(&self) -> super::LinkedIter<'a, '_> {
        self.queue.chunks()
    }

    /// Iterates over each chunk of string data in the queue.
    #[inline]
    #[must_use]
    pub fn chunks(&self) -> super::LinkedStrIter<'a, '_> {
        // SAFETY: The queue only contains valid utf-8 strings.
        unsafe { super::LinkedStrIter::new(self.queue.chunks()) }
    }

    /// Split the queue on a certain str sequence.
    #[inline]
    #[must_use]
    pub const fn split_on<'b>(&'b self, needle: &'b str) -> super::SplitOnStr<'a, 'b> {
        super::SplitOnStr::new(self, needle, 0)
    }

    /// Split the queue on a certain str sequence.
    #[inline]
    #[must_use]
    pub const fn splitn_on<'b>(&'b self, needle: &'b str, max: usize) -> super::SplitOnStr<'a, 'b> {
        super::SplitOnStr::new(self, needle, max)
    }

    /// Split the queue on a certain byte sequence.
    #[inline]
    #[must_use]
    pub const fn split_on_bytes<'b>(&'b self, needle: &'b [u8]) -> super::SplitOn<'a, 'b> {
        super::SplitOn::new(self.as_bytequeue(), needle, 0)
    }

    /// Split the queue on a certain byte sequence.
    #[inline]
    #[must_use]
    pub const fn splitn_on_bytes<'b>(
        &'b self,
        needle: &'b [u8],
        max: usize,
    ) -> super::SplitOn<'a, 'b> {
        super::SplitOn::new(self.as_bytequeue(), needle, max)
    }

    /// Split the queue on a certain byte position.
    /// `self` will contain the beginning `[0, at)`, and the returned queue will contain the end part `[at, len)`.
    ///
    /// # Panics
    ///
    /// If the position is in the middle of a multi-byte UTF-8 character, this will panic.
    #[inline]
    #[must_use]
    pub fn split_off(&mut self, at: usize) -> Self {
        /// check if the split is in the middle of a char
        fn inner<'b>(queue: &mut ByteQueue<'b>, at: usize) -> StringQueue<'b> {
            #[allow(clippy::panic)]
            let Some(by) = queue.bytes().skip_mut(at).next() else {
                panic!("StringQueue: index {at} is out of bounds");
            };
            assert!(
                by & 0b1100_0000 != 0b1000_0000,
                "StringQueue: Invalid UTF-8 split at index {at}"
            );
            let out = queue.split_off(at);
            // SAFETY: The split is checked to be valid UTF-8.
            unsafe { StringQueue::from_bytequeue(out) }
        }
        if at == self.len() {
            return Self::new();
        }
        if at == 0 {
            return core::mem::replace(self, Self::new());
        }
        inner(&mut self.queue, at)
    }

    fn check_range(&self, range: impl core::ops::RangeBounds<usize>) -> (usize, usize) {
        // checks that the range is in the correct address range
        let (start, end) = self.queue.check_range(range);
        if start == self.len() {
            return (start, end);
        }
        let mut len = end - start;
        let mut bytes = self.queue.bytes();
        if start != 0 {
            bytes = bytes.skip(start);
            #[allow(clippy::unwrap_used)]
            let by = bytes.next().unwrap();
            assert!(
                by & 0b1100_0000 != 0b1000_0000,
                "StringQueue: Invalid UTF-8 start in range"
            );
            if start == end || end == self.len() {
                return (start, end);
            }
            len -= 1;
            // fallthrough to check the end
        } else if end == self.len() {
            return (start, end);
        } else {
            // fallthrough to check the end
        }
        let mut bytes = bytes.skip(len);
        #[allow(clippy::unwrap_used)]
        let by = bytes.next().unwrap();
        assert!(
            by & 0b1100_0000 != 0b1000_0000,
            "StringQueue: Invalid UTF-8 end in range"
        );
        (start, end)
    }

    /// Drain a range from the queue.
    ///
    /// # Panics
    ///
    /// Panics if the range boundary falls in the middle of a multi-byte UTF-8 character.
    #[inline]
    pub fn drain<R: core::ops::RangeBounds<usize>>(
        &mut self,
        range: R,
    ) -> super::DrainChars<'a, '_> {
        let (start, end) = self.check_range(range);
        // SAFETY: The range is checked to be valid UTF-8.
        unsafe { super::DrainChars::new(self, start, end) }
    }

    /// Find the first byte position of a char in the queue.
    #[inline]
    #[must_use]
    pub fn find_char<F: FnMut(char) -> bool>(&mut self, mut fun: F) -> Option<usize> {
        self.chars_indecies()
            .find(|&(_, ch)| fun(ch))
            .map(|(position, _)| position)
    }

    /// Move data to the returned `StringQueue` until the char predicate returns `false`.
    #[inline]
    #[must_use]
    pub fn take_while<F: FnMut(char) -> bool>(&mut self, mut fun: F) -> Self {
        let Some(position) = self.find_char(|ch| !fun(ch)) else {
            return core::mem::replace(self, Self::new());
        };
        if position == 0 {
            return Self::new();
        }
        let mut ret = self.queue.split_off(position);
        core::mem::swap(&mut self.queue, &mut ret);
        Self { queue: ret }
    }

    /// Takes and removes the first line from the queue.
    /// If a newline (`'\n'`) is found, the returned queue will contain all data up to, and including, the newline.
    /// If the queue does not contain a newline character, the returned queue will contain all data currently in the queue.
    #[inline]
    #[must_use]
    pub fn take_line(&mut self) -> Self {
        let Some(position) = self.find_char(|ch| ch == '\n') else {
            return core::mem::replace(self, Self::new());
        };
        let mut ret = self.queue.split_off(position + 1);
        core::mem::swap(&mut self.queue, &mut ret);
        Self { queue: ret }
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
impl From<alloc::string::String> for StringQueue<'_> {
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
            alloc::borrow::Cow::Borrowed(val) => StringData::from_borrowed(val),
            alloc::borrow::Cow::Owned(val) => StringData::from_owned(val),
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
        // SAFETY: The queue only contains valid utf-8 strings.
        unsafe { StringData::from_bytedata_unchecked(out) }
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<'a> From<StringQueue<'a>> for alloc::string::String {
    #[inline]
    fn from(data: StringQueue<'a>) -> Self {
        let mut out = Self::with_capacity(data.len());
        for chunk in data.chunks() {
            out.push_str(chunk.as_str());
        }
        out
    }
}

impl<'a> FromIterator<StringData<'a>> for StringQueue<'a> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = StringData<'a>>>(iter: T) -> Self {
        let mut out = Self::new();
        for item in iter {
            out.queue.push_back(item.into_bytedata());
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
    #[inline]
    fn extend<T: IntoIterator<Item = StringData<'a>>>(&mut self, iter: T) {
        for item in iter {
            self.queue.push_back(item.into_bytedata());
        }
    }
}

impl<'a> Extend<&'a str> for StringQueue<'a> {
    #[inline]
    fn extend<T: IntoIterator<Item = &'a str>>(&mut self, iter: T) {
        for item in iter {
            self.queue.push_back(StringData::from_borrowed(item));
        }
    }
}

#[cfg(feature = "alloc")]
impl Extend<alloc::string::String> for StringQueue<'_> {
    #[inline]
    fn extend<T: IntoIterator<Item = alloc::string::String>>(&mut self, iter: T) {
        for item in iter {
            self.queue.push_back(StringData::from_owned(item));
        }
    }
}

impl core::fmt::Display for crate::StringQueue<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for chunk in self.chunks() {
            core::fmt::Display::fmt(chunk, f)?;
        }
        Ok(())
    }
}

impl core::fmt::Debug for crate::StringQueue<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.chunks()).finish()
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl core::fmt::Write for crate::StringQueue<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.push_back(StringData::from_borrowed(s).into_shared());
        Ok(())
    }
}

impl Default for crate::StringQueue<'_> {
    #[inline]
    fn default() -> Self {
        Self {
            queue: crate::ByteQueue::new(),
        }
    }
}

impl Eq for crate::StringQueue<'_> {}
impl<'b> PartialEq<crate::StringQueue<'b>> for crate::StringQueue<'_> {
    #[inline]
    fn eq(&self, other: &crate::StringQueue<'b>) -> bool {
        self.queue == other.queue
    }
}

impl<'b> PartialEq<crate::ByteQueue<'b>> for crate::StringQueue<'_> {
    #[inline]
    fn eq(&self, other: &crate::ByteQueue<'b>) -> bool {
        self.queue == *other
    }
}
impl<'b> PartialEq<crate::StringQueue<'b>> for crate::ByteQueue<'_> {
    #[inline]
    fn eq(&self, other: &crate::StringQueue<'b>) -> bool {
        *self == other.queue
    }
}

impl<'b> PartialEq<crate::StringData<'b>> for crate::StringQueue<'_> {
    #[inline]
    fn eq(&self, other: &crate::StringData<'b>) -> bool {
        self.queue == *other
    }
}
impl<'b> PartialEq<crate::StringQueue<'b>> for crate::StringData<'_> {
    #[inline]
    fn eq(&self, other: &crate::StringQueue<'b>) -> bool {
        *self == other.queue
    }
}

impl<'b> PartialEq<crate::ByteData<'b>> for crate::StringQueue<'_> {
    #[inline]
    fn eq(&self, other: &crate::ByteData<'b>) -> bool {
        self.queue == *other
    }
}
impl<'b> PartialEq<crate::StringQueue<'b>> for crate::ByteData<'_> {
    #[inline]
    fn eq(&self, other: &crate::StringQueue<'b>) -> bool {
        *self == other.queue
    }
}

impl<'b> PartialEq<&'b str> for crate::StringQueue<'_> {
    #[inline]
    fn eq(&self, other: &&'b str) -> bool {
        self.queue == **other
    }
}
impl PartialEq<str> for crate::StringQueue<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.queue == other
    }
}
impl<'a> PartialEq<crate::StringQueue<'a>> for str {
    #[inline]
    fn eq(&self, other: &crate::StringQueue<'a>) -> bool {
        self == &other.queue
    }
}

impl PartialEq<[u8]> for crate::StringQueue<'_> {
    #[inline]
    fn eq(&self, other: &[u8]) -> bool {
        self.queue == other
    }
}
impl<'a> PartialEq<crate::StringQueue<'a>> for [u8] {
    #[inline]
    fn eq(&self, other: &crate::StringQueue<'a>) -> bool {
        self == &other.queue
    }
}
impl<'b> PartialEq<&'b [u8]> for crate::StringQueue<'_> {
    #[inline]
    fn eq(&self, other: &&'b [u8]) -> bool {
        self.queue == **other
    }
}
