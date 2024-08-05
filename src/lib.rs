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
#![cfg_attr(
    all(feature = "nightly", feature = "core_io_borrowed_buf"),
    feature(core_io_borrowed_buf)
)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::module_name_repetitions)]
#![warn(
    clippy::alloc_instead_of_core,
    clippy::allow_attributes,
    clippy::allow_attributes_without_reason,
    clippy::as_underscore,
    clippy::assertions_on_result_states,
    clippy::clone_on_ref_ptr,
    clippy::create_dir,
    clippy::dbg_macro,
    clippy::decimal_literal_representation,
    clippy::default_numeric_fallback,
    clippy::default_union_representation,
    clippy::deref_by_slicing,
    clippy::else_if_without_else,
    clippy::empty_drop,
    clippy::empty_enum_variants_with_brackets,
    clippy::empty_structs_with_brackets,
    clippy::error_impl_error,
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::exit,
    clippy::expect_used,
    clippy::filetype_is_file,
    clippy::float_cmp_const,
    clippy::fn_to_numeric_cast_any,
    clippy::format_push_string,
    clippy::get_unwrap,
    clippy::impl_trait_in_params,
    clippy::infinite_loop,
    clippy::integer_division,
    clippy::let_underscore_must_use,
    clippy::let_underscore_untyped,
    clippy::lossy_float_literal,
    clippy::map_err_ignore,
    clippy::min_ident_chars,
    clippy::missing_assert_message,
    clippy::missing_asserts_for_indexing,
    clippy::missing_inline_in_public_items,
    clippy::mixed_read_write_in_expression,
    clippy::multiple_inherent_impl,
    clippy::multiple_unsafe_ops_per_block,
    clippy::mutex_atomic,
    clippy::needless_raw_strings,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::partial_pub_fields,
    clippy::pattern_type_mismatch,
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::pub_without_shorthand,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::redundant_type_annotations,
    clippy::ref_patterns,
    clippy::renamed_function_params,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::self_named_module_files,
    clippy::semicolon_outside_block,
    clippy::shadow_unrelated,
    clippy::std_instead_of_alloc,
    clippy::std_instead_of_core,
    clippy::str_to_string,
    clippy::string_add,
    clippy::string_lit_chars_any,
    clippy::string_slice,
    clippy::string_to_string,
    clippy::tests_outside_test_module,
    clippy::todo,
    clippy::try_err,
    clippy::undocumented_unsafe_blocks,
    clippy::unimplemented,
    clippy::unnecessary_safety_comment,
    clippy::unnecessary_safety_doc,
    clippy::unnecessary_self_imports,
    clippy::unneeded_field_pattern,
    clippy::unseparated_literal_suffix,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::verbose_file_reads,
    clippy::wildcard_enum_match_arm
)]
#![warn(
    missing_abi,
    missing_docs,
    missing_copy_implementations,
    missing_crate_level_docs,
    missing_debug_implementations,
    missing_unsafe_on_extern
)]
#![warn(
    absolute_paths_not_starting_with_crate,
    deprecated_safe,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    keyword_idents_2024,
    macro_use_extern_crate,
    meta_variable_misuse,
    non_ascii_idents,
    non_local_definitions,
    redundant_lifetimes,
    single_use_lifetimes,
    trivial_numeric_casts,
    unit_bindings,
    unnameable_types,
    unreachable_pub
)]

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
#[allow(unreachable_pub)]
pub use self::macros::*;

mod byte_chunk;
pub use byte_chunk::ByteChunk;

#[cfg(feature = "queue")]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
pub mod queue;
#[cfg(feature = "queue")]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
pub use queue::{ByteQueue, StringQueue};

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub mod external;

