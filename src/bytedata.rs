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
pub(crate) struct ByteSlice<'a> {
    len: u64,
    addr: *const u8,
    _marker: core::marker::PhantomData<&'a u8>,
}

unsafe impl<'a> Send for ByteSlice<'a> {}
unsafe impl<'a> Sync for ByteSlice<'a> {}

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
        let p = unsafe { core::mem::transmute_copy::<Self, u8>(self) };
        (p & 0b1000_0000) != 0
    }

    #[inline]
    pub(crate) const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub(crate) const fn len(&self) -> usize {
        let a = self.len.to_le() >> 8;
        a as usize
    }

    #[inline]
    pub(crate) const fn as_slice(&self) -> &'a [u8] {
        let len = self.len();
        unsafe { core::slice::from_raw_parts(self.addr, len) }
    }

    #[allow(dead_code)]
    #[inline]
    pub(crate) const fn as_static(&self) -> Option<&'static [u8]> {
        if self.is_static() {
            Some(unsafe { core::mem::transmute::<&[u8], &'static [u8]>(self.as_slice()) })
        } else {
            None
        }
    }
}

impl ByteSlice<'static> {
    #[inline]
    fn make_static(&mut self) {
        let p = self as *mut Self as *mut u8;
        unsafe { *p |= 0b1000_0000 };
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
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
        if (base_kind & 0b0000_0011) == 0b0000_0000 {
            return Kind::Shared;
        }
        #[cfg(not(feature = "alloc"))]
        if (base_kind & 0b0000_0011) == 0b0000_0000 {
            panic!("alloc feature is not enabled, so no path should trigger this");
        }

        panic!("no path should trigger this");
    }
}

pub(crate) enum Kind {
    Chunk,
    Slice,
    #[cfg(feature = "alloc")]
    Shared,
}

const KIND_CHUNK_MASK: u8 = 0b0000_0111;

type WrappedChunk = DataKind<crate::byte_chunk::ByteChunk>;

/// A container of bytes that can be either static, borrowed, or shared.
pub union ByteData<'a> {
    /// A chunk of bytes that is 14 bytes or less.
    pub(crate) chunk: WrappedChunk,
    /// A byte slice.
    pub(crate) slice: ByteSlice<'a>,
    #[cfg(feature = "alloc")]
    /// A shared byte slice.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub(crate) shared: core::mem::ManuallyDrop<SharedBytes>,
}

impl<'a> Clone for ByteData<'a> {
    fn clone(&self) -> Self {
        match unsafe { self.chunk.kind() } {
            Kind::Chunk => Self {
                chunk: unsafe { self.chunk },
            },
            Kind::Slice => Self {
                slice: unsafe { self.slice },
            },
            #[cfg(feature = "alloc")]
            Kind::Shared => Self {
                shared: core::mem::ManuallyDrop::new(unsafe { SharedBytes::clone(&self.shared) }),
            },
        }
    }
}

const fn empty_chunk() -> WrappedChunk {
    WrappedChunk {
        kind: KIND_CHUNK_MASK,
        data: unsafe { core::mem::zeroed() },
    }
}

impl<'a> Drop for ByteData<'a> {
    fn drop(&mut self) {
        match unsafe { self.chunk.kind() } {
            Kind::Chunk | Kind::Slice => (),
            #[cfg(feature = "alloc")]
            Kind::Shared => unsafe {
                core::ptr::drop_in_place(&mut self.shared);
                self.chunk = empty_chunk();
            },
        }
    }
}

impl<'a> ByteData<'a> {
    /// Returns an empty `ByteData`.
    #[inline]
    pub const fn empty() -> Self {
        unsafe { core::mem::MaybeUninit::zeroed().assume_init() }
    }

    /// Creates a `ByteData` from a slice of bytes.
    #[inline]
    pub const fn from_static(dat: &'static [u8]) -> Self {
        Self {
            slice: ByteSlice::new(dat, true),
        }
    }

