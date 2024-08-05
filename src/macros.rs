/// Concatenate a list of byte literals or const byte slices into a single static byte slice.
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
#[macro_export]
macro_rules! concat_bytes_static {
    ($($e:expr),* $(,)*) => {
        {
            const __LEN: usize = 0 $(+ $e.len())*;
            static __ARR: [u8; __LEN] = $crate::build_const_bytes::<__LEN>([0_u8; __LEN], &[$($e),*]).0;
            &__ARR
        }
    };
}

/// Concatenate a list of string literals or consts of type `&'static str` into a single static string literal.
///
/// # Example
///
/// ```
/// static HELLO_WORLD: &str = ::bytedata::concat_str_static!("Hello", " ", "world!");
/// assert_eq!(HELLO_WORLD, "Hello world!");
/// ```
///
/// ```
/// const HELLO: &str = "Hello";
/// const WORLD: &str = ::bytedata::const_or_str(::bytedata::const_slice_str("world!", 0..5).ok(), "");
/// static HELLO_WORLD: &str = ::bytedata::concat_str_static!(HELLO, ::bytedata::const_or_str(Some(" "), ""), WORLD, "!");
/// assert_eq!(HELLO_WORLD, "Hello world!");
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
#[macro_export]
macro_rules! concat_str_static {
    ($($e:expr),* $(,)*) => {
        {
            #[allow(clippy::string_lit_as_bytes)]
            static __BYT: &[u8] = $crate::concat_bytes_static!( $($e.as_bytes()),* );
            static __STR: &str = match core::str::from_utf8(__BYT) {
                Ok(__s) => __s,
                Err(_) => panic!("concatenated string is not valid UTF-8"),
            };
            __STR
        }
    };
}
