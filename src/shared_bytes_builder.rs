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

impl core::fmt::Debug for SharedBytesBuilder {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SharedBytesBuilder")
            .field(
                "data",
                &crate::byte_string_render::ByteStringRender::from_slice(self.as_slice()),
            )
            .field("len", &self.len())
            .field("capacity", &self.capacity())
            .field("align", &self.align)
            .finish()
    }
}

// SAFETY: `SharedBytesBuilder` is `Send` and `Sync` because it's safe to share the heap data across threads.
unsafe impl Send for SharedBytesBuilder {}
// SAFETY: `SharedBytesBuilder` is `Send` and `Sync` because it's safe to share the heap data across threads.
unsafe impl Sync for SharedBytesBuilder {}

impl SharedBytesBuilder {
    /// Creates a new `SharedBytesBuilder`.
    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn new() -> Self {
        Self {
            len: 0,
            off: core::mem::size_of::<SharedBytesMeta>() as u32,
            dat: core::ptr::null_mut(),
            align: core::mem::align_of::<SharedBytesMeta>(),
        }
    }

    /// Creates a new `SharedBytesBuilder`.
    ///
    /// # Panics
    ///
    /// Panics if the alignment is not a power of two or is greater than 512 or the maximum allowed by the system.
    #[inline]
    #[must_use]
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

    const fn cap_check(cap: usize, alignment: usize) -> usize {
        const MAX_CAP: usize = if isize::BITS <= 32 {
            isize::MAX as usize
        } else {
            0xFFFF_FFFF
        };
        let alignment = if alignment < core::mem::align_of::<SharedBytesMeta>() {
            core::mem::align_of::<SharedBytesMeta>()
        } else {
            alignment
        };
        let max_cap = MAX_CAP - SharedBytesMeta::compute_start_offset(alignment) as usize;
        assert!(
            cap <= max_cap,
            "SharedBytesBuilder::with_aligned_capacity: capacity too large"
        );
        alignment
    }

    /// Creates a new `SharedBytesBuilder` with at least the specified capacity. The maximum capacity is `0xFFFF_FFF0` or `isize::MAX - 15`, whichever is lower.
    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn with_capacity(cap: usize) -> Self {
        if cap == 0 {
            return Self::new();
        }
        let align = Self::cap_check(cap, core::mem::align_of::<SharedBytesMeta>());
        Self::with_capacity_u32(cap as u32, align)
    }

    /// Creates a new `SharedBytesBuilder` with at least the specified capacity. The maximum capacity is `0xFFFF_FFF0 - align` or `isize::MAX - 15 - align`, whichever is lower.
    ///
    /// # Panics
    ///
    /// Panics if the capacity is too large.
    #[inline]
    #[must_use]
    pub fn with_aligned_capacity(cap: usize, alignment: usize) -> Self {
        if cap == 0 {
            return Self::with_alignment(alignment);
        }
        assert!(alignment <= 512, "SharedBytesBuilder::with_aligned_capacity: alignment must be less than or equal to 512");
        let align = Self::cap_check(cap, core::mem::align_of::<SharedBytesMeta>());
        let align2 = alignment.next_power_of_two();
        assert!(
            align2 == alignment,
            "SharedBytesBuilder::with_aligned_capacity: alignment must be a power of two"
        );
        #[allow(clippy::cast_possible_truncation)]
        let cap = cap as u32;
        Self::with_capacity_u32(cap, align)
    }

