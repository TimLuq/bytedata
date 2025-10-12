use crate::queue::{ChunkIter, LinkedIter};
use crate::ByteData;

#[cfg(feature = "alloc")]
pub(super) struct LinkedRoot<'a> {
    /// The first chunk in the queue, available for optimized use.
    pub(super) chamber: ByteData<'a>,
    pub(super) first: *mut super::linked_node_leaf::LinkedNodeLeaf<'a>,
    pub(super) last: *mut super::linked_node_leaf::LinkedNodeLeaf<'a>,
    /// The number of chunks in the queue.
    pub(super) count: usize,
}

#[cfg(feature = "alloc")]
// SAFETY: `LinkedRoot` is `Send` and `Sync` because the pointers points to heap allocated memory
unsafe impl Send for LinkedRoot<'_> {}

#[cfg(feature = "alloc")]
// SAFETY: `LinkedRoot` is `Send` and `Sync` because the pointers points to heap allocated memory
unsafe impl Sync for LinkedRoot<'_> {}

#[cfg(not(feature = "alloc"))]
#[allow(private_interfaces)]
pub(super) struct LinkedRoot<'a> {
    pub(super) data: super::linked_node_data::LinkedNodeData<'a>,
}

#[cfg(not(feature = "alloc"))]
impl<'a> LinkedRoot<'a> {
    #[inline]
    pub(super) const fn new() -> Self {
        Self {
            data: super::linked_node_data::LinkedNodeData::new(),
        }
    }

    #[inline]
    pub(super) const fn with_item(item: ByteData<'a>) -> Self {
        Self {
            data: super::linked_node_data::LinkedNodeData::with_item(item),
        }
    }

    #[inline]
    pub(super) const fn len(&self) -> usize {
        self.data.len as usize
    }

    #[inline]
    fn first_mut(&mut self) -> Option<&mut super::linked_node_data::LinkedNodeData<'a>> {
        Some(&mut self.data)
    }

    #[inline]
    fn last_mut(&mut self) -> Option<&mut super::linked_node_data::LinkedNodeData<'a>> {
        Some(&mut self.data)
    }

    #[inline]
    pub(super) fn pop_back(&mut self) -> Option<ByteData<'a>> {
        self.data.pop_back()
    }

    #[inline]
    pub(super) fn pop_front(&mut self) -> Option<ByteData<'a>> {
        self.data.pop_front()
    }

    #[inline]
    pub(super) fn front(&self) -> Option<&ByteData<'a>> {
        self.data.front()
    }

    #[inline]
    pub(super) fn back(&self) -> Option<&ByteData<'a>> {
        self.data.back()
    }

    #[inline]
    pub(super) fn iter(&'_ self) -> super::LinkedIter<'a, '_> {
        LinkedIter::new(&self.data)
    }
}

#[cfg(feature = "alloc")]
impl<'a> LinkedRoot<'a> {
    #[inline]
    pub(super) const fn new() -> Self {
        Self {
            chamber: ByteData::empty(),
            first: core::ptr::null_mut(),
            last: core::ptr::null_mut(),
            count: 0,
        }
    }

    #[inline]
    pub(super) const fn with_item(item: ByteData<'a>) -> Self {
        let count = if item.is_empty() { 0 } else { 1 };
        Self {
            chamber: item,
            first: core::ptr::null_mut(),
            last: core::ptr::null_mut(),
            count,
        }
    }

    #[inline]
    pub(super) const fn len(&self) -> usize {
        self.count
    }

