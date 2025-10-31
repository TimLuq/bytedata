//! # `ExternalBytes`
//!
//! External bytes are byte data that is not owned by the `bytedata` crate.
//! This can be used to create `ByteData` instances from byte data that is owned by another crate.

/// A structure describing the operations that can be performed on an external byte data type.
#[allow(missing_debug_implementations)]
pub struct ExternalOps<T> {
    drop: Option<unsafe fn(*mut T)>,
    as_slice: fn(&T) -> &[u8],
}

impl<T: Sized> ExternalOps<T> {
    /// Create a new `ExternalOps` instance.
    #[inline]
    pub const fn new(as_slice: fn(&T) -> &[u8]) -> Self {
        let drop: Option<unsafe fn(*mut T)> = if core::mem::needs_drop::<T>() {
            Some(core::ptr::drop_in_place::<T>)
        } else {
            None
        };
        Self { drop, as_slice }
    }
}

/// A trait for types that can be used as external byte data.
pub trait IntoExternalBytes: Sized {
    /// The external byte data type.
    type External: ExternalBytes + From<Self>;
}

impl<T: ExternalBytes> IntoExternalBytes for T {
    type External = T;
}

impl IntoExternalBytes for alloc::string::String {
    type External = alloc::vec::Vec<u8>;
}

/// A trait for types that can be used as external byte data.
pub trait ExternalBytes: core::any::Any + Sync + Sized + 'static {
    /// The operations that can be performed on this type.
    const OPS: ExternalOps<Self>;
}

impl ExternalBytes for alloc::boxed::Box<[u8]> {
    const OPS: ExternalOps<Self> = ExternalOps::new(|x| x);
}

impl ExternalBytes for alloc::sync::Arc<[u8]> {
    const OPS: ExternalOps<Self> = ExternalOps::new(|x| x);
}

impl ExternalBytes for alloc::vec::Vec<u8> {
    #[allow(clippy::incompatible_msrv)]
    const OPS: ExternalOps<Self> = ExternalOps::new(Self::as_slice);
}

impl ExternalBytes for alloc::sync::Arc<str> {
    const OPS: ExternalOps<Self> = ExternalOps::new(|x| x.as_bytes());
}

impl ExternalBytes for alloc::boxed::Box<str> {
    const OPS: ExternalOps<Self> = ExternalOps::new(|x| x.as_bytes());
}

pub(crate) struct ExtBytesWrapper {
    drop: Option<unsafe fn(*mut ())>,
    alloc: usize,
    ref_count: core::sync::atomic::AtomicU32,
    kind: core::any::TypeId,
    align: u32,
}

pub(crate) struct ExtBytesRef {
    data: *const ExtBytesWrapper,
    ptr: *const u8,
    len: usize,
}

impl ExtBytesRef {
    #[inline]
    pub(crate) const fn as_slice(&self) -> &[u8] {
        // SAFETY: `ptr` is a valid pointer to `u8`.
        unsafe { core::slice::from_raw_parts(self.ptr, self.len) }
    }

    #[inline]
    pub(crate) const fn len(&self) -> usize {
        self.len
    }
}

pub(crate) struct TakeExtBytesInner<'a, T> {
    data: &'a mut T,
    slice: &'a [u8],
}
impl<'a, T> TakeExtBytesInner<'a, T> {
    #[inline]
    pub(crate) fn with_slice_ref<'b, F: FnOnce(&'b T, &'b [u8]) -> R, R>(&'b self, fun: F) -> R
    where
        'a: 'b,
    {
        fun(self.data, self.slice)
    }
    #[inline]
    pub(crate) fn into_inner(self) -> &'a mut T {
        self.data
    }
}

#[repr(C)]
pub(crate) struct ExtBytes {
    magic: [u8; 8],
    data: *const ExtBytesRef,
}

// SAFETY: `ExtBytes` is a transparent wrapper around an owned box to `ExtBytesRef`.
unsafe impl Send for ExtBytes {}
// SAFETY: `ExtBytes` is a transparent wrapper around an owned box to `ExtBytesRef`.
unsafe impl Sync for ExtBytes {}

pub(crate) const KIND_EXT_BYTES: u8 = 0b0000_0011;

impl ExtBytes {
    const MAGIC: [u8; 8] = [KIND_EXT_BYTES, 0, 0, 0, 0, 0, 0, 0];

