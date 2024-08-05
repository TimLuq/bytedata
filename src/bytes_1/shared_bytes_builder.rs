use ::bytes_1 as bytes;

use crate::SharedBytesBuilder;

#[cfg_attr(docsrs, doc(cfg(feature = "bytes_1")))]
// SAFETY: this impl is safe.
unsafe impl bytes::BufMut for SharedBytesBuilder {
    #[inline]
    fn remaining_mut(&self) -> usize {
        (0xFFFF_FFFF - self.off) as usize
    }

    #[inline]
    unsafe fn advance_mut(&mut self, cnt: usize) {
        assert!(
            cnt <= self.remaining_mut(),
            "SharedBytesBuilder::advance_mut: index out of bounds"
        );
        #[allow(clippy::cast_possible_truncation)]
        let cnt = cnt as u32;
        self.off += cnt;
    }

    #[inline]
    fn chunk_mut(&mut self) -> &mut bytes::buf::UninitSlice {
        if self.off >= self.len {
            self.reserve_extra();
        }
        let len = self.len - self.off;
        // SAFETY: `off` is within bounds.
        let ptr = unsafe { self.dat.add(self.off as usize) };
        // SAFETY: `len` is within bounds.
        unsafe { bytes::buf::UninitSlice::from_raw_parts_mut(ptr, len as usize) }
    }

    #[inline]
    fn put<T: bytes::buf::Buf>(&mut self, mut src: T) {
        let rem = src.remaining();
        if rem == 0 {
            return;
        }
        assert!(
            rem <= self.remaining_mut(),
            "SharedBytesBuilder::put: index out of bounds"
        );
        self.reserve(rem);
        loop {
            let ch = src.chunk();
            let len = ch.len();
            self.extend_from_slice(ch);
            src.advance(len);
            if src.has_remaining() {
                continue;
            }
            return;
        }
    }

    #[inline]
    fn put_slice(&mut self, src: &[u8]) {
        self.extend_from_slice(src);
    }

    #[inline]
    fn put_bytes(&mut self, val: u8, cnt: usize) {
        assert!(
            cnt <= self.remaining_mut(),
            "SharedBytesBuilder::put_bytes: index out of bounds"
        );
        self.reserve(cnt);
        // SAFETY: `off` is within bounds.
        let ptr = unsafe { self.dat.add(self.off as usize) };
        // SAFETY: `cnt` is within bounds.
        unsafe {
            ptr.write_bytes(val, cnt);
        };
        #[allow(clippy::cast_possible_truncation)]
        let cnt = cnt as u32;
        self.off += cnt;
    }
}
