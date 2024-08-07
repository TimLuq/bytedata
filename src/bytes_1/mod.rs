#[cfg(all(not(feature = "bytes_1_safe"), feature = "alloc"))]
use core::sync::atomic::AtomicPtr;

#[cfg(all(not(feature = "bytes_1_safe"), feature = "alloc"))]
use ::bytes_1 as bytes;

mod bytedata;

#[cfg(feature = "alloc")]
mod external;
#[cfg(feature = "queue")]
mod queue;
#[cfg(feature = "alloc")]
mod shared_bytes;
#[cfg(feature = "alloc")]
mod shared_bytes_builder;

#[cfg(all(not(feature = "bytes_1_safe"), feature = "alloc"))]
/// A struct containing (hopefully) the same ABI as `Bytes` from the `bytes` crate.
#[allow(dead_code)]
struct SBytes {
    ptr: *const u8,
    len: usize,
    data: AtomicPtr<()>,
    vtable: &'static SBytesVtable,
}

#[cfg(all(not(feature = "bytes_1_safe"), feature = "alloc"))]
unsafe impl Sync for SBytes {}

#[cfg(all(not(feature = "bytes_1_safe"), feature = "alloc"))]
impl SBytes {
    const fn into_bytes(self) -> bytes::Bytes {
        unsafe { core::mem::transmute::<SBytes, bytes::Bytes>(self) }
    }
}

#[cfg(all(not(feature = "bytes_1_safe"), feature = "alloc"))]
/// A struct containing (hopefully) the same ABI as the hidden `Vtable` from the `bytes` crate.
#[allow(dead_code)]
struct SBytesVtable {
    /// fn(data, ptr, len)
    pub clone: unsafe fn(&AtomicPtr<()>, *const u8, usize) -> bytes::Bytes,
    /// fn(data, ptr, len)
    ///
    /// takes `Bytes` to value
    pub to_vec: unsafe fn(&AtomicPtr<()>, *const u8, usize) -> alloc::vec::Vec<u8>,
    pub to_mut: unsafe fn(&AtomicPtr<()>, *const u8, usize) -> bytes::BytesMut,
    /// fn(data)
    pub is_unique: unsafe fn(&AtomicPtr<()>) -> bool,
    /// fn(data, ptr, len)
    pub drop: unsafe fn(&mut AtomicPtr<()>, *const u8, usize),
}