    #[inline]
    fn first(&self) -> Option<&super::linked_node_data::LinkedNodeData<'a>> {
        // SAFETY: if the pointer is non-null it points to a valid `LinkedNodeLeaf`.
        unsafe { self.first.as_ref().map(|x| &x.data) }
    }

    #[inline]
    fn first_mut(&mut self) -> Option<&mut super::linked_node_data::LinkedNodeData<'a>> {
        // SAFETY: if the pointer is non-null it points to a valid `LinkedNodeLeaf`.
        unsafe { self.first.as_mut().map(|x| &mut x.data) }
    }

    #[inline]
    fn last(&self) -> Option<&super::linked_node_data::LinkedNodeData<'a>> {
        // SAFETY: if the pointer is non-null it points to a valid `LinkedNodeLeaf`.
        unsafe { self.last.as_ref().map(|x| &x.data) }
    }

    #[inline]
    fn last_mut(&mut self) -> Option<&mut super::linked_node_data::LinkedNodeData<'a>> {
        // SAFETY: if the pointer is non-null it points to a valid `LinkedNodeLeaf`.
        unsafe { self.last.as_mut().map(|x| &mut x.data) }
    }

    fn push_back_alloc(&mut self, data: ByteData<'a>) {
        let node = alloc::boxed::Box::new(super::linked_node_leaf::LinkedNodeLeaf::with_item(data));
        let node = alloc::boxed::Box::into_raw(node);
        self.count += 1;
        // SAFETY: if the pointer is non-null it points to a valid `LinkedNodeLeaf`.
        if let Some(last) = unsafe { self.last.as_mut() } {
            last.next = node;
            // SAFETY: the pointer is a valid and non-null pointer to a boxed `LinkedNodeLeaf`.
            unsafe { &mut *node }.prev = last;
            self.last = node;
            return;
        }
        self.first = node;
        self.last = node;
    }

    fn push_front_alloc(&mut self, data: ByteData<'a>) {
        let node = alloc::boxed::Box::new(super::linked_node_leaf::LinkedNodeLeaf::with_item(data));
        let node = alloc::boxed::Box::into_raw(node);
        self.count += 1;
        // SAFETY: if the pointer is non-null it points to a valid `LinkedNodeLeaf`.
        if let Some(first) = unsafe { self.first.as_mut() } {
            first.prev = node;
            // SAFETY: the pointer is a valid and non-null pointer to a boxed `LinkedNodeLeaf`.
            unsafe { &mut *node }.next = first;
            self.first = node;
            return;
        }
        self.first = node;
        self.last = node;
    }

    pub(super) fn pop_back(&mut self) -> Option<ByteData<'a>> {
        if self.count == 1 && !self.chamber.is_empty() {
            self.count = 0;
            return Some(core::mem::replace(&mut self.chamber, ByteData::empty()));
        }
        // SAFETY: if the pointer is non-null it points to a valid `LinkedNodeLeaf`.
        let last = unsafe { self.last.as_mut()? };
        let ret = last.data.pop_back()?;
        self.count -= 1;

        // dealloc the node if it is empty and there exists a previous node
        if last.data.len == 0 {
            // SAFETY: if the pointer is non-null it points to a valid `LinkedNodeLeaf`.
            if let Some(prev) = unsafe { last.prev.as_mut() } {
                prev.next = core::ptr::null_mut();
                self.last = prev;
                // SAFETY: the pointer is a valid and non-null pointer to a boxed `LinkedNodeLeaf`.
                core::mem::drop(unsafe { alloc::boxed::Box::from_raw(last) });
            }
        }
        Some(ret)
    }

    pub(super) fn pop_front(&mut self) -> Option<ByteData<'a>> {
        if self.count == 0 {
            return None;
        }
        if !self.chamber.is_empty() {
            self.count -= 1;
            return Some(core::mem::replace(&mut self.chamber, ByteData::empty()));
        }
        // SAFETY: if the pointer is non-null it points to a valid `LinkedNodeLeaf`.
        let first = unsafe { self.first.as_mut()? };
        let ret = first.data.pop_front()?;
        self.count -= 1;

        // dealloc the node if it is empty and there exists a next node
        if first.data.len == 0 {
            // SAFETY: if the pointer is non-null it points to a valid `LinkedNodeLeaf`.
            if let Some(next) = unsafe { first.next.as_mut() } {
                next.prev = core::ptr::null_mut();
                self.first = next;
                // SAFETY: the pointer is a valid and non-null pointer to a boxed `LinkedNodeLeaf`.
                core::mem::drop(unsafe { alloc::boxed::Box::from_raw(first) });
            }
        }

        Some(ret)
    }

    #[inline]
    pub(super) fn front(&self) -> Option<&ByteData<'a>> {
        if self.count == 0 {
            return None;
        }
        if !self.chamber.is_empty() {
            return Some(&self.chamber);
        }
        self.first().and_then(|x| x.front())
    }

    #[inline]
    pub(super) fn back(&self) -> Option<&ByteData<'a>> {
        if let Some(aa) = self.last().and_then(|x| x.back()) {
            return Some(aa);
        }
        if !self.chamber.is_empty() {
            return Some(&self.chamber);
        }
        None
    }

    #[inline]
    pub(super) fn iter(&self) -> LinkedIter<'a, '_> {
        let chamber = if self.chamber.is_empty() {
            None
        } else {
            Some(&self.chamber)
        };
        // SAFETY: if the pointer is non-null it points to a valid `LinkedNodeLeaf`.
        LinkedIter::new(chamber, unsafe { self.first.as_ref() })
    }

    pub(super) fn append(&mut self, mut other: Self) {
        if other.count == 0 {
            return;
        }
        if self.count == 0 {
            *self = other;
            return;
        }
        if self.count == 1 && !self.chamber.is_empty() {
            other.push_front(core::mem::replace(
                &mut self.chamber,
                ByteData::from_chunk(&[0]),
            ));
            *self = other;
            return;
        }

        // attempt to move the chambered item without allocating
        if !other.chamber.is_empty() {
            let chamber = core::mem::replace(&mut other.chamber, ByteData::empty());
            other.count -= 1;
            // SAFETY: if the pointer is non-null it points to a valid `LinkedNodeLeaf`.
            if let Some(fst) = unsafe { other.first.as_mut() } {
                if let Err(val) = fst.data.push_front(chamber) {
                    self.push_back(val);
                }
            } else {
                self.push_back(chamber);
            }
            if other.count == 0 {
                return;
            }
        }

        // SAFETY: if the pointer is non-null it points to a valid `LinkedNodeLeaf`.
        if let Some(last) = unsafe { self.last.as_mut() } {
            // SAFETY: if the pointer is non-null it points to a valid `LinkedNodeLeaf`.
            if let Some(first) = unsafe { other.first.as_mut() } {
                last.next = first;
                first.prev = last;
                self.last = other.last;
            } else {
                unreachable!("invalid state at append");
            }
        } else {
            self.first = other.first;
            self.last = other.last;
        }
        self.count += other.count;
        other.first = core::ptr::null_mut();
        other.last = core::ptr::null_mut();
        other.count = 0;
    }

    /// Ensures that all chunks in the queue are shared so they can be used for any lifetime.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub(super) fn make_shared(&mut self) {
        if !self.chamber.is_empty() {
            self.chamber.make_shared();
        }
        let mut node = self.first;
        // SAFETY: if the pointer is non-null, it is a valid pointer to a `LinkedNodeLeaf`.
        while let Some(n) = unsafe { node.as_mut() } {
            n.data.make_shared();
            node = n.next;
        }
    }
}

