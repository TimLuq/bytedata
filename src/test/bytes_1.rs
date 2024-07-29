#[cfg(feature = "alloc")]
#[test]
fn test_bytedata_bytes_1_static() {
    let s0: ::bytes_1::Bytes = ::bytes_1::Bytes::from_static(b"hello world");
    let s0 = crate::bytedata::ByteData::from(s0);
    #[cfg(not(feature = "bytes_1_safe"))]
    assert!(matches!(
        unsafe { s0.kind }.kind,
        crate::bytedata::KIND_STATIC
    ));
    #[cfg(feature = "bytes_1_safe")]
    assert!(matches!(
        unsafe { s0.kind }.kind,
        crate::bytedata::KIND_SHARED
    ));
    assert_eq!(s0, b"hello world".as_slice());
}

#[cfg(feature = "alloc")]
#[test]
fn test_bytedata_bytes_1_borrowed() {
    let s0: ::bytes_1::Bytes = ::bytes_1::Bytes::copy_from_slice(b"hello world");
    let s0 = crate::bytedata::ByteData::from(s0);
    assert!(matches!(
        unsafe { s0.kind }.kind,
        crate::bytedata::KIND_SHARED
    ));
    assert_eq!(s0, b"hello world".as_slice());
}

#[cfg(feature = "alloc")]
#[test]
fn test_bytedata_bytes_1_boxed() {
    let s0: ::bytes_1::Bytes =
        ::bytes_1::Bytes::from(alloc::boxed::Box::<[u8]>::from(b"hello world".as_slice()));
    let s0 = crate::bytedata::ByteData::from(s0);
    assert!(matches!(
        unsafe { s0.kind }.kind,
        crate::bytedata::KIND_SHARED
    ));
    assert_eq!(s0, b"hello world".as_slice());
}

#[cfg(feature = "alloc")]
#[test]
fn test_bytedata_bytes_1_vec_exact() {
    let s0: ::bytes_1::Bytes =
        ::bytes_1::Bytes::from(alloc::vec::Vec::<u8>::from(b"hello world".as_slice()));
    let s0 = crate::bytedata::ByteData::from(s0);
    assert!(matches!(
        unsafe { s0.kind }.kind,
        crate::bytedata::KIND_SHARED
    ));
    assert_eq!(s0, b"hello world".as_slice());
}

#[cfg(feature = "alloc")]
#[test]
fn test_bytedata_bytes_1_vec_extra() {
    let mut s0 = alloc::vec::Vec::with_capacity(32);
    s0.extend_from_slice(b"hello world");
    let s0: ::bytes_1::Bytes = ::bytes_1::Bytes::from(s0);
    let s0 = crate::bytedata::ByteData::from(s0);
    assert!(matches!(
        unsafe { s0.kind }.kind,
        crate::bytedata::KIND_SHARED
    ));
    assert_eq!(s0, b"hello world".as_slice());
}
