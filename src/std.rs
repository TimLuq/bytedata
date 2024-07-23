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
            return Err(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "write_vectored failed to write any data as the buffer is full",
            ));
        }
    }
}
