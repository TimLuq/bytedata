use std::io::{BufRead, IoSliceMut, Read, Write};

use crate::{ByteData, SharedBytes, SharedBytesBuilder};

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl Read for ByteData<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = buf.len().min(self.len());
        buf.copy_from_slice(&self.as_slice()[..len]);
        self.make_sliced(len..);
        Ok(len)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> std::io::Result<usize> {
        let mut offs = 0;
        let mut slic = self.as_slice();

        for buf in bufs {
            if buf.is_empty() {
                continue;
            }
            if slic.is_empty() {
                break;
            }

            let len = buf.len().min(slic.len());
            buf.copy_from_slice(&slic[..len]);
            slic = &slic[len..];
            offs += len;
        }

        if offs != 0 {
            self.make_sliced(offs..);
        }

        Ok(offs)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let len = self.len();
        buf.extend_from_slice(self.as_slice());
        *self = ByteData::empty();
        Ok(len)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl BufRead for ByteData<'_> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        Ok(self.as_slice())
    }

    fn consume(&mut self, amt: usize) {
        self.make_sliced(amt..);
    }

    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let d = self.as_slice();
        let len = d.len();
        if len == 0 {
            return Ok(0);
        }
        let mut i = 0;
        while i < len {
            if d[i] == byte {
                i += 1;
                buf.extend_from_slice(&d[..i]);
                self.make_sliced(i..);
                return Ok(i);
            }
            i += 1;
        }
        buf.extend_from_slice(d);
        *self = ByteData::empty();
        Ok(len)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl Read for SharedBytes {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = buf.len().min(self.len());
        buf.copy_from_slice(&self.as_slice()[..len]);
        self.make_sliced(len, self.len() - len);
        Ok(len)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> std::io::Result<usize> {
        let mut offs = 0;
        let mut slic = self.as_slice();

        for buf in bufs {
            if buf.is_empty() {
                continue;
            }
            if slic.is_empty() {
                break;
            }

            let len = buf.len().min(slic.len());
            buf.copy_from_slice(&slic[..len]);
            slic = &slic[len..];
            offs += len;
        }

        if offs != 0 {
            self.make_sliced(offs, self.len() - offs);
        }

        Ok(offs)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let len = self.len();
        buf.extend_from_slice(self.as_slice());
        *self = SharedBytes::empty();
        Ok(len)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl BufRead for SharedBytes {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        Ok(self.as_slice())
    }

    fn consume(&mut self, amt: usize) {
        self.make_sliced(amt, self.len() - amt);
    }

    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let d = self.as_slice();
        let len = d.len();
        if len == 0 {
            return Ok(0);
        }
        let mut i = 0;
        while i < len {
            if d[i] == byte {
                i += 1;
                buf.extend_from_slice(&d[..i]);
                self.make_sliced(i, self.len() - i);
                return Ok(i);
            }
            i += 1;
        }
        buf.extend_from_slice(d);
        *self = SharedBytes::empty();
        Ok(len)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl Write for SharedBytesBuilder {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let len = buf.len();
        self.extend_from_slice(buf);
        Ok(len)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        fn inner(targ: &mut [core::mem::MaybeUninit<u8>], bufs: &[std::io::IoSlice<'_>]) -> usize {
            let max = targ.len();
            let mut targ = targ.as_mut_ptr() as *mut u8;
            let mut len = 0;
            for buf in bufs {
                let l = buf.len().min(max - len);
                if l != 0 {
                    unsafe {
                        core::ptr::copy_nonoverlapping(buf.as_ptr(), targ, l);
                        targ = targ.add(l);
                    }
                    len += l;
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
        let len = self.apply_unfilled(|targ| {
            let len = inner(targ, bufs);
            (len, len)
        });
        if len != 0 {
            Ok(len)
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
impl<'a> Read for crate::ByteQueue<'a> {
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        let len = buf.len().min(self.len());
        let mut left = len;
        while left != 0 {
            let mut a = self.pop_front().unwrap();
            let l = a.len();
            if l <= left {
                buf[..l].copy_from_slice(&a);
                left -= l;
                buf = &mut buf[l..];
                continue;
            } else {
                buf.copy_from_slice(&a.as_slice()[..left]);
                a.make_sliced(left..);
                self.push_front(a);
                break;
            }
        }
        Ok(len)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> std::io::Result<usize> {
        let mut offs = 0;

        'outer: for buf in bufs {
            let mut buf = buf.as_mut();
            while !buf.is_empty() {
                let Some(mut a) = self.pop_front() else {
                    return Ok(offs);
                };
                let l = a.len();
                let left = buf.len();
                if l <= left {
                    buf[..l].copy_from_slice(&a);
                    buf = &mut buf[l..];
                    offs += l;
                    continue;
                } else {
                    buf.copy_from_slice(&a.as_slice()[..left]);
                    a.make_sliced(left..);
                    self.push_front(a);
                    offs += left;
                    continue 'outer;
                }
            }
        }
        Ok(offs)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let len = self.len();
        if len == 0 {
            return Ok(0);
        }
        let swapped = std::mem::replace(self, crate::ByteQueue::new());
        buf.reserve(len);
        for a in swapped.into_iter() {
            buf.extend_from_slice(&a);
        }
        Ok(len)
    }
}

#[cfg(feature = "queue")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
impl<'a> BufRead for crate::ByteQueue<'a> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        let Some(f) = self.front() else {
            return Ok(&[]);
        };
        Ok(f.as_slice())
    }

    fn consume(&mut self, amt: usize) {
        crate::ByteQueue::consume(self, amt);
    }

    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        if self.is_empty() {
            return Ok(0);
        }
        let mut offs = 0;

        while let Some(mut a) = self.pop_front() {
            let l = a.len();
            let b = a.as_slice();
            let mut i = 0;
            while i < l {
                if b[i] == byte {
                    i += 1;
                    buf.extend_from_slice(&b[..i]);
                    offs += i;
                    a.make_sliced(i..);
                    self.push_front(a);
                    return Ok(offs);
                }
                i += 1;
            }
            buf.extend_from_slice(&a);
            offs += l;
        }

        Ok(offs)
    }
}

#[cfg(feature = "queue")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
impl<'a> Write for crate::ByteQueue<'a> {
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

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        let mut shared = crate::SharedBytesBuilder::new();
        let res = Write::write_vectored(&mut shared, bufs)?;
        self.push_back(shared.build());
        Ok(res)
    }
}
