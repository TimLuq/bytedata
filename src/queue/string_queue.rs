use crate::StringData;
use super::ByteQueue;

/// A queue of strings.
pub struct StringQueue<'a> {
    queue: ByteQueue<'a>,
}

impl<'a> StringQueue<'a> {
    /// Create a new empty `StringQueue`.
    #[inline]
    pub const fn new() -> Self {
        Self { queue: ByteQueue::new() }
    }

    /// Create a new `StringQueue` with a single item.
    #[inline]
    pub const fn with_item(data: StringData<'a>) -> Self {
        Self { queue: ByteQueue::with_item(data.into_bytedata()) }
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

    /// Check if there are no bytes in the queue.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// The amount of bytes in the queue.
    #[inline]
    pub const fn len(&self) -> usize {
        self.queue.len()
    }

    /// The amount of bytes in the queue.
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