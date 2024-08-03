use crate::{SharedBytes, SharedBytesMeta};

/// A builder for `SharedBytes`.
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub struct SharedBytesBuilder {
    /// The total capacity of the buffer.
    pub(crate) len: u32,
    /// The offset of the first unused byte.
    pub(crate) off: u32,
    /// Pointer to the heap buffer.
    pub(crate) dat: *mut u8,
    /// The alignment of the buffer.
    pub(crate) align: usize,
}

unsafe impl Send for SharedBytesBuilder {}
unsafe impl Sync for SharedBytesBuilder {}

impl SharedBytesBuilder {
    /// Creates a new `SharedBytesBuilder`.
    #[inline]
    pub const fn new() -> Self {
        Self {
            len: 0,
            off: core::mem::size_of::<SharedBytesMeta>() as u32,
            dat: core::ptr::null_mut(),
            align: core::mem::align_of::<SharedBytesMeta>(),
        }
    }

    /// Creates a new `SharedBytesBuilder`.
    #[inline]
    pub const fn with_alignment(alignment: usize) -> Self {
        let align = alignment.next_power_of_two();
        assert!(
            align == alignment,
            "SharedBytesBuilder::with_alignment: alignment must be a power of two"
        );
        assert!(
            align <= 512,
            "SharedBytesBuilder::with_alignment: alignment must be less than or equal to 512"
        );
        let align = if align < core::mem::align_of::<SharedBytesMeta>() {
            core::mem::align_of::<SharedBytesMeta>()
        } else {
            align
        };
        let off = SharedBytesMeta::compute_start_offset(align);
        Self {
            len: 0,
            off,
            dat: core::ptr::null_mut(),
            align,
        }
    }

