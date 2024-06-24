use crate::ByteData;

use super::linked_node_data::LinkedNodeData;


pub(super) struct LinkedNodeLeaf<'a> {
    pub(super) prev: *mut LinkedNodeLeaf<'a>,
    pub(super) data: LinkedNodeData<'a>,
    pub(super) next: *mut LinkedNodeLeaf<'a>,
}

unsafe impl Send for LinkedNodeLeaf<'_> {}

unsafe impl Sync for LinkedNodeLeaf<'_> {}

impl<'a> LinkedNodeLeaf<'a> {
    pub(super) fn with_item(data: ByteData<'a>) -> Self {
        Self {
            prev: core::ptr::null_mut(),
            data: LinkedNodeData::with_item(data),
            next: core::ptr::null_mut(),
        }
    }
}
