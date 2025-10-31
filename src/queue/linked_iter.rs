use core::mem::MaybeUninit;

/// An iterator over byte chunks.
#[allow(missing_debug_implementations)]
pub struct LinkedIter<'a, 'b> {
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
        let Some(mut node) = self.node else {
            return usize::from(self.chamber.is_some());
        };
        let data = &node.data;

        let len = data.len as usize - self.offset;
        let mut len = len;
        // SAFETY: the pointer is either null or points to a valid node
        while let Some(aa) = unsafe { node.next.as_ref() } {
            len += aa.data.len as usize;
            node = aa;
        }
        if self.chamber.is_some() {
            len += 1;
        }
        len
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

#[allow(single_use_lifetimes)]
impl<'a: 'b, 'b> LinkedIter<'a, 'b> {
    /// Skips the next `n` items.
    #[inline]
    #[must_use]
    pub fn skip(mut self, n: usize) -> Self {
        if n != 0 {
            self.skip_mut(n);
        }
        self
    }

    /// Skips the next `n` items.
    #[inline]
    pub fn skip_mut(&mut self, n: usize) -> &mut Self {
        #[allow(single_use_lifetimes)]
        fn skip_mut_inner<'a: 'b, 'b>(this: &mut LinkedIter<'a, 'b>, mut n: usize) {
            #[cfg(feature = "alloc")]
            if this.chamber.take().is_some() {
                n -= 1;
            }
            while n != 0 {
                #[cfg(feature = "alloc")]
                let Some(node) = this.node
                else {
                    return;
                };
                #[cfg(feature = "alloc")]
                let data = &node.data;
                #[cfg(not(feature = "alloc"))]
                let data = this.data;

                #[cfg(not(feature = "alloc"))]
                if this.offset == data.len as usize {
                    return;
                }

                #[cfg(feature = "alloc")]
                if this.offset == data.len as usize {
                    // SAFETY: the pointer is either null or points to a valid node
                    this.node = unsafe { node.next.as_ref() };
                    this.offset = 0;
                    continue;
                }

                let skip = core::cmp::min(n, data.len as usize - this.offset);
                this.offset += skip;
                n -= skip;
            }
        }

        if n == 0 {
            return self;
        }
        skip_mut_inner(self, n);
        self
    }
}

impl<'a: 'b, 'b> Iterator for LinkedIter<'a, 'b> {
    type Item = &'b crate::ByteData<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        fn inner<'a: 'b, 'b>(this: &mut LinkedIter<'a, 'b>) -> Option<&'b crate::ByteData<'a>> {
            #[cfg(feature = "alloc")]
            let node = this.node?;
            #[cfg(feature = "alloc")]
            let data = &node.data;
            #[cfg(not(feature = "alloc"))]
            let data = this.data;

            if this.offset < data.len as usize {
                let ret: &MaybeUninit<crate::ByteData<'a>> =
                    &data.data[(data.beg as usize + this.offset) % data.data.len()];
                this.offset += 1;
                // SAFETY: the beg and len indicate a valid slot
                return Some(unsafe { ret.assume_init_ref() });
            }

            #[cfg(not(feature = "alloc"))]
            return None;

            #[cfg(feature = "alloc")]
            {
                // SAFETY: the pointer is either null or points to a valid node
                this.node = unsafe { node.next.as_ref() };
                this.offset = 0;
                this.next()
            }
        }

        #[cfg(feature = "alloc")]
        if let Some(val) = self.chamber.take() {
            return Some(val);
        }

        inner(self)
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

#[allow(single_use_lifetimes)]
impl<'a: 'b, 'b> ExactSizeIterator for LinkedIter<'a, 'b> {
    #[inline]
    fn len(&self) -> usize {
        self.item_len()
    }
}

#[allow(single_use_lifetimes)]
impl<'a: 'b, 'b> core::iter::FusedIterator for LinkedIter<'a, 'b> {}

/// An iterator over string chunks.
#[repr(transparent)]
#[allow(missing_debug_implementations)]
pub struct LinkedStrIter<'a, 'b> {
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

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            // SAFETY: The inner iterator returns chunks of `ByteData` which are valid UTF-8.
            .map(|val| unsafe {
                &*(val as *const crate::ByteData<'a>).cast::<crate::StringData<'a>>()
            })
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

#[allow(single_use_lifetimes)]
impl<'a: 'b, 'b> ExactSizeIterator for LinkedStrIter<'a, 'b> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.item_len()
    }
}

#[allow(single_use_lifetimes)]
impl<'a: 'b, 'b> core::iter::FusedIterator for LinkedStrIter<'a, 'b> {}
