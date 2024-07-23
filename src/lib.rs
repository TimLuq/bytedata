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

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(all(feature = "nightly", feature = "read_buf"), feature(read_buf))]
#![deny(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod bytedata;
use core::panic;

pub use self::bytedata::*;

mod byte_string_render;
pub use byte_string_render::*;

#[cfg(feature = "alloc")]
mod shared_bytes;
#[cfg(feature = "alloc")]
pub use self::shared_bytes::*;

#[cfg(feature = "alloc")]
mod shared_bytes_builder;
#[cfg(feature = "alloc")]
pub use self::shared_bytes_builder::*;

mod stringdata;
pub use self::stringdata::*;

#[cfg(feature = "std")]
mod std;

#[cfg(feature = "macros")]
mod macros;
#[cfg(feature = "macros")]
#[allow(unused_imports)]
pub use self::macros::*;

#[cfg(feature = "chunk")]
mod byte_chunk;
#[cfg(feature = "chunk")]
pub use byte_chunk::ByteChunk;

#[cfg(feature = "queue")]
pub mod queue;
#[cfg(feature = "queue")]
pub use queue::{ByteQueue, StringQueue};

#[cfg(feature = "bytes_1")]
mod bytes_1;

#[cfg(feature = "http-body_04")]
mod http_body_04;

#[cfg(feature = "http-body_1")]
mod http_body_1;

#[cfg(feature = "nom_7")]
mod nom_7;

#[cfg(feature = "serde_1")]
mod serde_1;

/// Checks if two byte slices are equal in a `const` context.
/// This is however not a *constant time* equality check, as it will return `false` as early as possible.
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

/// Check if a byte slice starts with another in a `const` context.
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

/// Check if a byte slice ends with another in a `const` context.
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

