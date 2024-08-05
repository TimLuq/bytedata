use std::io::{BufRead, IoSliceMut, Read, Write};

use crate::{ByteData, SharedBytes, SharedBytesBuilder};

fn read_until_inner(dd: &[u8], byte: u8, buf: &mut Vec<u8>) -> usize {
    let len = dd.len();
    let mut i = 0;
    while i < len {
        if dd[i] == byte {
            i += 1;
            buf.extend_from_slice(&dd[..i]);
            return i;
        }
        i += 1;
    }
    buf.extend_from_slice(dd);
    len
}

fn read_vectored_inner(mut slic: *const u8, len: usize, bufs: &mut [IoSliceMut<'_>]) -> usize {
    let mut left = len;

    for buf in bufs {
        if left == 0 {
            break;
        }
        if buf.is_empty() {
            continue;
        }

        let blen = buf.len().min(len);
        // SAFETY: we know that the buffer is large enough as we checked it before
        unsafe { core::ptr::copy_nonoverlapping(slic, buf.as_mut_ptr(), blen) };
        // SAFETY: the pointer gets moved by `blen` which is guaranteed to be in bounds
        slic = unsafe { slic.add(blen) };
        left -= blen;
    }

    len - left
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl Read for ByteData<'_> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = buf.len().min(self.len());
        buf.copy_from_slice(&self.as_slice()[..len]);
        self.make_sliced(len..);
        Ok(len)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> std::io::Result<usize> {
        let slic = self.as_slice();
        let len = slic.len();
        if len == 0 {
            return Ok(0);
        }
        let offs = read_vectored_inner(slic.as_ptr(), len, bufs);
        if offs == len {
            *self = Self::empty();
        } else if offs != 0 {
            self.make_sliced(offs..);
        } else {
            // buffer list must have been empty, so we don't need to do anything
        }

        Ok(offs)
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let len = self.len();
        buf.extend_from_slice(self.as_slice());
        *self = ByteData::empty();
        Ok(len)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl BufRead for ByteData<'_> {
    #[inline]
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        Ok(self.as_slice())
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        self.make_sliced(amt..);
    }

    #[inline]
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let dd = self.as_slice();
        let len = dd.len();
        if len == 0 {
            return Ok(0);
        }
        let ret = read_until_inner(dd, byte, buf);
        if ret == len {
            *self = ByteData::empty();
        } else {
            self.make_sliced(ret..);
        }
        Ok(ret)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl Read for SharedBytes {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = buf.len().min(self.len());
        buf.copy_from_slice(&self.as_slice()[..len]);
        self.make_sliced(len, self.len() - len);
        Ok(len)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> std::io::Result<usize> {
        let slic = self.as_slice();
        let len = slic.len();
        if len == 0 {
            return Ok(0);
        }
        let offs = read_vectored_inner(slic.as_ptr(), len, bufs);
        if offs == len {
            *self = Self::empty();
        } else if offs != 0 {
            self.make_sliced(offs, len - offs);
        } else {
            // buffer list must have been empty, so we don't need to do anything
        }

        Ok(offs)
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let len = self.len();
        buf.extend_from_slice(self.as_slice());
        *self = Self::empty();
        Ok(len)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl BufRead for SharedBytes {
    #[inline]
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        Ok(self.as_slice())
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        self.make_sliced(amt, self.len() - amt);
    }

    #[inline]
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let dd = self.as_slice();
        let len = dd.len();
        if len == 0 {
            return Ok(0);
        }
        let ret = read_until_inner(dd, byte, buf);
        if ret == len {
            *self = Self::empty();
        } else {
            self.make_sliced(ret, len - ret);
        }
        Ok(ret)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl Write for SharedBytesBuilder {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let len = buf.len();
        self.extend_from_slice(buf);
        Ok(len)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        fn inner(targ: &mut [core::mem::MaybeUninit<u8>], bufs: &[std::io::IoSlice<'_>]) -> usize {
            let max = targ.len();
            let targ: *mut core::mem::MaybeUninit<u8> = targ.as_mut_ptr();
            let mut targ = targ.cast::<u8>();
            let mut len = 0;
            for buf in bufs {
                let ml = buf.len().min(max - len);
                if ml != 0 {
                    // SAFETY: we know that the buffer is large enough as it was allocated before the call to `inner`
                    unsafe {
                        core::ptr::copy_nonoverlapping(buf.as_ptr(), targ, ml);
                    };
                    // SAFETY: the pointer gets moved by `ml` which is guaranteed to be in bounds
                    targ = unsafe { targ.add(ml) };
                    len += ml;
                }
            }
            len
        }
        let mut len = 0;
        for buf in bufs {
            len += buf.len();
        }
        if len == 0 {
            return Ok(0);
        }
        self.reserve(len);
        let out_len = self.apply_unfilled(|targ| {
            let out_len = inner(targ, bufs);
            (out_len, out_len)
        });
        if out_len != 0 {
            Ok(out_len)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "write_vectored failed to write any data as the buffer is full",
            ))
        }
    }
}

#[cfg(feature = "queue")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
impl Read for crate::ByteQueue<'_> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        fn read_inner(queue: &mut crate::ByteQueue<'_>, mut buf: *mut u8, mut left: usize) {
            loop {
                // SAFETY: we know that the queue is not empty due to thre being data left to read
                let mut aa = unsafe { queue.pop_front().unwrap_unchecked() };
                let ab = aa.as_slice();
                let len = ab.len();
                let ab: *const u8 = ab.as_ptr();
                if len <= left {
                    // SAFETY: we know that the buffer is large enough as we checked it before
                    unsafe { core::ptr::copy_nonoverlapping(ab, buf, len) };
                    left -= len;
                    // SAFETY: we know that the pointer contains at least `len` more items
                    buf = unsafe { buf.add(len) };
                    if left != 0 {
                        continue;
                    }
                    return;
                }
                // SAFETY: we know that the buffer is large enough as we checked it before
                unsafe { core::ptr::copy_nonoverlapping(ab, buf, left) };
                aa.make_sliced(left..);
                queue.push_front(aa);
                return;
            }
        }
        let len = buf.len().min(self.len());
        if len != 0 {
            read_inner(self, buf.as_mut_ptr(), len);
        }
        Ok(len)
    }

    #[allow(clippy::missing_inline_in_public_items)]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> std::io::Result<usize> {
        let mut offs = 0;

        'outer: for buf in bufs {
            let mut buf = buf.as_mut();
            while !buf.is_empty() {
                let Some(mut aa) = self.pop_front() else {
                    return Ok(offs);
                };
                let len = aa.len();
                let left = buf.len();
                if len <= left {
                    buf[..len].copy_from_slice(&aa);
                    buf = &mut buf[len..];
                    offs += len;
                    continue;
                }
                buf.copy_from_slice(&aa.as_slice()[..left]);
                aa.make_sliced(left..);
                self.push_front(aa);
                offs += left;
                continue 'outer;
            }
        }
        Ok(offs)
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let len = self.len();
        if len == 0 {
            return Ok(0);
        }
        let swapped = core::mem::replace(self, crate::ByteQueue::new());
        buf.reserve(len);
        for aa in swapped.into_iter() {
            buf.extend_from_slice(&aa);
        }
        Ok(len)
    }
}