    pub(crate) fn create<'a, T: IntoExternalBytes>(ext_bytes: T) -> crate::ByteData<'a> {
        let ext_bytes = ext_bytes.into();
        let as_slice = <T::External>::OPS.as_slice;
        {
            // Try to use the data as a short chunk
            let sl = as_slice(&ext_bytes);
            let len = sl.len();
            if len <= crate::ByteChunk::LEN {
                return crate::ByteData::from_chunk_slice(sl);
            }
        }

        let align =
            core::mem::align_of::<T::External>().max(core::mem::align_of::<ExtBytesWrapper>());
        let mut alloc =
            core::mem::size_of::<T::External>() + core::mem::size_of::<ExtBytesWrapper>();
        if alloc % align != 0 {
            alloc += align - (alloc % align);
        }
        let mut offset = core::mem::size_of::<ExtBytesWrapper>();
        if offset % align != 0 {
            offset += align - (offset % align);
        }

        let (data, len, ptr) = {
            #[allow(clippy::unwrap_used)]
            let layout = core::alloc::Layout::from_size_align(alloc, align).unwrap();
            // SAFETY: `layout` is a valid layout.
            let data = unsafe { alloc::alloc::alloc(layout) };
            if data.is_null() {
                alloc::alloc::handle_alloc_error(layout);
            }

            // SAFETY: `data` is a valid pointer to an allocated area.
            let payload = unsafe { data.add(offset).cast::<T::External>() };
            // SAFETY: writing to the location we just calculated is safe.
            unsafe {
                payload.write(ext_bytes);
            };

            // SAFETY: `payload` is a valid pointer to `T::External` as we just wrote to it.
            let sl = as_slice(unsafe { &*payload });
            let len = sl.len();
            let ptr = sl.as_ptr();

            if len <= crate::ByteChunk::LEN {
                let aa = crate::ByteData::from_chunk_slice(sl);
                if let Some(drop) = <T::External>::OPS.drop {
                    // SAFETY: `payload` is a valid pointer to `T` which should be dropped.
                    unsafe { drop(payload) };
                }
                // SAFETY: `data` is a valid pointer to an allocated area.
                unsafe { alloc::alloc::dealloc(data, layout) };
                return aa;
            }

            #[allow(clippy::cast_ptr_alignment)]
            let header = data.cast::<ExtBytesWrapper>();
            let item = ExtBytesWrapper {
                // SAFETY: `T::OPS.drop` is an optional function pointer.
                drop: unsafe {
                    core::mem::transmute::<
                        Option<unsafe fn(*mut T::External)>,
                        Option<unsafe fn(*mut ())>,
                    >(<T::External>::OPS.drop)
                },
                alloc,
                ref_count: core::sync::atomic::AtomicU32::new(1),
                #[allow(clippy::cast_possible_truncation)]
                align: align as u32,
                kind: core::any::TypeId::of::<T::External>(),
            };
            // SAFETY: `header` is a valid pointer to `ExtBytesWrapper`.
            unsafe { header.write(item) };

            (header, len, ptr)
        };

        let data =
            alloc::boxed::Box::into_raw(alloc::boxed::Box::new(ExtBytesRef { data, ptr, len }));

        crate::ByteData {
            external: core::mem::ManuallyDrop::new(Self {
                magic: Self::MAGIC,
                data,
            }),
        }
    }

    #[inline]
    pub(crate) const fn as_slice(&self) -> &[u8] {
        // SAFETY: `data` is a valid pointer to `ExtBytesRef`.
        unsafe { (*self.data).as_slice() }
    }

    #[inline]
    pub(crate) const fn len(&self) -> usize {
        // SAFETY: `data` is a valid pointer to `ExtBytesRef`.
        unsafe { (*self.data).len() }
    }

    pub(crate) fn make_sliced_range<
        R: core::ops::RangeBounds<usize> + core::slice::SliceIndex<[u8], Output = [u8]>,
    >(
        &mut self,
        range: R,
    ) {
        debug_assert_eq!(
            self.magic[0], KIND_EXT_BYTES,
            "invalid magic number in ExtBytes"
        );
        debug_assert!(!self.data.is_null(), "null pointer in ExtBytes");

        // SAFETY: `data` is a valid pointer to `ExtBytesRef`.
        let dd = unsafe { &mut *self.data.cast_mut() };
        let len = dd.len();
        let start = match range.start_bound() {
            core::ops::Bound::Included(&st) => st,
            core::ops::Bound::Excluded(&st) => st + 1,
            core::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            core::ops::Bound::Included(&end) => end + 1,
            core::ops::Bound::Excluded(&end) => end,
            core::ops::Bound::Unbounded => len,
        };
        assert!(end <= len && start <= len, "index out of bounds");
        assert!(end >= start, "end < start");
        let new_len = end - start;
        // SAFETY: `dd.ptr` is a valid pointer to `u8`.
        dd.ptr = unsafe { dd.ptr.add(start) };
        dd.len = new_len;
    }

