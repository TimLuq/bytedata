#[test]
fn test_shared_bytes_from_slice() {
    let mut s0 = crate::shared_bytes::SharedBytes::builder();
    s0.extend_from_slice(b"hello world");
}
