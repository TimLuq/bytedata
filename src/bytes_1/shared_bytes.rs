#[cfg(not(feature = "bytes_1_safe"))]
use core::sync::atomic::{AtomicPtr, Ordering};

#[cfg(not(feature = "bytes_1_safe"))]
use alloc::vec::Vec;

use ::bytes_1 as bytes;

use crate::SharedBytes;

#[cfg(not(feature = "bytes_1_safe"))]
/// A vtable compatible with bytes::Vtable.
static SHARED_BYTES_BVT: super::SBytesVtable = super::SBytesVtable {
    clone: |data, ptr, len| {
        let p = data.load(Ordering::Relaxed);
        let meta = unsafe { &*(p as *const crate::SharedBytesMeta) };
        meta.refcnt.fetch_add(1, Ordering::Relaxed);
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
    is_unique: |data| unsafe {
        let p = data.load(Ordering::Relaxed);
        let meta = &*(p as *const crate::SharedBytesMeta);
        meta.refcnt.load(Ordering::Relaxed) == 1
    },
    drop: |data, _ptr, _len| unsafe {
        let p = data.load(Ordering::Relaxed) as *mut u8;
        let meta = &*(p as *const crate::SharedBytesMeta);
        if meta.refcnt.fetch_sub(1, Ordering::Relaxed) != 1 {
            return;
        }
        let layout =
            alloc::alloc::Layout::from_size_align(meta.len as usize, meta.align()).unwrap();
        alloc::alloc::dealloc(p, layout);
    },
};

#[cfg(feature = "bytes_1_safe")]
#[cfg_attr(docsrs, doc(cfg(feature = "bytes_1")))]
impl From<SharedBytes> for bytes::Bytes {
    #[inline]
    fn from(dat: SharedBytes) -> Self {
        bytes::Bytes::copy_from_slice(dat.as_slice())
    }
}

#[cfg(not(feature = "bytes_1_safe"))]
#[cfg_attr(docsrs, doc(cfg(feature = "bytes_1")))]
impl From<SharedBytes> for bytes::Bytes {
    fn from(dat: SharedBytes) -> Self {
        if dat.len == 0 {
            return bytes::Bytes::new();
        }
        let dat = core::mem::ManuallyDrop::new(dat);
        super::SBytes {
            ptr: unsafe { dat.dat().add(dat.off as usize) },
            len: dat.len as usize,
            data: AtomicPtr::new(dat.dat() as *mut ()),
            vtable: &SHARED_BYTES_BVT,
        }
        .into_bytes()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "bytes_1")))]
impl From<bytes::Bytes> for SharedBytes {
    #[inline]
    fn from(dat: bytes::Bytes) -> Self {
        Self::from_slice(dat.as_ref())
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "bytes_1")))]
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

    #[inline]
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