    pub(crate) fn sliced_range<
        's,
        R: core::ops::RangeBounds<usize> + core::slice::SliceIndex<[u8], Output = [u8]>,
    >(
        &self,
        range: R,
    ) -> crate::ByteData<'s> {
        debug_assert_eq!(
            self.magic[0], KIND_EXT_BYTES,
            "invalid magic number in ExtBytes"
        );
        debug_assert!(!self.data.is_null(), "null pointer in ExtBytes");
        // SAFETY: `data` is a valid pointer to `ExtBytesRef`.
        let dd = unsafe { &*self.data };
        let len = dd.len();
        let start = match range.start_bound() {
            core::ops::Bound::Included(&start) => start,
            core::ops::Bound::Excluded(&start) => start + 1,
            core::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            core::ops::Bound::Included(&end) => end + 1,
            core::ops::Bound::Excluded(&end) => end,
            core::ops::Bound::Unbounded => len,
        };
        assert!(end <= len && start <= len, "index out of bounds");
        assert!(end >= start, "end < start");
        let new_len = end - start;
        if new_len <= crate::byte_chunk::ByteChunk::LEN {
            // SAFETY: `dd.ptr` is a valid pointer to `u8`.
            let ptr = unsafe { dd.ptr.add(start) };
            // SAFETY: `ptr` is a valid pointer to `u8`.
            return crate::ByteData::from_chunk_slice(unsafe {
                core::slice::from_raw_parts(ptr, new_len)
            });
        }
        let ret = self.clone();
        // SAFETY: `ret.data` is a valid pointer to `ExtBytesRef`.
        let bref = unsafe { &mut *ret.data.cast_mut() };
        // SAFETY: `bref.ptr` is a valid pointer to `u8`.
        bref.ptr = unsafe { bref.ptr.add(start) };
        bref.len = new_len;
        crate::ByteData {
            external: core::mem::ManuallyDrop::new(ret),
        }
    }

    pub(crate) fn with_inner<T: core::any::Any, R, F: FnOnce(&T, &[u8]) -> R>(
        &self,
        fun: F,
    ) -> Option<R> {
        debug_assert_eq!(
            self.magic[0], KIND_EXT_BYTES,
            "invalid magic number in ExtBytes"
        );
        debug_assert!(!self.data.is_null(), "null pointer in ExtBytes");
        // SAFETY: `data` is a valid pointer to `ExtBytesRef`.
        let dd = unsafe { &*self.data };
        if dd.data.is_null() {
            return None;
        }
        // SAFETY: `ExtBytesRef.data` is a valid pointer to `ExtBytesWrapper`.
        let ee = unsafe { &*dd.data };
        if ee.kind != core::any::TypeId::of::<T>() {
            return None;
        }
        let mut offset = core::mem::size_of::<ExtBytesWrapper>();
        let align_mod = offset % ee.align as usize;
        if align_mod != 0 {
            offset += ee.align as usize - align_mod;
        }
        // SAFETY: `dd.data` is a valid pointer to the container data located at `offset`.
        let t_val = unsafe { dd.data.cast::<u8>().add(offset) };
        // SAFETY: `t_val` should now be cast to `*const T`.
        let t_val = unsafe { &*t_val.cast::<T>() };
        // SAFETY: `dd.ptr` is a valid pointer to the slice start.
        Some(fun(t_val, unsafe {
            core::slice::from_raw_parts(dd.ptr, dd.len)
        }))
    }

    /// Take the inner value of the `ExtBytes` instance if the type matches and there is only one reference.
    pub(crate) fn take_inner<
        T: core::any::Any,
        R,
        F: for<'a> FnOnce(TakeExtBytesInner<'a, T>) -> R,
    >(
        self,
        fun: F,
    ) -> Result<R, Self> {
        debug_assert_eq!(
            self.magic[0], KIND_EXT_BYTES,
            "invalid magic number in ExtBytes"
        );
        debug_assert!(!self.data.is_null(), "null pointer in ExtBytes");
        // SAFETY: `data` is a valid pointer to `ExtBytesRef`.
        let dd = unsafe { &*self.data };
        if dd.data.is_null() {
            return Err(self);
        }
        // SAFETY: `ExtBytesRef.data` is a valid pointer to `ExtBytesWrapper`.
        let ee = unsafe { &*dd.data };
        if ee.kind != core::any::TypeId::of::<T>()
            || ee.ref_count.load(core::sync::atomic::Ordering::Relaxed) != 1
        {
            return Err(self);
        }
        let mut offset = core::mem::size_of::<ExtBytesWrapper>();
        let align_mod = offset % ee.align as usize;
        if align_mod != 0 {
            offset += ee.align as usize - align_mod;
        }
        // SAFETY: `dd.data` is a valid pointer to the container data located at `offset`.
        let t_val = unsafe { dd.data.cast::<u8>().add(offset) };
        // SAFETY: `t_val` should now be cast to `*const T`.
        let t_val = unsafe { &mut *t_val.cast::<T>().cast_mut() };
        // SAFETY: `dd.ptr` is a valid pointer to the slice start.
        let slic = unsafe { core::slice::from_raw_parts(dd.ptr, dd.len) };
        let dat = fun(TakeExtBytesInner {
            data: t_val,
            slice: slic,
        });
        core::mem::drop(self);
        Ok(dat)
    }
}

