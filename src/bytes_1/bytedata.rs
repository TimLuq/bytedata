use ::bytes_1 as bytes;

use crate::ByteData;

#[cfg_attr(docsrs, doc(cfg(feature = "bytes_1")))]
impl<'a> From<ByteData<'a>> for bytes::Bytes {
    #[allow(clippy::missing_inline_in_public_items)]
    fn from(dat: ByteData<'a>) -> Self {
        match dat.kind() {
            crate::bytedata::Kind::Slice => {
                // SAFETY: Slice kind is already checked
                let aa = unsafe { &dat.slice };
                if aa.is_empty() {
                    core::mem::forget(dat);
                    return Self::new();
                }
                if let Some(aa) = aa.as_static() {
                    let ret = Self::from_static(aa);
                    core::mem::forget(dat);
                    return ret;
                }
                let ret = Self::copy_from_slice(aa.as_slice());
                core::mem::forget(dat);
                ret
            }
            crate::bytedata::Kind::Chunk => {
                // SAFETY: Chunk kind is already checked
                let len = unsafe { dat.chunk.data.len };
                if len == 0 {
                    core::mem::forget(dat);
                    return Self::new();
                }
                // SAFETY: Chunk kind is already checked
                let ret = Self::copy_from_slice(unsafe { dat.chunk.data.as_slice() });
                core::mem::forget(dat);
                ret
            }
            #[cfg(feature = "alloc")]
            crate::bytedata::Kind::Shared => {
                // SAFETY: Shared kind is already checked
                let ret = unsafe { core::mem::transmute::<ByteData<'a>, crate::SharedBytes>(dat) };
                ret.into()
            }
            #[cfg(feature = "alloc")]
            crate::bytedata::Kind::External => {
                // SAFETY: External kind is already checked
                let ext =
                    unsafe { core::mem::transmute::<ByteData<'a>, crate::external::ExtBytes>(dat) };
                let ret = ext.with_inner(|tval: &Self, extb: &[u8]| {
                    let start = {
                        let sp: *const u8 = extb.as_ptr();
                        let tp: *const u8 = tval.as_ptr();
                        sp as usize - tp as usize
                    };
                    tval.slice(start..(start + extb.len()))
                });
                ret.unwrap_or_else(|| Self::copy_from_slice(ext.as_slice()))
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
    #[inline]
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

    #[inline]
    fn advance(&mut self, cnt: usize) {
        assert!(cnt <= self.len(), "ByteData::advance: index out of bounds");
        self.make_sliced(cnt..);
    }

    #[inline]
    fn has_remaining(&self) -> bool {
        !self.is_empty()
    }

    #[inline]
    fn copy_to_bytes(&mut self, len: usize) -> bytes::Bytes {
        let currlen = self.len();
        assert!(
            len <= currlen,
            "ByteData::copy_to_bytes: index out of bounds"
        );
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