#[cfg(feature = "queue")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
impl BufRead for crate::ByteQueue<'_> {
    #[inline]
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        let Some(ff) = self.front() else {
            return Ok(&[]);
        };
        Ok(ff.as_slice())
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        crate::ByteQueue::consume(self, amt);
    }

    #[inline]
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        fn read_until_inner(
            queue: &mut crate::ByteQueue<'_>,
            byte: u8,
            buf: &mut Vec<u8>,
        ) -> usize {
            let mut offs = 0;

            while let Some(mut aa) = queue.pop_front() {
                let len = aa.len();
                let bb = aa.as_slice();
                let mut i = 0;
                while i < len {
                    if bb[i] == byte {
                        i += 1;
                        buf.extend_from_slice(&bb[..i]);
                        offs += i;
                        aa.make_sliced(i..);
                        queue.push_front(aa);
                        return offs;
                    }
                    i += 1;
                }
                buf.extend_from_slice(aa.as_slice());
                offs += len;
            }

            offs
        }

        if self.is_empty() {
            return Ok(0);
        }
        Ok(read_until_inner(self, byte, buf))
    }
}

#[cfg(feature = "queue")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
impl Write for crate::ByteQueue<'_> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let len = buf.len();
        if len <= crate::ByteChunk::LEN {
            self.push_back(ByteData::from_chunk_slice(buf));
            return Ok(len);
        }
        let buf = ByteData::from_shared(buf.into());
        self.push_back(buf);
        Ok(len)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        let mut shared = crate::SharedBytesBuilder::new();
        let res = Write::write_vectored(&mut shared, bufs)?;
        self.push_back(shared.build());
        Ok(res)
    }
}