#[cfg(not(feature = "alloc"))]
mod external {
    pub(crate) const KIND_EXT_BYTES: u8 = 0b0000_0011;
}

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
#[must_use]
#[inline]
pub const fn const_eq(a_val: &[u8], b_val: &[u8]) -> bool {
    if a_val.len() != b_val.len() {
        return false;
    }
    let mut i = 0;
    while i < a_val.len() {
        if a_val[i] != b_val[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// Check if a byte slice starts with another in a `const` context.
#[must_use]
#[inline]
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
#[must_use]
#[inline]
pub const fn const_ends_with(haystack: &[u8], needle: &[u8]) -> bool {
    let len = needle.len();
    if haystack.len() < len {
        return false;
    }
    let mut i = 0;
    let offs = haystack.len() - len;
    // SAFETY: then ending part of the haystack
    let end_offset = unsafe { haystack.as_ptr().add(offs) };
    // SAFETY: then ending part of the haystack
    let hayend = unsafe { core::slice::from_raw_parts(end_offset, len) };
    while i < needle.len() {
        if hayend[i] != needle[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// Helper function for slicing slices in a `const` context.
/// Can be used to replace [`slice::get`](https://doc.rust-lang.org/core/primitive.slice.html#method.get) or brackets (such as in `b[1..4]`).
#[must_use]
#[inline]
pub const fn const_slice(data: &'_ [u8], range: core::ops::Range<usize>) -> Option<&'_ [u8]> {
    let start = range.start;
    let end = range.end;
    if start > end || end > data.len() {
        return None;
    }
    // SAFETY: the range is within bounds
    let data = unsafe { data.as_ptr().add(start) };
    // SAFETY: the range is within bounds
    unsafe { Some(core::slice::from_raw_parts(data, end - start)) }
}

/// The different states that can occur when slicing a `str`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::exhaustive_enums)]
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
    #[must_use]
    #[inline]
    pub const fn ok(self) -> Option<&'a str> {
        match self {
            StrSliceResult::Success(st) => Some(st),
            StrSliceResult::OutOfBounds | StrSliceResult::InvalidUtf8 => None,
        }
    }

    /// Returns the sliced `str` if the slice was valid. Otherwise panics.
    #[must_use]
    #[inline]
    pub const fn unwrap(self) -> &'a str {
        match self {
            StrSliceResult::Success(st) => st,
            StrSliceResult::OutOfBounds | StrSliceResult::InvalidUtf8 => {
                panic!("unwrap of StrSliceResult failed")
            }
        }
    }

    /// Returns the error if the slice was invalid.
    #[must_use]
    #[inline]
    pub const fn err(self) -> Option<Self> {
        match self {
            StrSliceResult::Success(_) => None,
            StrSliceResult::OutOfBounds | StrSliceResult::InvalidUtf8 => Some(self),
        }
    }

    /// Checks if the slice was valid.
    #[must_use]
    #[inline]
    pub const fn is_ok(&self) -> bool {
        matches!(self, &StrSliceResult::Success(_))
    }

    /// Checks if the slice was invalid.
    #[must_use]
    #[inline]
    pub const fn is_err(&self) -> bool {
        !matches!(self, &StrSliceResult::Success(_))
    }
}

/// Helper function for slicing `str`s in a `const` context.
/// Can be used to replace [`str::get`](https://doc.rust-lang.org/core/primitive.str.html#method.get) or brackets (such as in `s[1..4]`).
#[inline]
#[must_use]
pub const fn const_slice_str(data: &'_ str, range: core::ops::Range<usize>) -> StrSliceResult<'_> {
    let data = data.as_bytes();
    let start = range.start;
    let end = range.end;
    if start > end || end > data.len() {
        return StrSliceResult::OutOfBounds;
    }
    if start != 0 && data[start] & 0b1100_0000 == 0b1000_0000 {
        return StrSliceResult::InvalidUtf8;
    }
    if end != data.len() && data[end] & 0b1100_0000 == 0b1000_0000 {
        return StrSliceResult::InvalidUtf8;
    }
    // SAFETY: the range is within bounds
    let data = unsafe { data.as_ptr().add(start) };
    // SAFETY: the range is within bounds
    let data = unsafe { core::slice::from_raw_parts(data, end - start) };
    // SAFETY: the slice is valid UTF-8
    unsafe { StrSliceResult::Success(core::str::from_utf8_unchecked(data)) }
}

/// Simple helper function to return a constant string or a default string.
#[inline]
#[must_use]
pub const fn const_or_str<'a>(value: Option<&'a str>, default: &'a str) -> &'a str {
    match value {
        Some(value) => value,
        None => default,
    }
}

/// Simple helper function to return a constant byte sequence or a default sequence.
#[inline]
#[must_use]
pub const fn const_or_bytes<'a>(value: Option<&'a [u8]>, default: &'a [u8]) -> &'a [u8] {
    match value {
        Some(value) => value,
        None => default,
    }
}

/// Helper function to build a constant array of bytes from a list of byte slices and a known maximum buffer length.
///
/// # Panics
///
/// Will panic if the buffer is too small.
#[inline]
#[must_use]
pub const fn build_const_bytes<const N: usize>(
    mut data: [u8; N],
    inputs: &[&[u8]],
) -> ([u8; N], usize) {
    let mut pos = 0;
    let mut i = 0;
    while i < inputs.len() {
        let input = inputs[i];
        let mut j = 0;
        while j < input.len() {
            assert!(pos < N, "build_const_bytes: array too small");
            data[pos] = input[j];
            pos += 1;
            j += 1;
        }
        i += 1;
    }
    (data, pos)
}

/// Helper function to split a constant slice of bytes on a specific byte.
/// The matching byte will still be there in the second slice.
#[inline]
#[must_use]
pub const fn const_split_once_byte(haystack: &'_ [u8], needle: u8) -> Option<(&'_ [u8], &'_ [u8])> {
    let mut pos = 0;
    while pos < haystack.len() {
        if haystack[pos] == needle {
            let av = const_or_bytes(const_slice(haystack, 0..pos), &[]);
            let bv = const_or_bytes(const_slice(haystack, pos..haystack.len()), &[]);
            return Some((av, bv));
        }
        pos += 1;
    }
    None
}

/// Helper function to find the first position of a subsequence of bytes.
/// Time complexity `O(n*m)`.
#[inline]
#[must_use]
pub const fn const_find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    let mut pos = 0;
    if needle.len() > haystack.len() {
        return None;
    }
    let max_start = haystack.len() - needle.len();
    loop {
        // SAFETY: the position is within bounds
        let hayptr = unsafe { haystack.as_ptr().add(pos) };
        // SAFETY: the position is within bounds
        let hs = unsafe { core::slice::from_raw_parts(hayptr, needle.len()) };
        if !const_eq(hs, needle) {
            pos += 1;
            if pos <= max_start {
                continue;
            }
            return None;
        }
        return Some(pos);
    }
}

/// Helper function to split a constant sequence of bytes on a specific sequence of bytes.
/// The matching bytes will still be there in the second slice.
/// Time complexity `O(n*m)`.
#[inline]
#[must_use]
pub const fn const_split_once_bytes<'a>(
    haystack: &'a [u8],
    needle: &'_ [u8],
) -> Option<(&'a [u8], &'a [u8])> {
    let Some(pos) = const_find_bytes(haystack, needle) else {
        return None;
    };
    // SAFETY: the position is within bounds
    let av = unsafe { core::slice::from_raw_parts(haystack.as_ptr(), pos) };
    // SAFETY: the position is within bounds
    let hayptr = unsafe { haystack.as_ptr().add(pos) };
    // SAFETY: the position is within bounds
    let hs = unsafe { core::slice::from_raw_parts(hayptr, haystack.len() - pos) };

    Some((av, hs))
}

/// Helper function to split a constant str of bytes on a specific substring.
/// The matching substring will still be there in the second string.
/// Time complexity `O(n*m)`.
#[inline]
#[must_use]
pub const fn const_split_once_str<'a>(
    haystack: &'a str,
    needle: &'_ str,
) -> Option<(&'a str, &'a str)> {
    match const_split_once_bytes(haystack.as_bytes(), needle.as_bytes()) {
        Some((av, bv)) => {
            // SAFETY: if the haystack and needle is safe, this is safe
            let av = unsafe { core::str::from_utf8_unchecked(av) };
            // SAFETY: if the haystack and needle is safe, this is safe
            let bv = unsafe { core::str::from_utf8_unchecked(bv) };
            Some((av, bv))
        }
        None => None,
    }
}

#[cfg(test)]
mod test;
