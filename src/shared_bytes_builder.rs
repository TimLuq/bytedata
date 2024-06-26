use crate::SharedBytes;

/// A builder for `SharedBytes`.
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub struct SharedBytesBuilder {
    pub(crate) len: u32,
    pub(crate) off: u32,
    pub(crate) dat: *mut u8,
}

unsafe impl Send for SharedBytesBuilder {}

impl SharedBytesBuilder {
    /// Creates a new `SharedBytesBuilder`.
    pub const fn new() -> Self {
        Self {
            len: 0,
            off: 8,
            dat: core::ptr::null_mut(),
        }
    }

    /// Creates a new `SharedBytesBuilder` with the specified capacity. The maximum capacity is `0xFFFF_FFF7`.
    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        if cap > 0xFFFF_FFF7 {
            panic!("SharedBytesBuilder::with_capacity: capacity too large");
        }
        if cap == 0 {
            return Self::new();
        }
        Self::with_capacity_u32(cap as u32)
    }

    fn with_capacity_u32(cap: u32) -> Self {
        let layout = alloc::alloc::Layout::from_size_align(cap as usize + 8, 4).unwrap();
        let ptr = unsafe {
            let ptr = alloc::alloc::alloc(layout);
            (ptr as *mut u32).write_volatile(cap);
            (ptr.offset(4) as *mut u32).write_volatile(0);
            ptr
        };
        Self {
            len: cap + 8,
            off: 8,
            dat: ptr,
        }
    }

    #[cfg_attr(not(feature = "bytes_1"), allow(dead_code))]
    pub(crate) fn reserve_extra(&mut self) {
        let off = self.off as usize;
        if off == 0xFFFF_FFFF {
            return;
        }
        let new_len = if off >= 0x0000_8000 {
            (off & 0xFFFF_8000) + 0x0000_8000
        } else {
            (off + 7).next_power_of_two()
        };
        self.reserve_exact(new_len.min(0xFFFF_FFFF));
    }

    #[cfg_attr(not(feature = "bytes_1"), allow(dead_code))]
    pub(crate) fn reserve(&mut self, additional: usize) {
        let off = self.off as usize;
        if off == 0xFFFF_FFFF {
            return;
        }
        let new_off = off + additional;
        let new_len = if new_off >= 0x0000_8000 {
            (new_off & 0xFFFF_8000) + 0x0000_8000
        } else {
            new_off.next_power_of_two()
        };
        self.reserve_exact(new_len.min(0xFFFF_FFFF));
    }

    pub(crate) fn reserve_exact(&mut self, new_len: usize) {
        if new_len <= self.len as usize {
            return;
        }
        let ptr = if self.len == 0 {
            unsafe {
                let layout = alloc::alloc::Layout::from_size_align(new_len, 4).unwrap();
                let p = alloc::alloc::alloc(layout);
                (p.offset(4) as *mut u32).write_volatile(0);
                p
            }
        } else {
            unsafe {
                let old_layout =
                    alloc::alloc::Layout::from_size_align(self.len as usize, 4).unwrap();
                let mut ptr = alloc::alloc::realloc(self.dat, old_layout, new_len);
                if ptr.is_null() {
                    let layout = alloc::alloc::Layout::from_size_align(new_len, 4).unwrap();
                    ptr = alloc::alloc::alloc(layout);
                    let p = alloc::alloc::alloc(layout);
                    (p.offset(4) as *mut u32).write_volatile(0);
                    ptr.offset(8)
                        .copy_from(self.dat.offset(8), self.off as usize - 8);
                    alloc::alloc::dealloc(self.dat, old_layout);
                }
                ptr
            }
        };
        self.len = new_len as u32;
        self.dat = ptr;
    }

    /// Pushes a slice of bytes to the end of the buffer.
    pub fn extend_from_slice(&mut self, dat: &[u8]) {
        if dat.is_empty() {
            return;
        }
        if dat.len() > 0xFFFF_FFF7 {
            panic!("SharedBytesBuilder::push: slice too large");
        }
        let new_off = self.off as usize + dat.len();
        if new_off > 0xFFFF_FFFF {
            panic!("SharedBytesBuilder::push: slice too large to append to existing data");
        }

        // reallocate if necessary
        if new_off > self.len as usize {
            let new_len = if new_off > 0x0000_8000 {
                ((new_off & 0xFFFF_8000) + 0x0000_8000).min(0xFFFF_FFFF)
            } else {
                new_off.next_power_of_two()
            };
            self.reserve_exact(new_len);
        }

        unsafe {
            self.dat
                .offset(self.off as isize)
                .copy_from(dat.as_ptr(), dat.len());
        }
        self.off += dat.len() as u32;
    }

    /// Freezes the builder and returns a `SharedBytes`.
    pub fn build(self) -> SharedBytes {
        let SharedBytesBuilder { len, off, dat } = self;
        if len == 0 {
            return SharedBytes {
                len: 0,
                off: 0,
                dat: core::ptr::null(),
            };
        }
        unsafe {
            *(dat as *mut u32) = len;
            (dat.offset(4) as *mut u32).write_volatile(1);
        }
        SharedBytes {
            len: off - 8,
            off: 8,
            dat,
        }
    }

    /// Returns the number of bytes currently in the buffer.
    #[inline]
    pub const fn len(&self) -> usize {
        self.off as usize - 8
    }

    /// Returns `true` if the buffer is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.off == 8
    }

    /// Returns the bytes as a slice.
    pub const fn as_slice(&self) -> &[u8] {
        if self.off == 8 {
            return &[];
        }
        unsafe { core::slice::from_raw_parts(self.dat.offset(8), self.off as usize - 8) }
    }

    /// Returns the bytes as a mut slice.
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        if self.off == 8 {
            return &mut [];
        }
        unsafe { core::slice::from_raw_parts_mut(self.dat.offset(8), self.off as usize - 8) }
    }

    /// Apply a function to the unused reserved bytes.
    ///
    /// The function is passed a mutable slice of `MaybeUninit<u8>` and returns a tuple of the return value and the number of bytes filled.
    pub fn apply_unfilled<R, F>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut [core::mem::MaybeUninit<u8>]) -> (R, usize),
    {
        let off = self.off as isize;
        let data = if off == 8 {
            &mut [] as &mut [core::mem::MaybeUninit<u8>]
        } else {
            unsafe {
                core::slice::from_raw_parts_mut(
                    self.dat.offset(off) as *mut core::mem::MaybeUninit<u8>,
                    self.len as usize - off as usize,
                )
            }
        };
        let (ret, len) = f(data);
        assert!(len <= data.len());
        self.len += len as u32;
        ret
    }
}