    /// Creates a `ByteData` from a slice of bytes. The slice must be 14 bytes or less. If the slice is larger, this will panic.
    #[inline]
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
    pub const fn from_borrowed(dat: &'a [u8]) -> Self {
        if dat.is_empty() {
            Self {
                chunk: empty_chunk(),
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
    pub const fn from_shared(dat: SharedBytes) -> Self {
        Self {
            shared: core::mem::ManuallyDrop::new(dat),
        }
    }

    #[cfg(feature = "alloc")]
    /// Creates a `ByteData` from a `Vec<u8>`.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[inline]
    pub fn from_owned(dat: Vec<u8>) -> Self {
        if dat.is_empty() {
            return Self::empty();
        }
        if dat.len() <= 14 {
            return Self {
                chunk: WrappedChunk {
                    kind: KIND_CHUNK_MASK,
                    data: crate::byte_chunk::ByteChunk::from_slice(dat.as_slice()),
                },
            };
        }
        Self {
            shared: core::mem::ManuallyDrop::new(dat.into()),
        }
    }

    #[cfg(feature = "alloc")]
    /// Creates a `ByteData` from a `Cow<'_, [u8]>`.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn from_cow(dat: Cow<'a, [u8]>) -> Self {
        match dat {
            Cow::Borrowed(b) => Self::from_borrowed(b),
            Cow::Owned(o) => Self::from_owned(o),
        }
    }

    #[cfg(feature = "alloc")]
    /// Creates a `ByteData` from a `Cow<'static, [u8]>`.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn from_cow_static(dat: Cow<'static, [u8]>) -> Self {
        match dat {
            Cow::Borrowed(b) => Self::from_static(b),
            Cow::Owned(o) => Self::from_owned(o),
        }
    }

    /// Returns the underlying byte slice.
    pub const fn as_slice(&self) -> &[u8] {
        match unsafe { self.chunk.kind() } {
            Kind::Chunk => unsafe { self.chunk.data.as_slice() },
            Kind::Slice => unsafe { self.slice.as_slice() },
            #[cfg(feature = "alloc")]
            Kind::Shared => unsafe {
                core::mem::transmute::<&core::mem::ManuallyDrop<SharedBytes>, &SharedBytes>(
                    &self.shared,
                )
            }
            .as_slice(),
        }
    }

    /// Returns the length of the underlying byte slice.
    pub const fn len(&self) -> usize {
        match unsafe { self.chunk.kind() } {
            Kind::Chunk => unsafe { self.chunk.data.len() },
            Kind::Slice => unsafe { self.slice.len() },
            #[cfg(feature = "alloc")]
            Kind::Shared => unsafe {
                core::mem::transmute::<&core::mem::ManuallyDrop<SharedBytes>, &SharedBytes>(
                    &self.shared,
                )
            }
            .len(),
        }
    }

    /// Returns `true` if the underlying byte slice is empty.
    pub const fn is_empty(&self) -> bool {
        match unsafe { self.chunk.kind() } {
            Kind::Chunk => unsafe { self.chunk.data.is_empty() },
            Kind::Slice => unsafe { self.slice.is_empty() },
            #[cfg(feature = "alloc")]
            Kind::Shared => unsafe {
                core::mem::transmute::<&core::mem::ManuallyDrop<SharedBytes>, &SharedBytes>(
                    &self.shared,
                )
            }
            .is_empty(),
        }
    }

    /// Check if the underlying byte slice is equal to another. This can be used in a `const` context.
    #[inline]
    pub const fn eq_const(&self, other: &ByteData<'_>) -> bool {
        crate::const_eq(self.as_slice(), other.as_slice())
    }

    /// Check if the underlying byte slice is equal to another. This can be used in a `const` context.
    #[inline]
    pub const fn eq_slice(&self, other: &[u8]) -> bool {
        crate::const_eq(self.as_slice(), other)
    }

    /// Check if the ending of a `SharedBytes` matches the given bytes.
    #[inline]
    pub const fn ends_with(&self, needle: &[u8]) -> bool {
        crate::const_ends_with(self.as_slice(), needle)
    }

    /// Check if the beginning of a `SharedBytes` matches the given bytes.
    #[inline]
    pub const fn starts_with(&self, needle: &[u8]) -> bool {
        crate::const_starts_with(self.as_slice(), needle)
    }

    /// Returns a `ByteData` with the given range of bytes.
    pub fn sliced<R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>>(
        &self,
        range: R,
    ) -> Self {
        match unsafe { self.chunk.kind() } {
            Kind::Chunk => Self::from_chunk_slice(unsafe { &self.chunk.data.as_slice()[range] }),
            Kind::Slice => {
                let a = unsafe { &self.slice };
                Self {
                    slice: ByteSlice::new(&a.as_slice()[range], a.is_static()),
                }
            }
            #[cfg(feature = "alloc")]
            Kind::Shared => {
                let dat = unsafe {
                    core::mem::transmute::<&core::mem::ManuallyDrop<SharedBytes>, &SharedBytes>(
                        &self.shared,
                    )
                };
                let dat = dat.sliced_range(range);
                if dat.len() <= 14 {
                    Self::from_chunk_slice(dat.as_slice())
                } else {
                    Self::from_shared(dat)
                }
            }
        }
    }

    /// Transform the range of bytes this `ByteData` represents.
    pub fn into_sliced<R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>>(
        mut self,
        range: R,
    ) -> Self {
        match unsafe { self.chunk.kind() } {
            Kind::Chunk => {
                unsafe { self.chunk.data.make_sliced(range) };
                self
            }
            Kind::Slice => {
                let a = unsafe { &mut self.slice };
                *a = ByteSlice::new(&a.as_slice()[range], a.is_static());
                self
            }
            #[cfg(feature = "alloc")]
            Kind::Shared => {
                let dat = unsafe {
                    core::mem::transmute::<
                        &mut core::mem::ManuallyDrop<SharedBytes>,
                        &mut SharedBytes,
                    >(&mut self.shared)
                };
                dat.make_sliced_range(range);
                if dat.len() <= 14 {
                    let r = Self::from_chunk_slice(dat.as_slice());
                    unsafe { core::ptr::drop_in_place(dat) };
                    core::mem::forget(self);
                    r
                } else {
                    self
                }
            }
        }
    }

    /// Transform the range of bytes this `ByteData` represents.
    pub fn make_sliced<R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>>(
        &'_ mut self,
        range: R,
    ) {
        match unsafe { self.chunk.kind() } {
            Kind::Chunk => {
                unsafe { self.chunk.data.make_sliced(range) };
            }
            Kind::Slice => {
                let a = unsafe { &mut self.slice };
                *a = ByteSlice::new(&a.as_slice()[range], a.is_static());
            }
            #[cfg(feature = "alloc")]
            Kind::Shared => {
                let dat = unsafe {
                    core::mem::transmute::<
                        &mut core::mem::ManuallyDrop<SharedBytes>,
                        &mut SharedBytes,
                    >(&mut self.shared)
                };
                dat.make_sliced_range(range);
                if dat.len() <= 14 {
                    let r = crate::ByteChunk::from_slice(dat.as_slice());
                    unsafe {
                        core::ptr::drop_in_place(dat);
                        self.chunk = DataKind {
                            kind: KIND_CHUNK_MASK,
                            data: r,
                        };
                    }
                }
            }
        }
    }

    #[cfg(feature = "alloc")]
    /// Transform any borrowed data into shared data. This is useful when you wish to change the lifetime of the data.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn into_shared<'s>(mut self) -> ByteData<'s> {
        match unsafe { self.chunk.kind() } {
            Kind::Chunk | Kind::Shared => unsafe {
                core::mem::transmute::<ByteData, ByteData>(self)
            },
            Kind::Slice => {
                let a = unsafe { &self.slice };
                if a.is_static() {
                    unsafe { core::mem::transmute::<ByteData, ByteData>(self) }
                } else if a.len() <= 14 {
                    let r = crate::byte_chunk::ByteChunk::from_slice(a.as_slice());
                    core::mem::forget(self);
                    ByteData {
                        chunk: DataKind {
                            kind: KIND_CHUNK_MASK,
                            data: r,
                        },
                    }
                } else {
                    let r = SharedBytes::from_slice(a.as_slice());
                    self.shared = core::mem::ManuallyDrop::new(r);
                    unsafe { core::mem::transmute::<ByteData, ByteData>(self) }
                }
            }
        }
    }