impl<'a> LinkedRoot<'a> {
    pub(super) fn push_back(&mut self, mut data: ByteData<'a>) {
        if data.is_empty() {
            return;
        }
        #[cfg(feature = "alloc")]
        if self.count == 0 {
            self.chamber = data;
            self.count = 1;
            return;
        }
        #[allow(unused_assignments)]
        if let Some(last) = self.last_mut() {
            data = match last.push_back(data) {
                Ok(()) => {
                    #[cfg(feature = "alloc")]
                    {
                        self.count += 1;
                    };
                    return;
                }
                Err(val) => val,
            };
        }
        #[cfg(not(feature = "alloc"))]
        panic!(
            "ByteQueue::push_back: out of space, use `alloc` feature to enable dynamic allocation"
        );
        #[cfg(feature = "alloc")]
        self.push_back_alloc(data);
    }

    pub(super) fn push_front(&mut self, mut data: ByteData<'a>) {
        if data.is_empty() {
            return;
        }
        #[cfg(feature = "alloc")]
        {
            core::mem::swap(&mut self.chamber, &mut data);
            if data.is_empty() {
                self.count += 1;
                return;
            }
        }
        #[allow(unused_assignments)]
        if let Some(first) = self.first_mut() {
            data = match first.push_front(data) {
                Ok(()) => {
                    #[cfg(feature = "alloc")]
                    {
                        self.count += 1;
                    };
                    return;
                }
                Err(val) => val,
            };
        }
        #[cfg(not(feature = "alloc"))]
        panic!(
            "ByteQueue::push_front: out of space, use `alloc` feature to enable dynamic allocation"
        );
        #[cfg(feature = "alloc")]
        self.push_front_alloc(data);
    }

    #[inline]
    fn from_iter<I: Iterator<Item = ByteData<'a>>>(iter: I) -> Self {
        let mut root = Self::new();
        root.extend(iter);
        root
    }

    fn extend<I: Iterator<Item = ByteData<'a>>>(&mut self, iter: I) {
        for i in iter {
            self.push_back(i);
        }
    }

    #[inline]
    pub(super) const fn into_iter(self) -> ChunkIter<'a> {
        ChunkIter::new(self)
    }
}

impl Clone for LinkedRoot<'_> {
    #[inline]
    fn clone(&self) -> Self {
        Self::from_iter(self.iter().cloned())
    }
}

impl Default for LinkedRoot<'_> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for LinkedRoot<'_> {
    #[cfg(feature = "alloc")]
    fn drop(&mut self) {
        let mut node = self.first;
        // SAFETY: if the pointer is non-null, it is a valid pointer to a `LinkedNodeLeaf`.
        while let Some(n) = unsafe { node.as_mut() } {
            // SAFETY: the pointer is a valid and non-null pointer to a boxed `LinkedNodeLeaf`.
            let n = unsafe { alloc::boxed::Box::from_raw(n) };
            node = n.next;
            core::mem::drop(n);
        }
    }

    #[cfg(not(feature = "alloc"))]
    fn drop(&mut self) {
        // SAFETY: it is safe to drop the inner data
        unsafe { core::ptr::drop_in_place(&mut self.data) };
        self.data.len = 0;
    }
}