    fn with_capacity_u32(cap: u32, align: usize) -> Self {
        let off = SharedBytesMeta::compute_start_offset(align);
        let len = cap + off;
        #[allow(clippy::unwrap_used)]
        let layout = core::alloc::Layout::from_size_align(len as usize, align).unwrap();
        // SAFETY: the layout must be valid
        let ptr = unsafe { alloc::alloc::alloc(layout) };
        if ptr.is_null() {
            alloc::alloc::handle_alloc_error(layout);
        }
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
            #[allow(clippy::unwrap_used)]
            let layout = core::alloc::Layout::from_size_align(new_len, self.align).unwrap();
            // SAFETY: the layout must be valid
            let ptr = unsafe { alloc::alloc::alloc(layout) };
            if ptr.is_null() {
                alloc::alloc::handle_alloc_error(layout);
            }
            ptr
        } else {
            let start_off = SharedBytesMeta::compute_start_offset(self.align) as usize;
            #[allow(clippy::unwrap_used)]
            let old_layout =
                core::alloc::Layout::from_size_align(self.len as usize, self.align).unwrap();
            // SAFETY: `old_layout` is the layout of the old allocation.
            let mut ptr = unsafe { alloc::alloc::realloc(self.dat, old_layout, new_len) };
            if ptr.is_null() {
                #[allow(clippy::unwrap_used)]
                let layout = core::alloc::Layout::from_size_align(new_len, self.align).unwrap();
                // SAFETY: the layout must be valid
                ptr = unsafe { alloc::alloc::alloc(layout) };
                if ptr.is_null() {
                    alloc::alloc::handle_alloc_error(layout);
                }
                // SAFETY: `start_off` is always less than or equal to `len`.
                let src = unsafe { self.dat.add(start_off) };
                // SAFETY: `start_off` is always less than or equal to `len`.
                let dst = unsafe { ptr.add(start_off) };
                // SAFETY: `start_off` is always less than or equal to `len`.
                unsafe {
                    dst.copy_from_nonoverlapping(src, self.off as usize - start_off);
                };
                // SAFETY: `old_layout` is the layout of the old allocation.
                unsafe {
                    alloc::alloc::dealloc(self.dat, old_layout);
                };
            }
            ptr
        };
        #[allow(clippy::cast_possible_truncation)]
        let new_len = new_len as u32;
        self.len = new_len;
        self.dat = ptr;
    }

    /// Pushes a slice of bytes to the end of the buffer.
    ///
    /// # Panics
    ///
    /// Panics if the slice is too large to append to the existing data.
    #[inline]
    pub fn extend_from_slice(&mut self, dat: &[u8]) {
        fn extend_from_slice_inner(builder: &mut SharedBytesBuilder, dat: &[u8]) {
            let off = builder.off as usize;

            #[allow(clippy::panic)]
            let new_off = match off.checked_add(dat.len()) {
                Some(new_off) if new_off <= 0xFFFF_FFFF => new_off,
                _ => {
                    panic!("SharedBytesBuilder::extend_from_slice: slice too large to append to existing data");
                }
            };

            // reallocate if necessary
            if new_off > builder.len as usize {
                let new_len = if new_off > 0x0000_8000 {
                    (new_off & 0xFFFF_8000)
                        .saturating_add(0x0000_8000)
                        .min(0xFFFF_FFFF)
                } else {
                    new_off.next_power_of_two()
                };
                builder.reserve_exact(new_len);
            }

            // SAFETY: `off` is always less than or equal to the allocated `len`.
            let dest = unsafe { builder.dat.add(off) };
            let len = dat.len();

            // SAFETY: `dest` is a valid pointer to `len` bytes.
            unsafe {
                core::ptr::copy_nonoverlapping(dat.as_ptr(), dest, len);
            };
            #[allow(clippy::cast_possible_truncation)]
            let new_off = new_off as u32;
            builder.off = new_off;
        }

        if dat.is_empty() {
            return;
        }
        extend_from_slice_inner(self, dat);
    }

    /// Clear the buffer.
    #[inline]
    pub fn clear(&mut self) {
        if self.len == 0 {
            return;
        }
        let data_off = SharedBytesMeta::compute_start_offset(self.align);
        self.off = data_off;
    }

    /// Truncates the buffer. This method does nothing if the buffer contains less than `len` bytes.
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        if self.len == 0 {
            return;
        }
        #[allow(clippy::cast_possible_truncation)]
        let data_off = SharedBytesMeta::compute_start_offset(self.align).saturating_add(len as u32);
        if self.off > data_off {
            self.off = data_off;
        }
    }

    /// Freezes the builder and returns a [`SharedBytes`] representing the data.
    #[inline]
    #[must_use]
    pub fn build(self) -> SharedBytes {
        #[allow(clippy::needless_pass_by_value)]
        fn build_inner(slf: core::mem::ManuallyDrop<SharedBytesBuilder>) -> SharedBytes {
            #[allow(clippy::declare_interior_mutable_const)]
            const INIT: SharedBytesMeta = SharedBytesMeta::new().with_refcount(1);

            let len = slf.len;
            let align = slf.align;
            let data_off = SharedBytesMeta::compute_start_offset(align);
            let off = slf.off;
            let dat = slf.dat;
            if data_off == off {
                if !dat.is_null() {
                    #[allow(clippy::unwrap_used)]
                    let layout = core::alloc::Layout::from_size_align(len as usize, align).unwrap();
                    // SAFETY: the layout must have been allocated
                    unsafe {
                        alloc::alloc::dealloc(dat, layout);
                    };
                }
                return SharedBytes {
                    len: 0,
                    off: 0,
                    dat_addr: 0,
                };
            }
            #[allow(clippy::cast_ptr_alignment)]
            let meta = dat.cast::<SharedBytesMeta>();
            // SAFETY: `meta` is a valid pointer to a `SharedBytesMeta`.
            unsafe { meta.write(INIT.with_align(align).with_len(len)) };
            let dat_addr = dat as usize as u64;
            let dat_addr = dat_addr.to_le();
            SharedBytes {
                len: off - data_off,
                off: data_off,
                dat_addr,
            }
        }

        let this = core::mem::ManuallyDrop::new(self);
        if this.len == 0 {
            return SharedBytes {
                len: 0,
                off: 0,
                dat_addr: 0,
            };
        }

        build_inner(this)
    }

    /// Returns total the number of bytes currently available in the buffer.
    #[inline]
    #[must_use]
    pub const fn capacity(&self) -> usize {
        if self.len == 0 {
            return 0;
        }
        let data_off = SharedBytesMeta::compute_start_offset(self.align);
        (self.len - data_off) as usize
    }

    /// Returns the number of bytes currently written to in the buffer.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        let data_off = SharedBytesMeta::compute_start_offset(self.align);
        (self.off - data_off) as usize
    }

    /// Returns `true` if the buffer is empty.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        if self.len == 0 {
            return true;
        }
        let data_off = SharedBytesMeta::compute_start_offset(self.align);
        self.off == data_off
    }

    /// Returns the bytes as a slice.
    #[inline]
    #[must_use]
    pub const fn as_slice(&self) -> &[u8] {
        if self.len == 0 {
            return &[];
        }
        let data_off = SharedBytesMeta::compute_start_offset(self.align);
        if self.off == data_off {
            return &[];
        }
        // SAFETY: `data_off` is the start offset of the data.
        let dat = unsafe { self.dat.add(data_off as usize) };
        let len = (self.off - data_off) as usize;
        // SAFETY: `len` is the length of the data.
        unsafe { core::slice::from_raw_parts(dat, len) }
    }

    /// Returns the bytes as a mut slice.
    #[inline]
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        if self.len == 0 {
            return &mut [];
        }
        let data_off = SharedBytesMeta::compute_start_offset(self.align);
        // SAFETY: `data_off` is the start offset of the data.
        let dat = unsafe { self.dat.add(data_off as usize) };
        let len = (self.off - data_off) as usize;
        // SAFETY: `len` is the length of the data.
        unsafe { core::slice::from_raw_parts_mut(dat, len) }
    }

    /// Apply a function to the unused reserved bytes.
    ///
    /// The function is passed a mutable slice of `MaybeUninit<u8>` and returns a tuple of the return value and the number of bytes filled.
    ///
    /// # Panics
    ///
    /// Panics if the `fun` function returns a length greater than the reserved capacity.
    #[inline]
    pub fn apply_unfilled<R, F>(&mut self, fun: F) -> R
    where
        F: FnOnce(&mut [core::mem::MaybeUninit<u8>]) -> (R, usize),
    {
        let data = if self.len == 0 || self.off == self.len {
            &mut [] as &mut [core::mem::MaybeUninit<u8>]
        } else {
            let off = self.off as usize;
            // SAFETY: `off` is always less than or equal to `len`.
            let dat = unsafe { self.dat.add(off) };
            let dat = dat.cast::<core::mem::MaybeUninit<u8>>();
            let len = self.len as usize - off;
            // SAFETY: `len` is the allocated length minus the start offset
            unsafe { core::slice::from_raw_parts_mut(dat, len) }
        };
        let (ret, len) = fun(data);
        assert!(
            len <= data.len(),
            "SharedBytesBuilder::apply_unfilled: returned length exceeds reserved capacity"
        );
        #[allow(clippy::cast_possible_truncation)]
        {
            self.off += len as u32;
        };
        ret
    }

    /// Apply a function to the unused reserved bytes in an async context.
    ///
    /// The function is passed a pinned mutable slice of `MaybeUninit<u8>` and returns a future that when ready outputs a tuple of the final return value and the number of bytes filled.
    ///
    /// # Panics
    ///
    /// Panics if the `fun` function returns a length greater than the reserved capacity.
    #[inline]
    pub async fn apply_unfilled_async<'l, R, Fut, Fun>(&'l mut self, fun: Fun) -> R
    where
        Fut: core::future::Future<Output = (R, usize)> + Send + 'l,
        Fun: (FnOnce(core::pin::Pin<&'l mut [core::mem::MaybeUninit<u8>]>) -> Fut) + Send + 'l,
    {
        let data = if self.len == 0 || self.off == self.len {
            &mut [] as &mut [core::mem::MaybeUninit<u8>]
        } else {
            let off = self.off as usize;
            // SAFETY: `off` is always less than or equal to `len`.
            let dat = unsafe { self.dat.add(off) };
            let dat = dat.cast::<core::mem::MaybeUninit<u8>>();
            let len = self.len as usize - off;
            // SAFETY: `len` is the allocated length minus the start offset
            unsafe { core::slice::from_raw_parts_mut(dat, len) }
        };
        let maxlen = data.len();
        let data = core::pin::Pin::new(data);
        let (ret, len) = fun(data).await;
        assert!(
            len <= maxlen,
            "SharedBytesBuilder::apply_unfilled_async: returned length exceeds reserved capacity"
        );
        #[allow(clippy::cast_possible_truncation)]
        {
            self.off += len as u32;
        };
        ret
    }
}

impl Default for SharedBytesBuilder {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SharedBytesBuilder {
    #[inline]
    #[allow(clippy::unwrap_used)]
    fn drop(&mut self) {
        if self.len == 0 {
            return;
        }
        let layout = core::alloc::Layout::from_size_align(self.len as usize, self.align).unwrap();
        // SAFETY: `len` is the allocated length.
        unsafe {
            alloc::alloc::dealloc(self.dat, layout);
        };
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

impl From<SharedBytesBuilder> for crate::ByteData<'_> {
    #[inline]
    fn from(value: SharedBytesBuilder) -> Self {
        value.build().into()
    }
}

impl From<&str> for SharedBytesBuilder {
    #[inline]
    fn from(value: &str) -> Self {
        if value.is_empty() {
            return SharedBytesBuilder::new();
        }
        let mut builder = SharedBytesBuilder::with_capacity(value.len());
        builder.extend_from_slice(value.as_bytes());
        builder
    }
}

impl From<&[u8]> for SharedBytesBuilder {
    #[inline]
    fn from(value: &[u8]) -> Self {
        if value.is_empty() {
            return SharedBytesBuilder::new();
        }
        let mut builder = SharedBytesBuilder::with_capacity(value.len());
        builder.extend_from_slice(value);
        builder
    }
}

impl core::iter::Extend<u8> for SharedBytesBuilder {
    #[inline]
    fn extend<I: IntoIterator<Item = u8>>(&mut self, iter: I) {
        for i in iter {
            self.extend_from_slice(&[i]);
        }
    }
}

impl<'a> core::iter::Extend<&'a u8> for SharedBytesBuilder {
    #[inline]
    fn extend<I: IntoIterator<Item = &'a u8>>(&mut self, iter: I) {
        for i in iter {
            self.extend_from_slice(core::slice::from_ref(i));
        }
    }
}

impl<'a> core::iter::Extend<&'a [u8]> for SharedBytesBuilder {
    #[inline]
    fn extend<I: IntoIterator<Item = &'a [u8]>>(&mut self, iter: I) {
        for i in iter {
            self.extend_from_slice(i);
        }
    }
}

impl<'a> core::iter::Extend<&'a str> for SharedBytesBuilder {
    #[inline]
    fn extend<I: IntoIterator<Item = &'a str>>(&mut self, iter: I) {
        for i in iter {
            self.extend_from_slice(i.as_bytes());
        }
    }
}