impl Default for SharedBytesBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SharedBytesBuilder {
    fn drop(&mut self) {
        if self.len != 0 && unsafe { (self.dat.offset(4) as *mut u32).read_volatile() } == 0 {
            unsafe {
                let layout = alloc::alloc::Layout::from_size_align(self.len as usize, 4).unwrap();
                alloc::alloc::dealloc(self.dat, layout);
            }
        }
    }
}

impl AsRef<[u8]> for SharedBytesBuilder {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl AsMut<[u8]> for SharedBytesBuilder {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_slice_mut()
    }
}

impl core::ops::Deref for SharedBytesBuilder {
    type Target = [u8];
    #[inline]
    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl core::ops::DerefMut for SharedBytesBuilder {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8] {
        self.as_slice_mut()
    }
}

impl core::iter::Extend<u8> for SharedBytesBuilder {
    fn extend<I: IntoIterator<Item = u8>>(&mut self, iter: I) {
        for i in iter {
            self.extend_from_slice(&[i]);
        }
    }
}

impl<'a> core::iter::Extend<&'a [u8]> for SharedBytesBuilder {
    fn extend<I: IntoIterator<Item = &'a [u8]>>(&mut self, iter: I) {
        for i in iter {
            self.extend_from_slice(i);
        }
    }
}

impl<'a> core::iter::Extend<&'a str> for SharedBytesBuilder {
    fn extend<I: IntoIterator<Item = &'a str>>(&mut self, iter: I) {
        for i in iter {
            self.extend_from_slice(i.as_bytes());
        }
    }
}

impl core::fmt::Write for SharedBytesBuilder {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.extend_from_slice(s.as_bytes());
        Ok(())
    }
}

impl core::fmt::LowerHex for SharedBytesBuilder {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = self.as_slice();
        if let Some(w) = f.width() {
            if w > s.len() * 2 {
                for _ in 0..w - s.len() * 2 {
                    core::fmt::Write::write_str(f, "0")?;
                }
            }
        }
        let mut i = 0;
        while i < s.len() {
            write!(f, "{:02x}", s[i])?;
            i += 1;
        }
        Ok(())
    }
}

impl core::fmt::UpperHex for SharedBytesBuilder {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = self.as_slice();
        if let Some(w) = f.width() {
            if w > s.len() * 2 {
                for _ in 0..w - s.len() * 2 {
                    core::fmt::Write::write_str(f, "0")?;
                }
            }
        }
        let mut i = 0;
        while i < s.len() {
            write!(f, "{:02X}", s[i])?;
            i += 1;
        }
        Ok(())
    }
}

#[cfg(feature = "read_buf")]
impl SharedBytesBuilder {
    /// Apply a function to the unused reserved bytes.
    pub fn apply_borrowed_buf<'this, R, F>(&'this mut self, f: F) -> R
    where
        F: FnOnce(&mut std::io::BorrowedBuf<'this>) -> R,
    {
        let off = self.off as isize;
        let mut bb = if off == 8 {
            std::io::BorrowedBuf::from(&mut [] as &mut [u8])
        } else {
            let data = unsafe { self.dat.offset(off) as *mut core::mem::MaybeUninit<u8> };
            let data =
                unsafe { core::slice::from_raw_parts_mut(data, self.len as usize - off as usize) };
            std::io::BorrowedBuf::from(data)
        };
        let ret = f(&mut bb);
        let len = bb.len() as u32;
        self.len += len;
        ret
    }
}
