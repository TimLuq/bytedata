use core::{
    ops::{Deref, Index, RangeBounds},
    slice::SliceIndex,
};

#[cfg(feature = "alloc")]
use alloc::{borrow::Cow, vec::Vec};

#[cfg(feature = "alloc")]
use crate::SharedBytes;

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(clippy::redundant_pub_crate)]
pub(crate) struct ByteSlice<'a> {
    len: u64,
    addr: *const u8,
    _marker: core::marker::PhantomData<&'a u8>,
}

// SAFETY: This is safe because the the data persists for `'a`.
unsafe impl Send for ByteSlice<'_> {}
// SAFETY: This is safe because the the data persists for `'a`.
unsafe impl Sync for ByteSlice<'_> {}

impl<'a> ByteSlice<'a> {
    #[inline]
    const fn new(data: &[u8], is_static: bool) -> Self {
        let mask = if is_static { 0b1000_1111 } else { 0b0000_1111 };
        let len = (((data.len() as u64) << 8) | mask).to_le();
        ByteSlice {
            addr: data.as_ptr(),
            len,
            _marker: core::marker::PhantomData,
        }
    }

    #[inline]
    pub(crate) const fn is_static(&self) -> bool {
        // SAFETY: This is safe because the `is_static` field is the first bit in the struct.
        //         Also the data is non-zero and initialized so this always works.
        let pv = unsafe { core::mem::transmute_copy::<Self, u8>(self) };
        (pv & 0b1000_0000) != 0
    }

    #[inline]
    pub(crate) const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub(crate) const fn len(&self) -> usize {
        let aa = self.len.to_le() >> 8_u16;
        aa as usize
    }

    #[inline]
    pub(crate) const fn as_slice(&self) -> &'a [u8] {
        let len = self.len();
        // SAFETY: This is safe because the data is valid for the length.
        unsafe { core::slice::from_raw_parts(self.addr, len) }
    }

    #[allow(dead_code)]
    #[inline]
    pub(crate) const fn as_static(&self) -> Option<&'static [u8]> {
        if self.is_static() {
            // SAFETY: This is safe because if `is_static` is set, then the data is actually static.
            Some(unsafe { core::mem::transmute::<&[u8], &'static [u8]>(self.as_slice()) })
        } else {
            None
        }
    }
}

impl ByteSlice<'static> {
    #[inline]
    fn make_static(&mut self) {
        let ptr = (self as *mut Self).cast::<u8>();
        // SAFETY: This is safe because the `is_static` field is the first bit in the struct.
        unsafe { *ptr |= 0b1000_0000 };
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(clippy::redundant_pub_crate)]
pub(crate) struct DataKind<T: Copy> {
    pub(crate) kind: u8,
    pub(crate) data: T,
}

impl<T: Copy> DataKind<T> {
    #[inline]
    pub(crate) const fn kind(&self) -> Kind {
        let base_kind = self.kind & 0b0000_1111;
        if base_kind == 0b0000_0111 {
            return Kind::Chunk;
        }
        if base_kind == 0b0000_1111 {
            return Kind::Slice;
        }

        #[cfg(feature = "alloc")]
        if base_kind == crate::external::KIND_EXT_BYTES {
            return Kind::External;
        }

        #[cfg(feature = "alloc")]
        if base_kind.trailing_zeros() >= 2 {
            return Kind::Shared;
        }

        #[cfg(not(feature = "alloc"))]
        if base_kind.trailing_zeros() >= 2 || base_kind == crate::external::KIND_EXT_BYTES {
            panic!("alloc feature is not enabled, so no path should trigger this");
        }

        panic!("no path should trigger this");
    }
}

#[allow(clippy::redundant_pub_crate)]
pub(crate) enum Kind {
    Chunk,
    Slice,
    #[cfg(feature = "alloc")]
    Shared,
    #[cfg(feature = "alloc")]
    External,
}

const KIND_CHUNK_MASK: u8 = 0b0000_0111;

type WrappedChunk = DataKind<crate::byte_chunk::ByteChunk>;

/// A container of bytes that can be either static, borrowed, or shared.
#[repr(C)]
pub union ByteData<'a> {
    /// A chunk of bytes that is 14 bytes or less.
    pub(crate) chunk: WrappedChunk,
    /// A byte slice.
    pub(crate) slice: ByteSlice<'a>,
    #[cfg(feature = "alloc")]
    /// A shared byte slice.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub(crate) shared: core::mem::ManuallyDrop<SharedBytes>,
    #[cfg(feature = "alloc")]
    /// A shared byte slice.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub(crate) external: core::mem::ManuallyDrop<crate::external::ExtBytes>,
}

