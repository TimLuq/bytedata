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

#[cfg(feature = "alloc")]
#[doc(hidden)]
#[inline]
#[allow(clippy::unwrap_used)]
#[must_use]
pub fn __format_shared<'a>(args: core::fmt::Arguments<'_>) -> crate::StringData<'a> {
    if let Some(args2) = args.as_str() {
        return crate::StringData::from_static(args2);
    }
    let mut me = crate::SharedStrBuilder::new();
    core::fmt::Write::write_fmt(&mut me, args).unwrap();
    me.build_str()
}

/// Formats a format string with arguments into a shared `StringData`.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[macro_export]
macro_rules! format_shared {
    ($fmt:expr) => {
        $crate::__format_shared(core::format_args!($fmt))
    };
    ($fmt:expr, $($args:tt)*) => {
        $crate::__format_shared(core::format_args!($fmt, $($args)*))
    };
}

#[cfg(all(feature = "queue", feature = "alloc"))]
#[doc(hidden)]
#[inline]
#[allow(clippy::unwrap_used)]
#[must_use]
pub fn __format_queue<'a>(args: core::fmt::Arguments<'_>) -> crate::StringQueue<'a> {
    if let Some(args2) = args.as_str() {
        return crate::StringQueue::with_item(crate::StringData::from_static(args2));
    }
    let mut me = crate::StringQueue::new();
    core::fmt::Write::write_fmt(&mut me, args).unwrap();
    me
}

/// Formats a format string with arguments into an owned `StringQueue`.
///
/// There is currently no way to optimize shallow clones of `StringData` or `StringQueue` instances, so prefer to use [`StringQueue::push_back`] or [`StringQueue::append`] to build a queue of prepared strings.
#[cfg(all(feature = "queue", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg_attr(docsrs, doc(cfg(feature = "queue")))]
#[macro_export]
macro_rules! format_queue {
    ($fmt:expr) => {
        $crate::__format_queue(core::format_args!($fmt))
    };
    ($fmt:expr, $($args:tt)*) => {
        $crate::__format_queue(core::format_args!($fmt, $($args)*))
    };
}
