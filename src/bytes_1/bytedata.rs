use ::bytes_1 as bytes;

use crate::ByteData;

impl From<ByteData<'_>> for bytes::Bytes {
    fn from(dat: ByteData) -> Self {
        match dat {
            ByteData::Static(dat) => bytes::Bytes::from_static(dat),
            ByteData::Shared(dat) => dat.into(),
            ByteData::Borrowed(dat) => bytes::Bytes::copy_from_slice(dat),
        }
    }
}

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