impl Clone for ByteData<'_> {
    #[allow(clippy::missing_inline_in_public_items)]
    fn clone(&self) -> Self {
        match self.kind() {
            Kind::Chunk => Self {
                // SAFETY: Chunk state has been checked, and it is `Copy`.
                chunk: unsafe { self.chunk },
            },
            Kind::Slice => Self {
                // SAFETY: Slice state has been checked, and it is `Copy`.
                slice: unsafe { self.slice },
            },
            #[cfg(feature = "alloc")]
            Kind::Shared => Self {
                // SAFETY: Shared state has been checked.
                shared: core::mem::ManuallyDrop::new(unsafe { SharedBytes::clone(&self.shared) }),
            },
            #[cfg(feature = "alloc")]
            Kind::External => Self {
                // SAFETY: External state has been checked.
                external: core::mem::ManuallyDrop::new(unsafe {
                    crate::external::ExtBytes::clone(&self.external)
                }),
            },
        }
    }
}

const fn empty_chunk() -> WrappedChunk {
    WrappedChunk {
        kind: KIND_CHUNK_MASK,
        // SAFETY: a chunk filled with zeros is always valid with a len of 0.
        data: unsafe { core::mem::zeroed() },
    }
}

impl Drop for ByteData<'_> {
    #[allow(clippy::missing_inline_in_public_items)]
    fn drop(&mut self) {
        match self.kind() {
            Kind::Chunk | Kind::Slice => (),
            #[cfg(feature = "alloc")]
            // SAFETY: Shared state has been checked.
            Kind::Shared => unsafe {
                core::ptr::drop_in_place(&mut self.shared);
                self.chunk = empty_chunk();
            },
            #[cfg(feature = "alloc")]
            // SAFETY: External state has been checked.
            Kind::External => unsafe {
                core::ptr::drop_in_place(&mut self.external);
                self.chunk = empty_chunk();
            },
        }
    }
}