    #[cfg(feature = "alloc")]
    /// Transform any borrowed data into shared data of a specific range. This is useful when you wish to change the lifetime of the data.
    ///
    /// This is essentially the same as `into_shared().into_sliced(range)`, but it is more efficient.
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn into_shared_range<'s, R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>>(
        mut self,
        range: R,
    ) -> ByteData<'s> {
        match unsafe { self.chunk.kind() } {
            Kind::Chunk => unsafe {
                self.chunk.data.make_sliced(range);
                core::mem::transmute::<ByteData, ByteData>(self)
            },
            Kind::Shared => unsafe {
                (*self.shared).make_sliced_range(range);
                core::mem::transmute::<ByteData, ByteData>(self)
            },
            Kind::Slice => {
                let a = unsafe { &self.slice };
                let r = &a.as_slice()[range];
                if r.len() <= 14 {
                    let r = crate::byte_chunk::ByteChunk::from_slice(r);
                    core::mem::forget(self);
                    return ByteData {
                        chunk: DataKind {
                            kind: KIND_CHUNK_MASK,
                            data: r,
                        },
                    };
                }
                if a.is_static() {
                    core::mem::forget(self);
                    return ByteData {
                        slice: ByteSlice::new(r, true),
                    };
                }
                let r = SharedBytes::from_slice(r);
                self.shared = core::mem::ManuallyDrop::new(r);
                unsafe { core::mem::transmute::<ByteData, ByteData>(self) }
            }
        }
    }

    /// Split the `ByteData` at the given position.
    #[inline]
    pub fn take_bytes(&mut self, position: usize) -> ByteData<'a> {
        if position == 0 {
            return ByteData::empty();
        }
        let a = self.sliced(0..position);
        self.make_sliced(position..);
        a
    }

    /// Consume the `ByteData` until the byte condition is triggered.
    pub fn take_while<F: FnMut(u8) -> bool>(&mut self, mut f: F) -> ByteData<'a> {
        let mut i = 0;
        let a = self.as_slice();
        while i < a.len() && f(a[i]) {
            i += 1;
        }
        if i == 0 {
            return ByteData::empty();
        }
        if i == a.len() {
            return core::mem::replace(self, ByteData::empty());
        }
        let a = self.sliced(0..i);
        self.make_sliced(i..);
        a
    }

    /// Split the `ByteData` at the given position.
    #[inline]
    pub fn split_at(mut self, position: usize) -> (ByteData<'a>, ByteData<'a>) {
        let a = self.sliced(0..position);
        self.make_sliced(position..);
        (a, self)
    }

    /// Split the `ByteData` at the first occurrence of the given byte sequence.
    #[inline]
    pub fn split_once_on(
        self,
        needle: &[u8],
    ) -> Result<(ByteData<'a>, ByteData<'a>), ByteData<'a>> {
        let a = match crate::const_split_once_bytes(self.as_slice(), needle) {
            Some((a, _)) => a.len(),
            None => return Err(self),
        };
        Ok(self.split_at(a))
    }

    /// Split the `ByteData` at the first occurrence of the given byte sequence.
    #[inline]
    pub fn split_on<'b>(self, needle: &'b [u8]) -> impl Iterator<Item = ByteData<'a>> + Send + 'b
    where
        'a: 'b,
    {
        struct It<'a, 'b>(ByteData<'a>, &'b [u8], bool);
        impl<'a, 'b> Iterator for It<'a, 'b> {
            type Item = ByteData<'a>;

            fn next(&mut self) -> Option<Self::Item> {
                if self.0.is_empty() {
                    return None;
                }
                let a = match crate::const_split_once_bytes(self.0.as_slice(), self.1) {
                    Some((a, _)) => a.len(),
                    None => {
                        let r = core::mem::replace(&mut self.0, ByteData::empty());
                        return Some(r);
                    }
                };
                if a == 0 && self.2 {
                    self.2 = false;
                    return Some(ByteData::empty());
                }
                self.2 = false;
                let a = self.0.take_bytes(a);
                Some(a)
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
    pub fn statically_borrowed(mut self) -> ByteData<'static> {
        if matches!(unsafe { self.chunk.kind() }, Kind::Slice) {
            unsafe { self.slice.make_static() };
        }
        unsafe { core::mem::transmute::<ByteData, ByteData>(self) }
    }
}

impl AsRef<[u8]> for ByteData<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<'a> Deref for ByteData<'a> {
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
impl<'a> From<SharedBytes> for ByteData<'a> {
    #[inline]
    fn from(dat: SharedBytes) -> Self {
        if dat.len() <= 14 {
            Self::from_chunk_slice(&dat)
        } else {
            Self::from_shared(dat)
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a> From<Vec<u8>> for ByteData<'a> {
    #[inline]
    fn from(dat: Vec<u8>) -> Self {
        Self::from_shared(dat.into())
    }
}

#[cfg(feature = "alloc")]
impl<'a> From<alloc::string::String> for ByteData<'a> {
    #[inline]
    fn from(dat: alloc::string::String) -> Self {
        Self::from_shared(dat.into())
    }
}

impl Index<usize> for ByteData<'_> {
    type Output = u8;
    fn index(&self, idx: usize) -> &Self::Output {
        let sl = self.as_slice();
        if idx >= sl.len() {
            panic!("ByteData::index: index out of bounds");
        }
        unsafe { &*sl.as_ptr().add(idx) }
    }
}

impl<'a, 'b> PartialEq<ByteData<'b>> for ByteData<'a> {
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

impl<'a, 'b> PartialEq<ByteData<'a>> for &'b [u8] {
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
        self.as_slice().hash(state)
    }
}

impl<'a, 'b> PartialOrd<ByteData<'b>> for ByteData<'a> {
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.as_slice(), f)
    }
}

impl core::fmt::LowerHex for ByteData<'_> {
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

impl core::fmt::UpperHex for ByteData<'_> {
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

impl<'a> Iterator for ByteData<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            return None;
        }
        let r = self[0];
        self.make_sliced(1..);
        Some(r)
    }
}

impl<'a> core::borrow::Borrow<[u8]> for ByteData<'a> {
    #[inline]
    fn borrow(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<'a> Default for ByteData<'a> {
    #[inline]
    fn default() -> Self {
        ByteData::empty()
    }
}
