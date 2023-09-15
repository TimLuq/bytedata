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
    pub const fn len(&self) -> usize {
        self.off as usize - 8
    }

    /// Returns `true` if the buffer is empty.
    pub const fn is_empty(&self) -> bool {
        self.off == 8
    }

    /// Returns the bytes as a slice.
    pub const fn as_slice(&self) -> &[u8] {
        if self.off == 8 {
            return &[];
        }
        unsafe {
            core::slice::from_raw_parts(self.dat.offset(self.off as isize), self.off as usize - 8)
        }
    }

    /// Returns the bytes as a mut slice.
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        if self.off == 8 {
            return &mut [];
        }
        unsafe {
            core::slice::from_raw_parts_mut(self.dat.offset(self.off as isize), self.off as usize - 8)
        }
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