impl<'a> ByteData<'a> {
    /// Returns an empty `ByteData`.
    #[inline]
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            chunk: empty_chunk(),
        }
    }

    #[inline]
    #[must_use]
    pub(crate) const fn kind(&self) -> Kind {
        // SAFETY: This is safe because the `kind` field is always the first field in the union.
        unsafe { self.chunk.kind() }
    }

    /// Creates a `ByteData` from a slice of bytes.
    #[inline]
    #[must_use]
    pub const fn from_static(dat: &'static [u8]) -> Self {
        if dat.len() <= crate::byte_chunk::ByteChunk::LEN {
            Self {
                chunk: WrappedChunk {
                    kind: KIND_CHUNK_MASK,
                    data: crate::byte_chunk::ByteChunk::from_slice(dat),
                },
            }
        } else {
            Self {
                slice: ByteSlice::new(dat, true),
            }
        }
    }

    /// Creates a `ByteData` from a slice of bytes. The slice must be 14 bytes or less. If the slice is larger, this will panic.
    #[inline]
    #[must_use]
    pub const fn from_chunk_slice(dat: &[u8]) -> Self {
        if dat.is_empty() {
            Self::empty()
        } else {
            Self {
                chunk: WrappedChunk {
                    kind: KIND_CHUNK_MASK,
                    data: crate::byte_chunk::ByteChunk::from_slice(dat),
                },
            }
        }
    }

    /// Creates a `ByteData` from a single byte.
    #[inline]
    #[must_use]
    pub const fn from_byte(b0: u8) -> Self {
        Self {
            chunk: WrappedChunk {
                kind: KIND_CHUNK_MASK,
                data: crate::byte_chunk::ByteChunk::from_byte(b0),
            },
        }
    }

    /// Creates a `ByteData` from an array of bytes. The array must be 14 bytes or less. If the array is larger, this will panic.
    #[inline]
    #[must_use]
    pub const fn from_chunk<const L: usize>(dat: &[u8; L]) -> Self {
        if L == 0 {
            Self::empty()
        } else {
            Self {
                chunk: WrappedChunk {
                    kind: KIND_CHUNK_MASK,
                    data: crate::byte_chunk::ByteChunk::from_array(dat),
                },
            }
        }
    }

    /// Creates a `ByteData` from a borrowed slice of bytes.
    #[inline]
    #[must_use]
    pub const fn from_borrowed(dat: &'a [u8]) -> Self {
        if dat.is_empty() {
            Self {
                chunk: empty_chunk(),
            }
        } else if dat.len() <= crate::byte_chunk::ByteChunk::LEN {
            Self {
                chunk: WrappedChunk {
                    kind: KIND_CHUNK_MASK,
                    data: crate::byte_chunk::ByteChunk::from_slice(dat),
                },
            }
        } else {
            Self {
                slice: ByteSlice::new(dat, false),
            }
        }
    }

    #[cfg(feature = "alloc")]
    /// Creates a `ByteData` from a `SharedBytes`.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[inline]
    #[must_use]
    pub const fn from_shared(dat: SharedBytes) -> Self {
        Self {
            shared: core::mem::ManuallyDrop::new(dat),
        }
    }

    #[cfg(feature = "alloc")]
    /// Creates a `ByteData` from a `Vec<u8>` using zero copy.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[inline]
    #[must_use]
    pub fn from_owned(dat: Vec<u8>) -> Self {
        if dat.is_empty() {
            return Self::empty();
        }
        if dat.len() <= crate::byte_chunk::ByteChunk::LEN {
            return Self {
                chunk: WrappedChunk {
                    kind: KIND_CHUNK_MASK,
                    data: crate::byte_chunk::ByteChunk::from_slice(dat.as_slice()),
                },
            };
        }
        crate::external::ExtBytes::create(dat)
    }

    #[cfg(feature = "alloc")]
    /// Creates a `ByteData` from an externally kept byte sequence.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[inline]
    pub fn from_external<E: crate::external::IntoExternalBytes>(dat: E) -> Self {
        crate::external::ExtBytes::create(dat)
    }

    #[cfg(feature = "alloc")]
    /// Creates a `ByteData` from a `Cow<'_, [u8]>`.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[must_use]
    #[inline]
    pub fn from_cow(dat: Cow<'a, [u8]>) -> Self {
        match dat {
            Cow::Borrowed(borr) => Self::from_borrowed(borr),
            Cow::Owned(ow) => Self::from_owned(ow),
        }
    }

    #[cfg(feature = "alloc")]
    /// Creates a `ByteData` from a `Cow<'static, [u8]>`.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[must_use]
    #[inline]
    pub fn from_cow_static(dat: Cow<'static, [u8]>) -> Self {
        match dat {
            Cow::Borrowed(borr) => Self::from_static(borr),
            Cow::Owned(ow) => Self::from_owned(ow),
        }
    }

    /// Returns the underlying byte slice.
    #[must_use]
    #[allow(clippy::missing_inline_in_public_items)]
    pub const fn as_slice(&self) -> &[u8] {
        match self.kind() {
            // SAFETY: Chunk state has been checked.
            Kind::Chunk => unsafe { self.chunk.data.as_slice() },
            // SAFETY: Slice state has been checked.
            Kind::Slice => unsafe { self.slice.as_slice() },
            #[cfg(feature = "alloc")]
            // SAFETY: Shared state has been checked.
            Kind::Shared => unsafe {
                core::mem::transmute::<&core::mem::ManuallyDrop<SharedBytes>, &SharedBytes>(
                    &self.shared,
                )
            }
            .as_slice(),
            #[cfg(feature = "alloc")]
            // SAFETY: External state has been checked.
            Kind::External => unsafe {
                core::mem::transmute::<
                    &core::mem::ManuallyDrop<crate::external::ExtBytes>,
                    &crate::external::ExtBytes,
                >(&self.external)
            }
            .as_slice(),
        }
    }

    /// Returns the length of the underlying byte slice.
    #[allow(clippy::missing_inline_in_public_items)]
    #[must_use]
    pub const fn len(&self) -> usize {
        match self.kind() {
            // SAFETY: Chunk state has been checked.
            Kind::Chunk => unsafe { self.chunk.data.len() },
            // SAFETY: Slice state has been checked.
            Kind::Slice => unsafe { self.slice.len() },
            #[cfg(feature = "alloc")]
            // SAFETY: Shared state has been checked.
            Kind::Shared => unsafe {
                core::mem::transmute::<&core::mem::ManuallyDrop<SharedBytes>, &SharedBytes>(
                    &self.shared,
                )
            }
            .len(),
            #[cfg(feature = "alloc")]
            // SAFETY: External state has been checked.
            Kind::External => unsafe {
                core::mem::transmute::<
                    &core::mem::ManuallyDrop<crate::external::ExtBytes>,
                    &crate::external::ExtBytes,
                >(&self.external)
            }
            .len(),
        }
    }

    /// Returns `true` if the underlying byte slice is empty.
    #[allow(clippy::missing_inline_in_public_items)]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        match self.kind() {
            // SAFETY: Chunk state has been checked.
            Kind::Chunk => unsafe { self.chunk.data.is_empty() },
            // SAFETY: Slice state has been checked.
            Kind::Slice => unsafe { self.slice.is_empty() },
            #[cfg(feature = "alloc")]
            // SAFETY: Shared state has been checked.
            Kind::Shared => unsafe {
                core::mem::transmute::<&core::mem::ManuallyDrop<SharedBytes>, &SharedBytes>(
                    &self.shared,
                )
            }
            .is_empty(),
            #[cfg(feature = "alloc")]
            Kind::External => false,
        }
    }

    /// Check if the underlying byte slice is equal to another. This can be used in a `const` context.
    #[inline]
    #[must_use]
    pub const fn eq_const(&self, other: &ByteData<'_>) -> bool {
        crate::const_eq(self.as_slice(), other.as_slice())
    }

    /// Check if the underlying byte slice is equal to another. This can be used in a `const` context.
    #[inline]
    #[must_use]
    pub const fn eq_slice(&self, other: &[u8]) -> bool {
        crate::const_eq(self.as_slice(), other)
    }

    /// Check if the ending of a `SharedBytes` matches the given bytes.
    #[inline]
    #[must_use]
    pub const fn ends_with(&self, needle: &[u8]) -> bool {
        crate::const_ends_with(self.as_slice(), needle)
    }

    /// Check if the beginning of a `SharedBytes` matches the given bytes.
    #[inline]
    #[must_use]
    pub const fn starts_with(&self, needle: &[u8]) -> bool {
        crate::const_starts_with(self.as_slice(), needle)
    }

    /// Returns a `ByteData` with the given range of bytes.
    #[allow(clippy::missing_inline_in_public_items)]
    #[must_use]
    pub fn sliced<R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>>(
        &self,
        range: R,
    ) -> Self {
        match self.kind() {
            // SAFETY: Chunk state has been checked.
            Kind::Chunk => Self::from_chunk_slice(unsafe { &self.chunk.data.as_slice()[range] }),
            Kind::Slice => {
                // SAFETY: Slice state has been checked.
                let aa = unsafe { &self.slice };
                let bb = &aa.as_slice()[range];
                if bb.len() <= crate::byte_chunk::ByteChunk::LEN {
                    Self::from_chunk_slice(bb)
                } else {
                    Self {
                        slice: ByteSlice::new(bb, aa.is_static()),
                    }
                }
            }
            #[cfg(feature = "alloc")]
            Kind::Shared => {
                // SAFETY: Shared state has been checked.
                let dat = unsafe { &self.shared };
                // SAFETY: safe dereference because the data is valid on the line above. Why isn't there a safe const `core::mem::ManuallyDrop::as_ref`?
                let dat = unsafe {
                    &*(dat as *const core::mem::ManuallyDrop<SharedBytes>).cast::<SharedBytes>()
                };
                let dat = dat.sliced_range(range);
                if dat.len() <= crate::byte_chunk::ByteChunk::LEN {
                    Self::from_chunk_slice(dat.as_slice())
                } else {
                    Self::from_shared(dat)
                }
            }
            #[cfg(feature = "alloc")]
            Kind::External => {
                // SAFETY: External state has been checked.
                let dat = unsafe { &self.external };
                // SAFETY: safe dereference because the data is valid on the line above. Why isn't there a safe const `core::mem::ManuallyDrop::as_ref`?
                let dat = unsafe {
                    &*(dat as *const core::mem::ManuallyDrop<crate::external::ExtBytes>)
                        .cast::<crate::external::ExtBytes>()
                };
                dat.sliced_range(range)
            }
        }
    }

    /// Transform the range of bytes this `ByteData` represents.
    #[allow(clippy::missing_inline_in_public_items)]
    #[must_use]
    pub fn into_sliced<R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>>(
        mut self,
        range: R,
    ) -> Self {
        match self.kind() {
            Kind::Chunk => {
                // SAFETY: Chunk state has been checked.
                unsafe { self.chunk.data.make_sliced(range) };
                self
            }
            Kind::Slice => {
                // SAFETY: Slice state has been checked.
                let aa = unsafe { &mut self.slice };
                *aa = ByteSlice::new(&aa.as_slice()[range], aa.is_static());
                self
            }
            #[cfg(feature = "alloc")]
            Kind::Shared => {
                // SAFETY: Shared state has been checked.
                let dat = unsafe { &mut self.shared };
                // SAFETY: safe dereference because the data is valid on the line above. Why isn't there a safe const `core::mem::ManuallyDrop::as_mut`?
                let dat = unsafe {
                    &mut *(dat as *mut core::mem::ManuallyDrop<SharedBytes>).cast::<SharedBytes>()
                };
                dat.make_sliced_range(range);
                if dat.len() <= crate::byte_chunk::ByteChunk::LEN {
                    let ret = Self::from_chunk_slice(dat.as_slice());
                    // SAFETY: safe drop because the data is valid on the line above.
                    unsafe { core::ptr::drop_in_place(dat) };
                    core::mem::forget(self);
                    ret
                } else {
                    self
                }
            }
            #[cfg(feature = "alloc")]
            Kind::External => {
                // SAFETY: External state has been checked.
                let dat = unsafe { &mut self.external };
                // SAFETY: safe dereference because the data is valid on the line above. Why isn't there a safe const `core::mem::ManuallyDrop::as_mut`?
                let dat = unsafe {
                    &mut *(dat as *mut core::mem::ManuallyDrop<crate::external::ExtBytes>)
                        .cast::<crate::external::ExtBytes>()
                };
                dat.make_sliced_range(range);
                if dat.len() <= crate::byte_chunk::ByteChunk::LEN {
                    let ret = Self::from_chunk_slice(dat.as_slice());
                    // SAFETY: safe drop because the data is valid on the line above.
                    unsafe { core::ptr::drop_in_place(dat) };
                    core::mem::forget(self);
                    ret
                } else {
                    self
                }
            }
        }
    }

    /// Transform the range of bytes this `ByteData` represents.
    #[allow(clippy::missing_inline_in_public_items)]
    pub fn make_sliced<R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>>(
        &'_ mut self,
        range: R,
    ) {
        match self.kind() {
            Kind::Chunk => {
                // SAFETY: Chunk state has been checked.
                unsafe { self.chunk.data.make_sliced(range) };
            }
            Kind::Slice => {
                // SAFETY: Slice state has been checked.
                let aa = unsafe { &mut self.slice };
                *aa = ByteSlice::new(&aa.as_slice()[range], aa.is_static());
            }
            #[cfg(feature = "alloc")]
            Kind::Shared => {
                // SAFETY: Shared state has been checked.
                let dat = unsafe { &mut self.shared };
                // SAFETY: safe dereference because the data is valid on the line above. Why isn't there a safe const `core::mem::ManuallyDrop::as_mut`?
                let dat = unsafe {
                    &mut *(dat as *mut core::mem::ManuallyDrop<SharedBytes>).cast::<SharedBytes>()
                };
                dat.make_sliced_range(range);
                if dat.len() > crate::byte_chunk::ByteChunk::LEN {
                    return;
                }
                let ret = crate::ByteChunk::from_slice(dat.as_slice());
                // SAFETY: safe drop because the data is valid on the lines above.
                unsafe {
                    core::ptr::drop_in_place(dat);
                };
                self.chunk = DataKind {
                    kind: KIND_CHUNK_MASK,
                    data: ret,
                };
            }
            #[cfg(feature = "alloc")]
            Kind::External => {
                // SAFETY: External state has been checked.
                let dat = unsafe { &mut self.external };
                // SAFETY: safe dereference because the data is valid on the line above. Why isn't there a safe const `core::mem::ManuallyDrop::as_mut`?
                let dat = unsafe {
                    &mut *(dat as *mut core::mem::ManuallyDrop<crate::external::ExtBytes>)
                        .cast::<crate::external::ExtBytes>()
                };
                dat.make_sliced_range(range);
                if dat.len() > crate::byte_chunk::ByteChunk::LEN {
                    return;
                }
                let ret = crate::ByteChunk::from_slice(dat.as_slice());
                // SAFETY: safe drop because the data is valid on the lines above.
                unsafe {
                    core::ptr::drop_in_place(dat);
                };
                self.chunk = DataKind {
                    kind: KIND_CHUNK_MASK,
                    data: ret,
                };
            }
        }
    }

    #[cfg(feature = "alloc")]
    /// Transform any borrowed data into shared data. This is useful when you wish to change the lifetime of the data.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[inline]
    #[must_use]
    pub fn into_shared<'s>(mut self) -> ByteData<'s> {
        match self.kind() {
            // SAFETY: these states are owned, so they can be transformed to any lifetime.
            Kind::Chunk | Kind::Shared | Kind::External => unsafe {
                core::mem::transmute::<ByteData<'a>, ByteData<'s>>(self)
            },
            Kind::Slice => {
                // SAFETY: Slice state has been checked.
                let aa = unsafe { &self.slice };
                if aa.is_static() {
                    // SAFETY: as the data is static, it can be transformed to any lifetime.
                    unsafe { core::mem::transmute::<ByteData<'a>, ByteData<'s>>(self) }
                } else if aa.len() <= crate::byte_chunk::ByteChunk::LEN {
                    let ret = crate::byte_chunk::ByteChunk::from_slice(aa.as_slice());
                    core::mem::forget(self);
                    ByteData {
                        chunk: DataKind {
                            kind: KIND_CHUNK_MASK,
                            data: ret,
                        },
                    }
                } else {
                    let ret = SharedBytes::from_slice(aa.as_slice());
                    self.shared = core::mem::ManuallyDrop::new(ret);
                    // SAFETY: the Shared state is owned, so it can be transformed to any lifetime.
                    unsafe { core::mem::transmute::<ByteData<'a>, ByteData<'s>>(self) }
                }
            }
        }
    }

    #[cfg(feature = "alloc")]
    /// Transform any borrowed data into shared data of a specific range. This is useful when you wish to change the lifetime of the data.
    ///
    /// This is essentially the same as `into_shared().into_sliced(range)`, but it is more efficient.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[allow(clippy::missing_inline_in_public_items)]
    #[must_use]
    pub fn into_shared_range<'s, R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>>(
        mut self,
        range: R,
    ) -> ByteData<'s> {
        match self.kind() {
            Kind::Chunk => {
                // SAFETY: Chunk state has been checked.
                unsafe {
                    self.chunk.data.make_sliced(range);
                };
                // SAFETY: the Chunk state is owned, so it can be transformed to any lifetime.
                unsafe { core::mem::transmute::<ByteData<'a>, ByteData<'s>>(self) }
            }
            Kind::Shared => {
                // SAFETY: Shared state has been checked.
                unsafe { (*self.shared).make_sliced_range(range) };
                // SAFETY: the Shared state is owned, so it can be transformed to any lifetime.
                unsafe { core::mem::transmute::<ByteData<'a>, ByteData<'s>>(self) }
            }
            Kind::External => {
                // SAFETY: External state has been checked.
                unsafe {
                    (*self.external).make_sliced_range(range);
                };
                // SAFETY: the External state is owned, so it can be transformed to any lifetime.
                unsafe { core::mem::transmute::<ByteData<'a>, ByteData<'s>>(self) }
            }
            Kind::Slice => {
                // SAFETY: Slice state has been checked.
                let aa = unsafe { &self.slice };
                let data = &aa.as_slice()[range];
                if data.len() <= crate::byte_chunk::ByteChunk::LEN {
                    let data = crate::byte_chunk::ByteChunk::from_slice(data);
                    core::mem::forget(self);
                    return ByteData {
                        chunk: DataKind {
                            kind: KIND_CHUNK_MASK,
                            data,
                        },
                    };
                }
                if aa.is_static() {
                    core::mem::forget(self);
                    return ByteData {
                        slice: ByteSlice::new(data, true),
                    };
                }
                let ret = SharedBytes::from_slice(data);
                self.shared = core::mem::ManuallyDrop::new(ret);
                // SAFETY: the Shared state is owned, so it can be transformed to any lifetime.
                unsafe { core::mem::transmute::<ByteData<'a>, ByteData<'s>>(self) }
            }
        }
    }

    /// Split the `ByteData` at the given position.
    #[inline]
    #[allow(clippy::return_self_not_must_use)]
    pub fn take_bytes(&mut self, position: usize) -> Self {
        if position == 0 {
            return ByteData::empty();
        }
        if position == self.len() {
            return core::mem::replace(self, ByteData::empty());
        }
        let aa = self.sliced(0..position);
        self.make_sliced(position..);
        aa
    }

    /// Consume the `ByteData` until the byte condition is triggered.
    #[inline]
    #[allow(clippy::return_self_not_must_use)]
    pub fn take_while<F: FnMut(u8) -> bool>(&mut self, mut fun: F) -> Self {
        let mut i = 0;
        let aa = self.as_slice();
        while i < aa.len() && fun(aa[i]) {
            i += 1;
        }
        self.take_bytes(i)
    }

    /// Split the `ByteData` at the given position.
    #[inline]
    #[must_use]
    pub fn split_at(mut self, position: usize) -> (Self, Self) {
        let aa = self.take_bytes(position);
        (aa, self)
    }

    /// Split the `ByteData` at the first occurrence of the given byte sequence.
    ///
    /// # Errors
    ///
    /// If the byte sequence is not found, the original `ByteData` is returned.
    #[inline]
    pub fn split_once_on(self, needle: &[u8]) -> Result<(Self, Self), Self> {
        let aa = match crate::const_split_once_bytes(self.as_slice(), needle) {
            Some((aa, _)) => aa.len(),
            None => return Err(self),
        };
        Ok(self.split_at(aa))
    }

    /// Split the `ByteData` at the first occurrence of the given byte sequence.
    #[inline]
    pub fn split_on<'b>(self, needle: &'b [u8]) -> impl Iterator<Item = ByteData<'a>> + Send + 'b
    where
        'a: 'b,
    {
        struct It<'a, 'b>(ByteData<'a>, &'b [u8], bool);
        impl<'a> Iterator for It<'a, '_> {
            type Item = ByteData<'a>;

            fn next(&mut self) -> Option<Self::Item> {
                if self.0.is_empty() {
                    return None;
                }
                let aa = if let Some((aa, _)) =
                    crate::const_split_once_bytes(self.0.as_slice(), self.1)
                {
                    aa.len()
                } else {
                    return Some(core::mem::replace(&mut self.0, ByteData::empty()));
                };
                if aa == 0 && self.2 {
                    self.2 = false;
                    return Some(ByteData::empty());
                }
                self.2 = false;
                let aa = self.0.take_bytes(aa);
                Some(aa)
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                if self.0.is_empty() {
                    (0, Some(0))
                } else if self.0.len() < self.1.len() {
                    (1, Some(1))
                } else if self.0.len() < self.1.len() * 2 {
                    (1, Some(2))
                } else {
                    (1, None)
                }
            }
        }
        It(self, needle, true)
    }
}

