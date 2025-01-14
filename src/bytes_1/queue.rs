use ::bytes_1 as bytes;

use crate::ByteQueue;

#[cfg_attr(docsrs, doc(cfg(feature = "bytes_1")))]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
impl<'a> From<ByteQueue<'a>> for bytes::Bytes {
    #[allow(clippy::missing_inline_in_public_items)]
    fn from(mut dat: ByteQueue<'a>) -> Self {
        use bytes::BufMut;
        if dat.is_empty() {
            return Self::new();
        }
        if dat.chunk_len() == 1 {
            #[allow(clippy::expect_used)]
            let chunk = dat.pop_front().expect("ByteQueue::from: chunk_len is 1");
            return chunk.into();
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
        self.front()
            .map(crate::ByteData::as_slice)
            .unwrap_or_default()
    }

    #[inline]
    fn advance(&mut self, mut cnt: usize) {
        assert!(cnt <= self.len(), "ByteData::advance: index out of bounds");
        while cnt > 0 {
            #[allow(clippy::unwrap_used)]
            let mut front = self.pop_front().unwrap();
            let len = front.len();
            if len > cnt {
                front.make_sliced(cnt..);
                self.push_front(front);
                return;
            }
            cnt -= len;
        }
    }

    #[inline]
    fn has_remaining(&self) -> bool {
        !self.is_empty()
    }

    #[allow(clippy::missing_inline_in_public_items)]
    fn copy_to_bytes(&mut self, mut len: usize) -> bytes::Bytes {
        use bytes::BufMut;
        let currlen = self.len();
        assert!(
            len <= currlen,
            "ByteData::copy_to_bytes: index out of bounds"
        );
        if len == 0 {
            return bytes::Bytes::new();
        }
        #[allow(clippy::unwrap_used)]
        let mut first = self.pop_front().unwrap();
        if first.len() == len {
            return first.into();
        }
        if first.len() > len {
            let ret = first.copy_to_bytes(len);
            self.push_front(first);
            return ret;
        }

        let mut ret = bytes::BytesMut::with_capacity(len);
        ret.put(first.as_slice());
        len -= first.len();
        while len > 0 {
            #[allow(clippy::unwrap_used)]
            let mut fchunk = self.pop_front().unwrap();
            let fl = fchunk.len();
            if fl > len {
                ret.put(&fchunk.as_slice()[..len]);
                fchunk.make_sliced(len..);
                self.push_front(fchunk);
                break;
            }
            ret.put(fchunk.as_slice());
            len -= fl;
        }
        ret.freeze()
    }

    #[cfg(feature = "std")]
    #[inline]
    fn chunks_vectored<'a>(&'a self, dst: &mut [std::io::IoSlice<'a>]) -> usize {
        let mut i = 0;
        for chunk in self.chunks().take(dst.len()) {
            dst[i] = std::io::IoSlice::new(chunk.as_slice());
            i += 1;
        }
        i
    }
}
