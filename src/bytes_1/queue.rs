use ::bytes_1 as bytes;

use crate::ByteQueue;

#[cfg_attr(docsrs, doc(cfg(feature = "bytes_1")))]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
impl From<ByteQueue<'_>> for bytes::Bytes {
    fn from(mut dat: ByteQueue) -> Self {
        use bytes::BufMut;
        if dat.is_empty() {
            return bytes::Bytes::new();
        }
        if dat.chunk_len() == 1 {
            return dat.pop_back().unwrap().into();
        }
        let mut ret = bytes::BytesMut::with_capacity(dat.len());
        for chunk in dat.chunks() {
            ret.put(chunk.as_slice());
        }
        ret.freeze()
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg_attr(docsrs, doc(cfg(feature = "bytes_1")))]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
impl From<bytes::Bytes> for ByteQueue<'_> {
    #[inline]
    fn from(dat: bytes::Bytes) -> Self {
        Self::from(crate::ByteData::from(dat))
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "bytes_1")))]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
impl bytes::Buf for ByteQueue<'_> {
    #[inline]
    fn remaining(&self) -> usize {
        self.len()
    }

    #[inline]
    fn chunk(&self) -> &[u8] {
        self.front().map(|s| s.as_slice()).unwrap_or_default()
    }

    fn advance(&mut self, mut cnt: usize) {
        if cnt > self.len() {
            panic!("ByteData::advance: index out of bounds");
        }
        while cnt > 0 {
            let mut f = self.pop_front().unwrap();
            let len = f.len();
            if len > cnt {
                f.make_sliced(cnt..);
                self.push_front(f);
                return;
            }
            cnt -= len;
        }
    }

    #[inline]
    fn has_remaining(&self) -> bool {
        !self.is_empty()
    }

    fn copy_to_bytes(&mut self, mut len: usize) -> bytes::Bytes {
        use bytes::BufMut;
        let currlen = self.len();
        if len > currlen {
            panic!("ByteData::copy_to_bytes: index out of bounds");
        }
        if len == 0 {
            return bytes::Bytes::new();
        }
        let mut f = self.pop_front().unwrap();
        if f.len() == len {
            return f.into();
        }
        if f.len() > len {
            let r = f.copy_to_bytes(len);
            self.push_front(f);
            return r;
        }

        let mut ret = bytes::BytesMut::with_capacity(len);
        ret.put(f.as_slice());
        len -= f.len();
        while len > 0 {
            let mut f = self.pop_front().unwrap();
            let l = f.len();
            if l > len {
                ret.put(&f.as_slice()[..len]);
                f.make_sliced(len..);
                self.push_front(f);
                break;
            }
            ret.put(f.as_slice());
            len -= l;
        }
        ret.freeze()
    }
}