impl Drop for ExtBytes {
    fn drop(&mut self) {
        debug_assert_eq!(
            self.magic[0], KIND_EXT_BYTES,
            "invalid magic number in ExtBytes"
        );
        debug_assert!(!self.data.is_null(), "null pointer in ExtBytes");

        // SAFETY: `data` is a valid pointer to a boxed `ExtBytesRef`.
        let ext_ref = unsafe { &mut *self.data.cast_mut() };
        debug_assert!(!ext_ref.data.is_null(), "null pointer in ExtBytes");
        // SAFETY: `ext_ref.data` is a valid pointer to a `ExtBytesWrapper`.
        let header = unsafe { &*ext_ref.data };
        let ref_count = header
            .ref_count
            .fetch_sub(1, core::sync::atomic::Ordering::Relaxed);
        if ref_count == 1 {
            if let Some(drop) = header.drop {
                let mut offset = core::mem::size_of::<ExtBytesWrapper>();
                let align_mod = offset % header.align as usize;
                if align_mod != 0 {
                    offset += header.align as usize - align_mod;
                }
                // SAFETY: `ext_ref.data` is a valid pointer to the container data located at `offset`.
                let ptr = unsafe { ext_ref.data.cast::<u8>().add(offset).cast_mut() };
                // SAFETY: `ptr` is a valid pointer to the data, which should be dropped.
                unsafe { drop(ptr.cast()) };
            }
            #[allow(clippy::unwrap_used)]
            let layout =
                core::alloc::Layout::from_size_align(header.alloc, header.align as usize).unwrap();
            // SAFETY: `ext_ref.data` is a valid pointer to an allocated area.
            unsafe { alloc::alloc::dealloc(ext_ref.data.cast::<u8>().cast_mut(), layout) };
        }
        // SAFETY: `ext_ref` is a valid pointer to a boxed `ExtBytesRef`.
        core::mem::drop(unsafe { alloc::boxed::Box::from_raw(ext_ref) });
        self.data = core::ptr::null();
    }
}

impl Clone for ExtBytes {
    fn clone(&self) -> Self {
        debug_assert_eq!(
            self.magic[0], KIND_EXT_BYTES,
            "invalid magic number in ExtBytes"
        );
        debug_assert!(!self.data.is_null(), "null pointer in ExtBytes");

        // SAFETY: `data` is a valid pointer to a boxed `ExtBytesRef`.
        let ext_ref = unsafe { &*self.data };
        debug_assert!(!ext_ref.data.is_null(), "null pointer in ExtBytes");

        // SAFETY: `ext_ref.data` is a valid pointer to a `ExtBytesWrapper`.
        let header = unsafe { &*(ext_ref.data) };
        header
            .ref_count
            .fetch_add(1, core::sync::atomic::Ordering::Relaxed);

        let ret = alloc::boxed::Box::new(ExtBytesRef {
            data: ext_ref.data,
            ptr: ext_ref.ptr,
            len: ext_ref.len,
        });

        Self {
            magic: Self::MAGIC,
            data: alloc::boxed::Box::into_raw(ret),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ByteData;

    #[test]
    /// Check if zero copy works for `Vec<u8>`.
    fn test_bytedata_ext_vec() {
        use alloc::vec::Vec;
        let mut data = Vec::<u8>::with_capacity(64);
        for i in 0..48 {
            data.push(i);
        }
        let data_copy = data.clone();
        let ptr = data.as_slice().as_ptr();
        let mut data = ByteData::from_external(data);
        data.make_sliced(..32);
        assert_eq!(data.len(), 32);
        let data = Vec::<u8>::from(data);
        assert_eq!(data, &data_copy[..32]);
        let check_ptr = data.as_slice().as_ptr();
        assert!(
            core::ptr::addr_eq(ptr, check_ptr),
            "pointers should be equal"
        );
    }
}
