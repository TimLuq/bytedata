use core::sync::atomic::AtomicPtr;

use alloc::vec::Vec;

use ::bytes_1 as bytes;

mod bytedata;
mod shared_bytes;
mod shared_bytes_builder;

/// A struct containing (hopefully) the same ABI as `Bytes` from the `bytes` crate.
#[allow(dead_code)]
struct SBytes {
    ptr: *const u8,
    len: usize,
    data: AtomicPtr<()>,
    vtable: &'static SBytesVtable,
}

unsafe impl Sync for SBytes {}

impl SBytes {
    const fn into_bytes(self) -> bytes::Bytes {
        unsafe { core::mem::transmute::<SBytes, bytes::Bytes>(self) }
    }
}

/// A struct containing (hopefully) the same ABI as the hidden `Vtable` from the `bytes` crate.
#[allow(dead_code)]
struct SBytesVtable {
    pub clone: unsafe fn(&AtomicPtr<()>, *const u8, usize) -> bytes::Bytes,
    pub to_vec: unsafe fn(&AtomicPtr<()>, *const u8, usize) -> Vec<u8>,
    pub drop: unsafe fn(&mut AtomicPtr<()>, *const u8, usize),
}
