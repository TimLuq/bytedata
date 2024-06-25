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
}

impl<'a: 'b, 'b> ExactSizeIterator for LinkedIter<'a, 'b> {
    fn len(&self) -> usize {
        self.item_len()
    }
}
