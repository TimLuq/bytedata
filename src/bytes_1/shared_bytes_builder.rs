use ::bytes_1 as bytes;

use crate::SharedBytesBuilder;

#[cfg_attr(docsrs, doc(cfg(feature = "bytes_1")))]
unsafe impl bytes::BufMut for SharedBytesBuilder {
    fn remaining_mut(&self) -> usize {
        (0xFFFF_FFFF - self.off) as usize
    }

    unsafe fn advance_mut(&mut self, cnt: usize) {
        if cnt > self.remaining_mut() {
            panic!("SharedBytesBuilder::advance_mut: index out of bounds");
        }
        self.off += cnt as u32;
    }

    fn chunk_mut(&mut self) -> &mut bytes::buf::UninitSlice {
        if self.off >= self.len {
            self.reserve_extra();
        }
        let len = self.len - self.off;
        unsafe {
            bytes::buf::UninitSlice::from_raw_parts_mut(
                self.dat.offset(self.off as isize),
                len as usize,
            )
        }
    }

    fn put<T: bytes::buf::Buf>(&mut self, mut src: T) {
        let rem = src.remaining();
        if rem == 0 {
            return;
        }
        if rem > self.remaining_mut() {
            panic!("SharedBytesBuilder::put: index out of bounds");
        }
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

    fn put_slice(&mut self, src: &[u8]) {
        self.extend_from_slice(src);
    }

    fn put_bytes(&mut self, val: u8, cnt: usize) {
        if cnt > self.remaining_mut() {
            panic!("SharedBytesBuilder::put_bytes: index out of bounds");
        }
        self.reserve(cnt);
        unsafe {
            let ptr = self.dat.offset(self.off as isize);
            ptr.write_bytes(val, cnt);
        }
        self.off += cnt as u32;
    }
}