impl ByteData<'static> {
    /// Forces any borrowed slice to be marked as static.
    #[inline]
    #[must_use]
    pub fn statically_borrowed(mut self) -> Self {
        if matches!(self.kind(), Kind::Slice) {
            // SAFETY: Slice state has been checked.
            unsafe { self.slice.make_static() };
        }
        self
    }
}

impl AsRef<[u8]> for ByteData<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl Deref for ByteData<'_> {
    type Target = [u8];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<'a> From<&'a [u8]> for ByteData<'a> {
    #[inline]
    fn from(dat: &'a [u8]) -> Self {
        Self::from_borrowed(dat)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl From<SharedBytes> for ByteData<'_> {
    #[inline]
    fn from(dat: SharedBytes) -> Self {
        if dat.len() <= crate::byte_chunk::ByteChunk::LEN {
            Self::from_chunk_slice(&dat)
        } else {
            Self::from_shared(dat)
        }
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl From<Vec<u8>> for ByteData<'_> {
    #[inline]
    fn from(dat: Vec<u8>) -> Self {
        let len = dat.len();
        if len <= crate::byte_chunk::ByteChunk::LEN {
            Self::from_chunk_slice(&dat)
        } else if len < 32 {
            Self::from_shared(dat.into())
        } else {
            Self::from_external(dat)
        }
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<'a> From<ByteData<'a>> for Vec<u8> {
    #[allow(clippy::missing_inline_in_public_items)]
    fn from(dat: ByteData<'a>) -> Self {
        if !matches!(dat.kind(), Kind::External) {
            return dat.as_slice().to_vec();
        }
        // SAFETY: External state has been checked.
        let dat = unsafe { core::mem::transmute::<ByteData<'a>, crate::external::ExtBytes>(dat) };

        let res = dat.take_inner::<Self, Self, _>(|inner| {
            let (off, len) = inner.with_slice_ref(|vec, slic| {
                // SAFETY: the slice should be a valid subslice.
                let offset = unsafe { slic.as_ptr().byte_offset_from(vec.as_slice().as_ptr()) };
                #[allow(clippy::cast_sign_loss)]
                let offset = offset as usize;
                let len = slic.len();
                debug_assert!(offset <= vec.len(), "ByteData::into_vec: offset out of bounds");
                debug_assert!(offset + len <= vec.len(), "ByteData::into_vec: len out of bounds");
                (offset, len)
            });
            let inner = inner.into_inner();
            inner.truncate(len + off);
            let mut inner = core::mem::take(inner);
            if off != 0 {
                core::mem::drop(inner.drain(0..off));
            }
            inner
        });
        match res {
            Ok(ok) => ok,
            Err(err) => err.as_slice().to_vec(),
        }
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl From<alloc::string::String> for ByteData<'_> {
    #[inline]
    fn from(dat: alloc::string::String) -> Self {
        Self::from_shared(dat.into())
    }
}

impl Index<usize> for ByteData<'_> {
    type Output = u8;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        let sl = self.as_slice();
        assert!(index < sl.len(), "ByteData::index: index out of bounds");
        // SAFETY: the index has been checked.
        let ptr = unsafe { sl.as_ptr().add(index) };
        // SAFETY: the index has been checked.
        unsafe { &*ptr }
    }
}

impl<'b> PartialEq<ByteData<'b>> for ByteData<'_> {
    #[inline]
    fn eq(&self, other: &ByteData<'b>) -> bool {
        self.as_slice().eq(other.as_slice())
    }
}

impl PartialEq<[u8]> for ByteData<'_> {
    #[inline]
    fn eq(&self, other: &[u8]) -> bool {
        self.as_slice().eq(other)
    }
}

impl<'b> PartialEq<&'b [u8]> for ByteData<'_> {
    #[inline]
    fn eq(&self, other: &&'b [u8]) -> bool {
        self.as_slice().eq(*other)
    }
}