/// Helper function for slicing slices in a `const` context.
/// Can be used to replace [`slice::get`](https://doc.rust-lang.org/core/primitive.slice.html#method.get) or brackets (such as in `b[1..4]`).
#[inline]
pub const fn const_slice(a: &'_ [u8], range: core::ops::Range<usize>) -> Option<&'_ [u8]> {
    let start = range.start;
    let end = range.end;
    if start > end || end > a.len() {
        return None;
    }
    unsafe {
        Some(core::slice::from_raw_parts(
            a.as_ptr().add(start),
            end - start,
        ))
    }
}

/// The different states that can occur when slicing a `str`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrSliceResult<'a> {
    /// The slicing operation resulted in a valid subslice.
    Success(&'a str),
    /// The slice would cause the result to be out of bounds of the original `str`.
    OutOfBounds,
    /// The slice would cause the result to be split on a UTF-8 char boundary.
    InvalidUtf8,
}

impl<'a> StrSliceResult<'a> {
    /// Returns the sliced `str` if the slice was valid.
    pub const fn ok(self) -> Option<&'a str> {
        match self {
            StrSliceResult::Success(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the sliced `str` if the slice was valid. Otherwise panics.
    pub const fn unwrap(self) -> &'a str {
        match self {
            StrSliceResult::Success(s) => s,
            _ => panic!("unwrap of StrSliceResult failed"),
        }
    }

    /// Returns the error if the slice was invalid.
    pub const fn err(self) -> Option<StrSliceResult<'a>> {
        match self {
            StrSliceResult::Success(_) => None,
            _ => Some(self),
        }
    }

    /// Checks if the slice was valid.
    pub const fn is_ok(&self) -> bool {
        matches!(self, StrSliceResult::Success(_))
    }

    /// Checks if the slice was invalid.
    pub const fn is_err(&self) -> bool {
        !matches!(self, StrSliceResult::Success(_))
    }
}

/// Helper function for slicing `str`s in a `const` context.
/// Can be used to replace [`str::get`](https://doc.rust-lang.org/core/primitive.str.html#method.get) or brackets (such as in `s[1..4]`).
#[inline]
pub const fn const_slice_str(a: &'_ str, range: core::ops::Range<usize>) -> StrSliceResult<'_> {
    let a = a.as_bytes();
    let start = range.start;
    let end = range.end;
    if start > end || end > a.len() {
        return StrSliceResult::OutOfBounds;
    }
    if start != 0 && a[start] & 0b1100_0000 == 0b1000_0000 {
        return StrSliceResult::InvalidUtf8;
    }
    if end != a.len() && a[end] & 0b1100_0000 == 0b1000_0000 {
        return StrSliceResult::InvalidUtf8;
    }
    unsafe {
        StrSliceResult::Success(core::str::from_utf8_unchecked(core::slice::from_raw_parts(
            a.as_ptr().add(start),
            end - start,
        )))
    }
}

/// Simple helper function to return a constant string or a default string.
#[inline]
pub const fn const_or_str<'a>(value: Option<&'a str>, default: &'a str) -> &'a str {
    match value {
        Some(value) => value,
        None => default,
    }
}

/// Simple helper function to return a constant byte sequence or a default sequence.
#[inline]
pub const fn const_or_bytes<'a>(value: Option<&'a [u8]>, default: &'a [u8]) -> &'a [u8] {
    match value {
        Some(value) => value,
        None => default,
    }
}

/// Helper function to build a constant array of bytes from a list of byte slices and a known maximum buffer length.
///
/// Will panic if the buffer is too small.
pub const fn build_const_bytes<const N: usize>(
    mut data: [u8; N],
    inputs: &[&[u8]],
) -> ([u8; N], usize) {
    let mut p = 0;
    let mut i = 0;
    while i < inputs.len() {
        let input = inputs[i];
        let mut j = 0;
        while j < input.len() {
            if p >= N {
                panic!("build_const_bytes: array too small");
            }
            data[p] = input[j];
            p += 1;
            j += 1;
        }
        i += 1;
    }
    (data, p)
}

/// Helper function to split a constant slice of bytes on a specific byte.
/// The matching byte will still be there in the second slice.
pub const fn const_split_once_byte(haystack: &'_ [u8], needle: u8) -> Option<(&'_ [u8], &'_ [u8])> {
    let mut p = 0;
    while p < haystack.len() {
        if haystack[p] == needle {
            let a = const_or_bytes(const_slice(haystack, 0..p), &[]);
            let b = const_or_bytes(const_slice(haystack, p..haystack.len()), &[]);
            return Some((a, b));
        }
        p += 1;
    }
    None
}

/// Helper function to find the first position of a subsequence of bytes.
/// Time complexity `O(n*m)`.
pub const fn const_find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    let mut p = 0;
    if needle.len() > haystack.len() {
        return None;
    }
    let max_start = haystack.len() - needle.len();
    loop {
        let hs = unsafe { core::slice::from_raw_parts(haystack.as_ptr().add(p), needle.len()) };
        if !const_eq(hs, needle) {
            p += 1;
            if p <= max_start {
                continue;
            }
            return None;
        }
        return Some(p);
    }
}

/// Helper function to split a constant sequence of bytes on a specific sequence of bytes.
/// The matching bytes will still be there in the second slice.
/// Time complexity `O(n*m)`.
#[inline]
pub const fn const_split_once_bytes<'a>(
    haystack: &'a [u8],
    needle: &'_ [u8],
) -> Option<(&'a [u8], &'a [u8])> {
    let Some(p) = const_find_bytes(haystack, needle) else {
        return None;
    };
    let a = unsafe { core::slice::from_raw_parts(haystack.as_ptr(), p) };
    let hs = unsafe { core::slice::from_raw_parts(haystack.as_ptr().add(p), haystack.len() - p) };
    return Some((a, hs));
}

/// Helper function to split a constant str of bytes on a specific substring.
/// The matching substring will still be there in the second string.
/// Time complexity `O(n*m)`.
#[inline]
pub const fn const_split_once_str<'a>(
    haystack: &'a str,
    needle: &'_ str,
) -> Option<(&'a str, &'a str)> {
    match const_split_once_bytes(haystack.as_bytes(), needle.as_bytes()) {
        Some((a, b)) => unsafe {
            // if the haystack and needle is safe, this is safe
            Some((
                core::str::from_utf8_unchecked(a),
                core::str::from_utf8_unchecked(b),
            ))
        },
        None => None,
    }
}

#[cfg(test)]
mod test;
