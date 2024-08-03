//! # ExternalBytes
//!
//! External bytes are byte data that is not owned by the `bytedata` crate.
//! This can be used to create `ByteData` instances from byte data that is owned by another crate.

/// A structure describing the operations that can be performed on an external byte data type.
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
pub trait ExternalBytes: core::any::Any + Sized + 'static {
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
    const OPS: ExternalOps<Self> = ExternalOps::new(alloc::vec::Vec::as_slice);
}

impl ExternalBytes for alloc::string::String {
    const OPS: ExternalOps<Self> = ExternalOps::new(alloc::string::String::as_bytes);
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
    pub(crate) const fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.ptr, self.len) }
    }

    #[inline]
    pub(crate) const fn len(&self) -> usize {
        self.len
    }
}

#[repr(C)]
pub(crate) struct ExtBytes {
    magic: [u8; 8],
    data: *const ExtBytesRef,
}

unsafe impl Send for ExtBytes {}
unsafe impl Sync for ExtBytes {}

pub(crate) const KIND_EXT_BYTES: u8 = 0b0000_0011;

impl ExtBytes {
    const MAGIC: [u8; 8] = [KIND_EXT_BYTES, 0, 0, 0, 0, 0, 0, 0];

    pub(crate) fn create<'a, T: ExternalBytes>(ext_bytes: T) -> crate::ByteData<'a> {
        let as_slice = T::OPS.as_slice;
        {
            // Try to use the data as a short chunk
            let sl = as_slice(&ext_bytes);
            let len = sl.len();
            if len <= crate::ByteChunk::LEN {
                return crate::ByteData::from_chunk_slice(sl);
            }
        }

        let align = core::mem::align_of::<T>().max(core::mem::align_of::<ExtBytesWrapper>());
        let mut alloc = core::mem::size_of::<T>() + core::mem::size_of::<ExtBytesWrapper>();
        if alloc % align != 0 {
            alloc += align - (alloc % align);
        }
        let mut offset = core::mem::size_of::<ExtBytesWrapper>();
        if offset % align != 0 {
            offset += align - (offset % align);
        }

        let (data, len, ptr) = unsafe {
            let layout = alloc::alloc::Layout::from_size_align(alloc, align).unwrap();
            let data = alloc::alloc::alloc(layout);
            if data.is_null() {
                alloc::alloc::handle_alloc_error(layout);
            }

            let payload = data.add(offset) as *mut T;
            payload.write(ext_bytes);

            let sl = as_slice(&*payload);
            let len = sl.len();
            let ptr = sl.as_ptr();

            if len <= crate::ByteChunk::LEN {
                let a = crate::ByteData::from_chunk_slice(sl);
                if let Some(drop) = T::OPS.drop {
                    drop(payload);
                }
                alloc::alloc::dealloc(data, layout);
                return a;
            }

            let header = data as *mut ExtBytesWrapper;
            header.write(ExtBytesWrapper {
                drop: core::mem::transmute::<Option<unsafe fn(*mut T)>, Option<unsafe fn(*mut ())>>(
                    T::OPS.drop,
                ),
                alloc,
                ref_count: core::sync::atomic::AtomicU32::new(1),
                align: align as u32,
                kind: core::any::TypeId::of::<T>(),
            });

            (header, len, ptr)
        };

        let data = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(ExtBytesRef { data, ptr, len }));

