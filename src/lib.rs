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

mod stringdata;
pub use self::stringdata::*;

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

/// Helper function for slicing slices in a `const` context. Can be used to replace [`slice::get`](https://doc.rust-lang.org/std/primitive.slice.html#method.get) or brackets in `b[1..4]`.
pub const fn const_slice(a: &'_ [u8], range: core::ops::Range<usize>) -> Option<&'_ [u8]> {
    let start = range.start;
    let end = range.end;
    if start > end || end > a.len() {
        return None;
    }
    unsafe { Some(core::slice::from_raw_parts(a.as_ptr().add(start), end - start)) }
}

/// An error that can occur when slicing a `str`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrSliceError {
    /// The slice would cause the result to be out of bounds of the original `str`.
    OutOfBounds,
    /// The slice would cause the result to be split on a UTF-8 char boundary.
    InvalidUtf8,
}

/// Helper function for slicing `str`s in a `const` context. Can be used to replace [`str::get`](https://doc.rust-lang.org/std/primitive.str.html#method.get) or brackets in `s[1..4]`.
pub const fn const_slice_str(a: &'_ str, range: core::ops::Range<usize>) -> Result<&'_ str, StrSliceError> {
    let a = a.as_bytes();
    let start = range.start;
    let end = range.end;
    if start > end || end > a.len() {
        return Err(StrSliceError::OutOfBounds);
    }
    if start != 0 && a[start] & 0b1100_0000 == 0b1000_0000 {
        return Err(StrSliceError::InvalidUtf8);
    }
    if end != a.len() && a[end] & 0b1100_0000 == 0b1000_0000 {
        return Err(StrSliceError::InvalidUtf8);
    }
    unsafe { Ok(core::str::from_utf8_unchecked(core::slice::from_raw_parts(a.as_ptr().add(start), end - start))) }
}

#[cfg(test)]
mod test;
