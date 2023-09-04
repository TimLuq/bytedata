//! # bytedata
//!
//! This crate provides a type that can be used to represent a byte slice that is either static, borrowed, or shared.
//! The byte slice can be accessed using the `as_slice` method which is `const`, which allows the type to be used in a `const` context.
//!
//! This crate is `no_std` but requires the `alloc` crate and uses the global allocator for shared byte slices.
//!
//! ## Example
//!
//! ```
//! use ::bytedata::ByteData;
//!
//! # fn main() {
//! const STATIC: ByteData<'static> = ByteData::from_static(b"hello world");
//! const BORROWED: ByteData<'_> = ByteData::from_borrowed(b"hello world");
//!
//! assert_eq!(STATIC, BORROWED);
//! assert_eq!(STATIC.as_slice(), BORROWED.as_slice());
//! # }

#![no_std]

extern crate alloc;

mod bytedata;
pub use self::bytedata::*;

mod shared_bytes;
pub use self::shared_bytes::*;

mod shared_bytes_builder;
pub use self::shared_bytes_builder::*;

#[cfg(feature = "bytes_1")]
mod bytes_1;

pub const fn const_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// Helper function for `starts_with` in a `const` context.
pub const fn const_starts_with(haystack: &[u8], needle: &[u8]) -> bool {
    if haystack.len() < needle.len() {
        return false;
    }
    let mut i = 0;
    while i < needle.len() {
        if haystack[i] != needle[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// Helper function for `ends_with` in a `const` context.
pub const fn const_ends_with(haystack: &[u8], needle: &[u8]) -> bool {
    let len = needle.len();
    if haystack.len() < len {
        return false;
    }
    let mut i = 0;
    let offs = haystack.len() - len;
    let haystack = unsafe { core::slice::from_raw_parts(haystack.as_ptr().add(offs), len) };
    while i < needle.len() {
        if haystack[i] != needle[i] {
            return false;
        }
        i += 1;
    }
    true
}

#[cfg(test)]
mod test;