    /// Creates a new `SharedBytesBuilder` with at least the specified capacity. The maximum capacity is `0xFFFF_FFF0` or `isize::MAX - 15`, whichever is lower.
    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        if cap == 0 {
            return Self::new();
        }
        Self::with_capacity_u32(cap as u32, core::mem::align_of::<SharedBytesMeta>())
    }

    /// Creates a new `SharedBytesBuilder` with at least the specified capacity. The maximum capacity is `0xFFFF_FFF0 - align` or `isize::MAX - 15 - align`, whichever is lower.
    pub fn with_aligned_capacity(cap: usize, alignment: usize) -> Self {
        const MAX_CAP: usize = if isize::BITS <= 32 {
            isize::MAX as usize
        } else {
            0xFFFF_FFFF
        };
        let align = if alignment < core::mem::align_of::<SharedBytesMeta>() {
            core::mem::align_of::<SharedBytesMeta>()
        } else {
            alignment
        };
        let max_cap = MAX_CAP - SharedBytesMeta::compute_start_offset(alignment) as usize;
        assert!(
            cap <= max_cap,
            "SharedBytesBuilder::with_aligned_capacity: capacity too large"
        );
        if cap == 0 {
            return Self::with_alignment(align);
        }
        let align2 = alignment.next_power_of_two();
        assert!(
            align2 == alignment,
            "SharedBytesBuilder::with_aligned_capacity: alignment must be a power of two"
        );
        assert!(alignment <= 512, "SharedBytesBuilder::with_aligned_capacity: alignment must be less than or equal to 512");
        Self::with_capacity_u32(cap as u32, align)
    }

    fn with_capacity_u32(cap: u32, align: usize) -> Self {
        let off = SharedBytesMeta::compute_start_offset(align);
        let len = cap + off;
        let layout = alloc::alloc::Layout::from_size_align(len as usize, align).unwrap();
        let ptr = unsafe {
            let ptr = alloc::alloc::alloc(layout);
            if ptr.is_null() {
                alloc::alloc::handle_alloc_error(layout);
            }
            ptr
        };
        Self {
            len,
            off,
            dat: ptr,
            align,
        }
    }

    #[cfg_attr(not(feature = "bytes_1"), allow(dead_code))]
    #[inline]
    pub(crate) fn reserve_extra(&mut self) {
        let off = self.off as usize;
        if off == 0xFFFF_FFFF {
            return;
        }
        let new_len = if off >= 0x0000_8000 {
            (off & 0xFFFF_8000)
                .saturating_add(0x0000_8000)
                .min(0xFFFF_FFFF)
        } else {
            (off + 7).next_power_of_two()
        };
        self.reserve_exact(new_len);
    }

    /// Reserves capacity for at least `additional` more bytes to be written to the buffer.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        let off = self.off as usize;
        if off == 0xFFFF_FFFF || additional == 0 {
            return;
        }
        let new_off = off.saturating_add(additional);
        let new_len = if new_off >= 0x0000_8000 {
            (new_off & 0xFFFF_8000)
                .saturating_add(0x0000_8000)
                .min(0xFFFF_FFFF)
        } else {
            new_off.next_power_of_two()
        };
        self.reserve_exact(new_len);
    }

    pub(crate) fn reserve_exact(&mut self, new_len: usize) {
        if new_len <= self.len as usize {
            return;
        }
        let ptr = if self.len == 0 {
            unsafe {
                let layout = alloc::alloc::Layout::from_size_align(new_len, self.align).unwrap();
                let p = alloc::alloc::alloc(layout);
                if p.is_null() {
                    alloc::alloc::handle_alloc_error(layout);
                }
                p
            }
        } else {
            let start_off = SharedBytesMeta::compute_start_offset(self.align) as usize;
            unsafe {
                let old_layout =
                    alloc::alloc::Layout::from_size_align(self.len as usize, self.align).unwrap();
                let mut ptr = alloc::alloc::realloc(self.dat, old_layout, new_len);
                if ptr.is_null() {
                    let layout =
                        alloc::alloc::Layout::from_size_align(new_len, self.align).unwrap();
                    ptr = alloc::alloc::alloc(layout);
                    let src = self.dat.add(start_off);
                    let dst = ptr.add(start_off);
                    dst.copy_from_nonoverlapping(src, self.off as usize - start_off);
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
        let new_off = match (self.off as usize).checked_add(dat.len()) {
            Some(new_off) if new_off <= 0xFFFF_FFFF => new_off,
            _ => {
                panic!("SharedBytesBuilder::extend_from_slice: slice too large to append to existing data");
            }
        };

        // reallocate if necessary
        if new_off > self.len as usize {
            let new_len = if new_off > 0x0000_8000 {
                (new_off & 0xFFFF_8000)
                    .saturating_add(0x0000_8000)
                    .min(0xFFFF_FFFF)
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

    /// Clear the buffer.
    pub fn clear(&mut self) {
        if self.len == 0 {
            return;
        }
        let data_off = SharedBytesMeta::compute_start_offset(self.align);
        self.off = data_off;
    }

    /// Truncates the buffer. This method does nothing if the buffer contains less than `len` bytes.
    pub fn truncate(&mut self, len: usize) {
        if self.len == 0 {
            return;
        }
        let data_off = SharedBytesMeta::compute_start_offset(self.align).saturating_add(len as u32);
        if self.off > data_off {
            self.off = data_off;
        }
    }

    /// Freezes the builder and returns a [`SharedBytes`] representing the data.
    pub fn build(self) -> SharedBytes {
        let slf = core::mem::ManuallyDrop::new(self);
        let len = slf.len;
        if len == 0 {
            return SharedBytes {
                len: 0,
                off: 0,
                dat_addr: 0,
            };
        }
        let align = slf.align;
        let data_off = SharedBytesMeta::compute_start_offset(align);
        let off = slf.off;
        let dat = slf.dat;
        if data_off == off {
            if !dat.is_null() {
                unsafe {
                    let layout =
                        alloc::alloc::Layout::from_size_align(len as usize, align).unwrap();
                    alloc::alloc::dealloc(dat, layout);
                }
            }
            return SharedBytes {
                len: 0,
                off: 0,
                dat_addr: 0,
            };
        }
        #[allow(clippy::declare_interior_mutable_const)]
        const INIT: SharedBytesMeta = SharedBytesMeta::new().with_refcount(1);
        unsafe { (dat as *mut SharedBytesMeta).write(INIT.with_align(align).with_len(len)) };
        let dat_addr = dat as usize as u64;
        let dat_addr = dat_addr.to_le();
        SharedBytes {
            len: off - data_off,
            off: data_off,
            dat_addr,
        }
    }

    /// Returns total the number of bytes currently available in the buffer.
    #[inline]
    pub const fn capacity(&self) -> usize {
        if self.len == 0 {
            return 0;
        }
        let data_off = SharedBytesMeta::compute_start_offset(self.align);
        (self.len - data_off) as usize
    }

    /// Returns the number of bytes currently written to in the buffer.
    #[inline]
    pub const fn len(&self) -> usize {
        let data_off = SharedBytesMeta::compute_start_offset(self.align);
        (self.off - data_off) as usize
    }

    /// Returns `true` if the buffer is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        if self.len == 0 {
            return true;
        }
        let data_off = SharedBytesMeta::compute_start_offset(self.align);
        self.off == data_off
    }

    /// Returns the bytes as a slice.
    pub const fn as_slice(&self) -> &[u8] {
        if self.len == 0 {
            return &[];
        }
        let data_off = SharedBytesMeta::compute_start_offset(self.align);
        if self.off == data_off {
            return &[];
        }
        unsafe {
            core::slice::from_raw_parts(
                self.dat.offset(data_off as isize),
                (self.off - data_off) as usize,
            )
        }
    }

    /// Returns the bytes as a mut slice.
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        if self.len == 0 {
            return &mut [];
        }
        let data_off = SharedBytesMeta::compute_start_offset(self.align);
        if self.off == data_off {
            return &mut [];
        }
        unsafe {
            core::slice::from_raw_parts_mut(
                self.dat.offset(data_off as isize),
                (self.off - data_off) as usize,
            )
        }
    }

    /// Apply a function to the unused reserved bytes.
    ///
    /// The function is passed a mutable slice of `MaybeUninit<u8>` and returns a tuple of the return value and the number of bytes filled.
    pub fn apply_unfilled<R, F>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut [core::mem::MaybeUninit<u8>]) -> (R, usize),
    {
        let data = if self.len == 0 || self.off == self.len {
            &mut [] as &mut [core::mem::MaybeUninit<u8>]
        } else {
            let off = self.off as isize;
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
        if self.len == 0 {
            return;
        }
        unsafe {
            let layout =
                alloc::alloc::Layout::from_size_align(self.len as usize, self.align).unwrap();
            alloc::alloc::dealloc(self.dat, layout);
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

#[cfg(feature = "core_io_borrowed_buf")]
#[cfg_attr(docsrs, doc(cfg(feature = "core_io_borrowed_buf")))]
impl SharedBytesBuilder {
    /// Apply a function to the unused reserved bytes.
    pub fn apply_borrowed_buf<'this, R, F>(&'this mut self, f: F) -> R
    where
        F: FnOnce(&mut core::io::BorrowedBuf<'this>) -> R,
    {
        let off = self.off as isize;
        let mut bb = if self.len == 0 {
            core::io::BorrowedBuf::from(&mut [] as &mut [u8])
        } else {
            let data = unsafe { self.dat.offset(off) as *mut core::mem::MaybeUninit<u8> };
            let data =
                unsafe { core::slice::from_raw_parts_mut(data, self.len as usize - off as usize) };
            core::io::BorrowedBuf::from(data)
        };
        let ret = f(&mut bb);
        let len = bb.len() as u32;
        self.len += len;
        ret
    }
}
