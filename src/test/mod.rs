#[cfg(feature = "alloc")]
mod shared_bytes;
#[cfg(feature = "alloc")]
mod shared_bytes_builder;

mod stringdata;

#[cfg(feature = "macros")]
mod macros;

#[cfg(all(feature = "queue", feature = "alloc"))]
mod queue;

#[cfg(feature = "bytes_1")]
mod bytes_1;

#[test]
fn next_char_test() {
    use crate::const_utf8_char_next;

    assert_eq!(const_utf8_char_next(b"abc"), ('a' as u32, 1));
    assert_eq!(const_utf8_char_next("£".as_bytes()), ('£' as u32, 2));
    assert_eq!(const_utf8_char_next("€".as_bytes()), ('€' as u32, 3));
    assert_eq!(const_utf8_char_next("𐍈".as_bytes()), ('𐍈' as u32, 4));
}
