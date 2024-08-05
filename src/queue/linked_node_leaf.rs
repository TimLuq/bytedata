use crate::ByteData;

use super::linked_node_data::LinkedNodeData;

pub(super) struct LinkedNodeLeaf<'a> {
    pub(super) prev: *mut LinkedNodeLeaf<'a>,
    pub(super) data: LinkedNodeData<'a>,
    pub(super) next: *mut LinkedNodeLeaf<'a>,
}

// SAFETY: `LinkedNodeLeaf` is `Send` and `Sync` because it is a leaf node in a linked list
unsafe impl Send for LinkedNodeLeaf<'_> {}

// SAFETY: `LinkedNodeLeaf` is `Send` and `Sync` because it is a leaf node in a linked list
unsafe impl Sync for LinkedNodeLeaf<'_> {}

impl<'a> LinkedNodeLeaf<'a> {
    pub(super) const fn with_item(data: ByteData<'a>) -> Self {
        Self {
            prev: core::ptr::null_mut(),
            data: LinkedNodeData::with_item(data),
            next: core::ptr::null_mut(),
        }
    }
}
