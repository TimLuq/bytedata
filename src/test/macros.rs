#[test]
fn test_macros_bytes() {
    static HW: &[u8] = crate::concat_bytes_static!(b"hello", b" ", b"world");
    assert_eq!(HW, b"hello world");
}

#[test]
fn test_macros_str() {
    static HW: &str = crate::concat_str_static!("hello", " ", "world");
    assert_eq!(HW, "hello world");
}
