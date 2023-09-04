use core::sync::atomic::{AtomicPtr, AtomicU32, Ordering};

use alloc::vec::Vec;

use ::bytes_1 as bytes;

use crate::SharedBytes;

/// A vtable compatible with bytes::Vtable.
static SHARED_BYTES_BVT: super::SBytesVtable = super::SBytesVtable {
    clone: |data, ptr, len| {
        let p = data.load(Ordering::Relaxed);
        unsafe { &mut *((p as *mut u8).offset(4) as *mut AtomicU32) }
            .fetch_add(1, Ordering::Relaxed);
        super::SBytes {
            ptr,
            len,
            data: AtomicPtr::new(p),
            vtable: &SHARED_BYTES_BVT,
        }
        .into_bytes()
    },
    to_vec: |_data, ptr, len| {
        if len == 0 {
            return Vec::new();
        }
        let mut vec = Vec::with_capacity(len);
        unsafe {
            core::ptr::copy_nonoverlapping(ptr, vec.as_mut_ptr(), len);
            vec.set_len(len);
        }
        vec
    },
    drop: |data, _ptr, _len| unsafe {
        let p = data.load(Ordering::Relaxed) as *mut u8;
        let refcnt = &mut *(p.offset(4) as *mut AtomicU32);
        if refcnt.fetch_sub(1, Ordering::Relaxed) == 1 {
            let layout =
                alloc::alloc::Layout::from_size_align(*(p as *mut u32) as usize, 4).unwrap();
            alloc::alloc::dealloc(p, layout);
        }
    },
};

impl From<SharedBytes> for bytes::Bytes {
    fn from(dat: SharedBytes) -> Self {
        super::SBytes {
            ptr: unsafe { dat.dat.add(dat.off as usize) },
            len: dat.len as usize,
            data: AtomicPtr::new(dat.dat as *mut ()),
            vtable: &SHARED_BYTES_BVT,
        }
        .into_bytes()
    }
}

impl From<bytes::Bytes> for SharedBytes {
    fn from(dat: bytes::Bytes) -> Self {
        Self::from_slice(dat.as_ref())
    }
}

impl bytes::Buf for SharedBytes {
    #[inline]
    fn remaining(&self) -> usize {
        self.len as usize
    }

    #[inline]
    fn chunk(&self) -> &[u8] {
        self.as_slice()
    }

    fn advance(&mut self, cnt: usize) {
        if cnt > self.len as usize {
            panic!("SharedBytes::advance: index out of bounds");
        }
        self.off += cnt as u32;
        self.len -= cnt as u32;
    }

    fn has_remaining(&self) -> bool {
        self.len > 0
    }

    fn copy_to_bytes(&mut self, len: usize) -> bytes::Bytes {
        let currlen = self.len as usize;
        if len > currlen {
            panic!("SharedBytes::copy_to_bytes: index out of bounds");
        }
        if len == 0 {
            return bytes::Bytes::new();
        }
        if len == currlen {
            return core::mem::replace(self, SharedBytes::empty()).into();
        }
        let ret = self.sliced(0, len).into();
        self.off += len as u32;
        self.len -= len as u32;
        ret
    }
}
