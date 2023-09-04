#[test]
fn test_shared_bytes_from_slice() {
    let s0 = crate::shared_bytes::SharedBytes::from_slice(b"hello world");
    assert_eq!(s0, b"hello world".as_slice());
    assert_eq!(s0.ref_count(), 1);
    let s1 = s0.clone();
    assert_eq!(s1, b"hello world".as_slice());
    assert_eq!(s0.ref_count(), 2);
    assert_eq!(s1.ref_count(), 2);
    let s2 = s1.sliced(1, 8);
    assert_eq!(s2, b"ello wor".as_slice());
    assert_eq!(s0.ref_count(), 3);
    assert_eq!(s1.ref_count(), 3);
    assert_eq!(s2.ref_count(), 3);
    core::mem::drop(s0);
    assert_eq!(s1.ref_count(), 2);
    assert_eq!(s2.ref_count(), 2);
    core::mem::drop(s1);
    assert_eq!(s2.ref_count(), 1);
}
