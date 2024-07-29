use core::mem::MaybeUninit;

/// An iterator over byte chunks.
pub struct LinkedIter<'a: 'b, 'b> {
    #[cfg(feature = "alloc")]
    chamber: Option<&'b crate::ByteData<'a>>,
    #[cfg(feature = "alloc")]
    node: Option<&'b super::linked_node_leaf::LinkedNodeLeaf<'a>>,
    #[cfg(not(feature = "alloc"))]
    data: &'b super::linked_node_data::LinkedNodeData<'a>,
    offset: usize,
}

#[cfg(feature = "alloc")]
impl<'a: 'b, 'b> LinkedIter<'a, 'b> {
    #[inline]
    pub(super) const fn new(
        chamber: Option<&'b crate::ByteData<'a>>,
        node: Option<&'b super::linked_node_leaf::LinkedNodeLeaf<'a>>,
    ) -> Self {
        Self {
            chamber,
            node,
            offset: 0,
        }
    }

    fn item_len(&self) -> usize {
        let mut node = match self.node {
            Some(v) => v,
            None => return self.chamber.is_some() as usize,
        };
        let data = &node.data;

        let len = data.len as usize - self.offset;
        let mut len = len;
        while let Some(a) = unsafe { node.next.as_ref() } {
            len += a.data.len as usize;
            node = a;
            continue;
        }
        if self.chamber.is_some() {
            len += 1;
        }
        len
    }

    /// Skips the next `n` items.
    #[inline]
    pub fn skip(mut self, n: usize) -> Self {
        if n != 0 {
            self.skip_mut(n);
        }
        self
    }

    /// Skips the next `n` items.
    pub fn skip_mut(&mut self, mut n: usize) -> &mut Self {
        if n == 0 {
            return self;
        }
        #[cfg(feature = "alloc")]
        if self.chamber.take().is_some() {
            n -= 1;
        }
        while n != 0 {
            #[cfg(feature = "alloc")]
            let Some(node) = self.node
            else {
                return self;
            };
            #[cfg(feature = "alloc")]
            let data = &node.data;
            #[cfg(not(feature = "alloc"))]
            let data = self.data;

            #[cfg(not(feature = "alloc"))]
            if self.offset == data.len as usize {
                return self;
            }

            #[cfg(feature = "alloc")]
            if self.offset == data.len as usize {
                self.node = unsafe { node.next.as_ref() };
                self.offset = 0;
                continue;
            }

            let skip = core::cmp::min(n, data.len as usize - self.offset);
            self.offset += skip;
            n -= skip;
        }
        self
    }
}

#[cfg(not(feature = "alloc"))]
impl<'a: 'b, 'b> LinkedIter<'a, 'b> {
    #[inline]
    pub(super) const fn new(data: &'b super::linked_node_data::LinkedNodeData<'a>) -> Self {
        Self { data, offset: 0 }
    }

    #[inline]
    const fn item_len(&self) -> usize {
        self.data.len as usize - self.offset
    }
}

impl<'a: 'b, 'b> Iterator for LinkedIter<'a, 'b> {
    type Item = &'b crate::ByteData<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        #[cfg(feature = "alloc")]
        if let Some(v) = self.chamber.take() {
            return Some(v);
        }
        #[cfg(feature = "alloc")]
        let node = self.node?;
        #[cfg(feature = "alloc")]
        let data = &node.data;
        #[cfg(not(feature = "alloc"))]
        let data = self.data;

        if self.offset < data.len as usize {
            let r: &MaybeUninit<crate::ByteData<'a>> =
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

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.item_len();
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize {
        self.item_len()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n != 0 {
            self.skip_mut(n);
        }
        self.next()
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        let len = self.item_len();
        if len == 0 {
            return None;
        }
        self.skip_mut(len - 1);
        self.next()
    }
}

impl<'a: 'b, 'b> ExactSizeIterator for LinkedIter<'a, 'b> {
    fn len(&self) -> usize {
        self.item_len()
    }
}

impl<'a: 'b, 'b> core::iter::FusedIterator for LinkedIter<'a, 'b> {}

/// An iterator over string chunks.
#[repr(transparent)]
pub struct LinkedStrIter<'a: 'b, 'b> {
    inner: LinkedIter<'a, 'b>,
}

impl<'a: 'b, 'b> LinkedStrIter<'a, 'b> {
    #[inline]
    pub(super) const unsafe fn new(inner: LinkedIter<'a, 'b>) -> Self {
        Self { inner }
    }
}

impl<'a: 'b, 'b> Iterator for LinkedStrIter<'a, 'b> {
    type Item = &'b crate::StringData<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|v| unsafe { &*(v as *const crate::ByteData<'a> as *const crate::StringData<'a>) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.inner.item_len();
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize {
        self.inner.item_len()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n != 0 {
            self.inner.skip_mut(n);
        }
        self.next()
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        let len = self.inner.item_len();
        if len == 0 {
            return None;
        }
        self.inner.skip_mut(len - 1);
        self.next()
    }
}

impl<'a: 'b, 'b> ExactSizeIterator for LinkedStrIter<'a, 'b> {
    fn len(&self) -> usize {
        self.inner.item_len()
    }
}

impl<'a: 'b, 'b> core::iter::FusedIterator for LinkedStrIter<'a, 'b> {}