impl PartialEq<ByteData<'_>> for [u8] {
    #[inline]
    fn eq(&self, other: &ByteData<'_>) -> bool {
        self.eq(other.as_slice())
    }
}

impl<'a> PartialEq<ByteData<'a>> for &'_ [u8] {
    #[inline]
    fn eq(&self, other: &ByteData<'a>) -> bool {
        (*self).eq(other.as_slice())
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl PartialEq<Vec<u8>> for ByteData<'_> {
    #[inline]
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.as_slice().eq(other)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl PartialEq<ByteData<'_>> for Vec<u8> {
    #[inline]
    fn eq(&self, other: &ByteData<'_>) -> bool {
        self.eq(other.as_slice())
    }
}

impl Eq for ByteData<'_> {}

impl core::hash::Hash for ByteData<'_> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}

impl<'b> PartialOrd<ByteData<'b>> for ByteData<'_> {
    #[inline]
    fn partial_cmp(&self, other: &ByteData<'b>) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl PartialOrd<[u8]> for ByteData<'_> {
    #[inline]
    fn partial_cmp(&self, other: &[u8]) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(other)
    }
}

impl PartialOrd<ByteData<'_>> for [u8] {
    #[inline]
    fn partial_cmp(&self, other: &ByteData<'_>) -> Option<core::cmp::Ordering> {
        self.partial_cmp(other.as_slice())
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl PartialOrd<Vec<u8>> for ByteData<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Vec<u8>) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(AsRef::<[u8]>::as_ref(other))
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl PartialOrd<ByteData<'_>> for Vec<u8> {
    #[inline]
    fn partial_cmp(&self, other: &ByteData<'_>) -> Option<core::cmp::Ordering> {
        AsRef::<[u8]>::as_ref(self).partial_cmp(other.as_slice())
    }
}

impl Ord for ByteData<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl core::fmt::Debug for ByteData<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&crate::ByteStringRender::from_slice(self.as_slice()), f)
    }
}

impl core::fmt::LowerHex for ByteData<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::byte_string_render::lower_hex_slice(self.as_slice(), f)
    }
}

impl core::fmt::UpperHex for ByteData<'_> {
    #[inline]
    #[allow(clippy::min_ident_chars)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::byte_string_render::upper_hex_slice(self.as_slice(), f)
    }
}

impl Iterator for ByteData<'_> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let sl = self.as_slice();
        if sl.is_empty() {
            return None;
        }
        // SAFETY: the slice is not empty.
        let ret = unsafe { *sl.as_ptr() };
        self.make_sliced(1..);
        Some(ret)
    }
}

impl core::borrow::Borrow<[u8]> for ByteData<'_> {
    #[inline]
    fn borrow(&self) -> &[u8] {
        self.as_slice()
    }
}

impl Default for ByteData<'_> {
    #[inline]
    fn default() -> Self {
        ByteData::empty()
    }
}