        crate::ByteData {
            external: core::mem::ManuallyDrop::new(ExtBytes {
                magic: Self::MAGIC,
                data,
            }),
        }
    }

    pub(crate) const fn as_slice(&self) -> &[u8] {
        unsafe { (*self.data).as_slice() }
    }

    pub(crate) const fn len(&self) -> usize {
        unsafe { (*self.data).len() }
    }

    pub(crate) fn make_sliced_range<
        R: core::ops::RangeBounds<usize> + core::slice::SliceIndex<[u8], Output = [u8]>,
    >(
        &mut self,
        range: R,
    ) {
        debug_assert_eq!(self.magic[0], KIND_EXT_BYTES);
        debug_assert!(!self.data.is_null(), "null pointer in ExtBytes");
        let d = unsafe { &mut *(self.data as *mut ExtBytesRef) };
        let len = d.len();
        let start = match range.start_bound() {
            core::ops::Bound::Included(&s) => s,
            core::ops::Bound::Excluded(&s) => s + 1,
            core::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            core::ops::Bound::Included(&e) => e + 1,
            core::ops::Bound::Excluded(&e) => e,
            core::ops::Bound::Unbounded => len,
        };
        if end > len || start > len {
            panic!("index out of bounds");
        }
        if end < start {
            panic!("end < start");
        }
        let len = end - start;
        d.ptr = unsafe { d.ptr.add(start) };
        d.len = len;
    }

    pub(crate) fn sliced_range<
        's,
        R: core::ops::RangeBounds<usize> + core::slice::SliceIndex<[u8], Output = [u8]>,
    >(
        &self,
        range: R,
    ) -> crate::ByteData<'s> {
        debug_assert_eq!(self.magic[0], KIND_EXT_BYTES);
        debug_assert!(!self.data.is_null(), "null pointer in ExtBytes");
        let d = unsafe { &*self.data };
        let len = d.len();
        let start = match range.start_bound() {
            core::ops::Bound::Included(&s) => s,
            core::ops::Bound::Excluded(&s) => s + 1,
            core::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            core::ops::Bound::Included(&e) => e + 1,
            core::ops::Bound::Excluded(&e) => e,
            core::ops::Bound::Unbounded => len,
        };
        if end > len || start > len {
            panic!("index out of bounds");
        }
        if end < start {
            panic!("end < start");
        }
        let len = end - start;
        if len <= crate::byte_chunk::ByteChunk::LEN {
            return crate::ByteData::from_chunk_slice(unsafe {
                core::slice::from_raw_parts(d.ptr.add(start), len)
            });
        }
        let ret = self.clone();
        let d = unsafe { &mut *(ret.data as *mut ExtBytesRef) };
        d.ptr = unsafe { d.ptr.add(start) };
        d.len = len;
        crate::ByteData {
            external: core::mem::ManuallyDrop::new(ret),
        }
    }

    pub(crate) fn with_inner<T: core::any::Any, R, F: FnOnce(&T, &[u8]) -> R>(
        &self,
        f: F,
    ) -> Option<R> {
        debug_assert_eq!(self.magic[0], KIND_EXT_BYTES);
        debug_assert!(!self.data.is_null(), "null pointer in ExtBytes");
        let d = unsafe { &*self.data };
        if d.data.is_null() {
            return None;
        }
        let e = unsafe { &*d.data };
        if e.kind != core::any::TypeId::of::<T>() {
            return None;
        }
        let mut offset = core::mem::size_of::<ExtBytesWrapper>();
        let align_mod = offset % e.align as usize;
        if align_mod != 0 {
            offset += e.align as usize - align_mod;
        }
        let t = unsafe { &*((d.data as *const u8).add(offset) as *const T) };
        Some(f(t, unsafe { core::slice::from_raw_parts(d.ptr, d.len) }))
    }
}

impl Drop for ExtBytes {
    fn drop(&mut self) {
        debug_assert_eq!(self.magic[0], KIND_EXT_BYTES);
        debug_assert!(!self.data.is_null(), "null pointer in ExtBytes");
        unsafe {
            let ext_ref = &mut *(self.data as *mut ExtBytesRef);
            debug_assert!(!ext_ref.data.is_null(), "null pointer in ExtBytes");
            let header = &*(ext_ref.data);
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
                    drop((ext_ref.data as *const u8).add(offset) as *mut ());
                }
                let layout =
                    alloc::alloc::Layout::from_size_align(header.alloc, header.align as usize)
                        .unwrap();
                alloc::alloc::dealloc(ext_ref.data as *mut u8, layout);
            }
            core::mem::drop(alloc::boxed::Box::from_raw(self.data as *mut ExtBytesRef));
            self.data = core::ptr::null();
        }
    }
}

impl Clone for ExtBytes {
    fn clone(&self) -> Self {
        debug_assert_eq!(self.magic[0], KIND_EXT_BYTES);
        debug_assert!(!self.data.is_null(), "null pointer in ExtBytes");
        let a = unsafe {
            let ext_ref = &*self.data;
            debug_assert!(!ext_ref.data.is_null(), "null pointer in ExtBytes");
            let header = &*(ext_ref.data);
            header
                .ref_count
                .fetch_add(1, core::sync::atomic::Ordering::Relaxed);
            alloc::boxed::Box::new(ExtBytesRef {
                data: ext_ref.data,
                ptr: ext_ref.ptr,
                len: ext_ref.len,
            })
        };
        Self {
            magic: Self::MAGIC,
            data: alloc::boxed::Box::into_raw(a),
        }
    }
}
