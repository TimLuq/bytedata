use ::bytes_1 as bytes;

use crate::ByteData;

#[cfg_attr(docsrs, doc(cfg(feature = "bytes_1")))]
impl From<ByteData<'_>> for bytes::Bytes {
    fn from(dat: ByteData) -> Self {
        match unsafe { dat.chunk.kind() } {
            crate::bytedata::Kind::Slice => {
                let a = unsafe { &dat.slice };
                if a.is_empty() {
                    core::mem::forget(dat);
                    return bytes::Bytes::new();
                }
                if let Some(a) = a.as_static() {
                    let r = bytes::Bytes::from_static(a);
                    core::mem::forget(dat);
                    return r;
                }
                let r = bytes::Bytes::copy_from_slice(a.as_slice());
                core::mem::forget(dat);
                r
            }
            crate::bytedata::Kind::Chunk => {
                let l = unsafe { dat.chunk.data.len };
                if l == 0 {
                    core::mem::forget(dat);
                    return bytes::Bytes::new();
                }
                let r = bytes::Bytes::copy_from_slice(unsafe { dat.chunk.data.as_slice() });
                core::mem::forget(dat);
                r
            }
            #[cfg(feature = "alloc")]
            crate::bytedata::Kind::Shared => {
                let s = unsafe { core::mem::transmute::<ByteData, crate::SharedBytes>(dat) };
                s.into()
            }
        }
    }
}

#[cfg(all(not(feature = "bytes_1_safe"), feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg_attr(docsrs, doc(cfg(feature = "bytes_1")))]
impl From<bytes::Bytes> for ByteData<'_> {
    fn from(dat: bytes::Bytes) -> Self {
        let b = unsafe { core::mem::transmute::<&bytes::Bytes, &super::SBytes>(&dat) };
        if !b.data.load(core::sync::atomic::Ordering::Relaxed).is_null() {
            Self::from_shared(dat.into())
        } else {
            // if the order of the fields in `bytes::Bytes` or `bytes::Vtable` changes this will leak due to `forget`, which is better than crashing I guess
            let ret = Self::from_static(unsafe { core::slice::from_raw_parts(b.ptr, b.len) });
            core::mem::forget(dat);
            ret
        }
    }
}

#[cfg(all(feature = "bytes_1_safe", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg_attr(docsrs, doc(cfg(feature = "bytes_1")))]
impl From<bytes::Bytes> for ByteData<'_> {
    fn from(dat: bytes::Bytes) -> Self {
        Self::from_shared(dat.into())
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "bytes_1")))]
impl bytes::Buf for ByteData<'_> {
    #[inline]
    fn remaining(&self) -> usize {
        self.len()
    }

    #[inline]
    fn chunk(&self) -> &[u8] {
        self.as_slice()
    }

    fn advance(&mut self, cnt: usize) {
        if cnt > self.len() {
            panic!("ByteData::advance: index out of bounds");
        }
        self.make_sliced(cnt..);
    }

    #[inline]
    fn has_remaining(&self) -> bool {
        !self.is_empty()
    }

    fn copy_to_bytes(&mut self, len: usize) -> bytes::Bytes {
        let currlen = self.len();
        if len > currlen {
            panic!("ByteData::copy_to_bytes: index out of bounds");
        }
        if len == 0 {
            return bytes::Bytes::new();
        }
        if len == currlen {
            return core::mem::replace(self, ByteData::empty()).into();
        }
        let ret = self.sliced(0..len).into();
        self.make_sliced(len..);
        ret
    }
}