impl core::fmt::Write for SharedBytesBuilder {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.extend_from_slice(s.as_bytes());
        Ok(())
    }
}

impl core::fmt::LowerHex for SharedBytesBuilder {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::byte_string_render::lower_hex_slice(self.as_slice(), f)
    }
}

impl core::fmt::UpperHex for SharedBytesBuilder {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::byte_string_render::upper_hex_slice(self.as_slice(), f)
    }
}

#[cfg(feature = "core_io_borrowed_buf")]
#[cfg_attr(docsrs, doc(cfg(feature = "core_io_borrowed_buf")))]
#[allow(clippy::multiple_inherent_impl)]
impl SharedBytesBuilder {
    /// Apply a function to the unused reserved bytes.
    #[inline]
    pub fn apply_borrowed_buf<'this, R, F>(&'this mut self, fun: F) -> R
    where
        F: FnOnce(&mut core::io::BorrowedBuf<'this>) -> R,
    {
        let off = self.off as usize;
        let mut bb = if self.len == 0 {
            core::io::BorrowedBuf::from(&mut [] as &mut [u8])
        } else {
            // SAFETY: `off` is always less than or equal to `len`.
            let data = unsafe { self.dat.add(off) };
            let data = data.cast::<core::mem::MaybeUninit<u8>>();
            let len = self.len as usize - off;
            // SAFETY: `len` is the allocated length minus the start offset
            let data = unsafe { core::slice::from_raw_parts_mut(data, len) };
            core::io::BorrowedBuf::from(data)
        };
        let ret = fun(&mut bb);
        #[allow(clippy::cast_possible_truncation)]
        let len = bb.len() as u32;
        self.len += len;
        ret
    }
}
