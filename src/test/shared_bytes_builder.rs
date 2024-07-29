#[test]
fn test_shared_bytes_builder_from_slice() {
    let data = b"hello world".as_slice();
    let data2 = b"2".as_slice();
    let mut s0 = crate::shared_bytes::SharedBytes::builder();
    s0.extend_from_slice(data);
    s0.extend_from_slice(data2);

    assert_eq!(s0.align, core::mem::align_of::<crate::SharedBytesMeta>());
    assert_eq!(
        s0.off as usize,
        core::mem::size_of::<crate::SharedBytesMeta>() + data.len() + data2.len()
    );
    assert_eq!(
        s0.capacity(),
        32 - core::mem::size_of::<crate::SharedBytesMeta>()
    );

    let s0 = s0.build();
    assert_eq!(s0.len(), data.len() + data2.len());
    assert_eq!(&s0.as_slice()[..data.len()], data);
    assert_eq!(&s0.as_slice()[data.len()..], data2);
    assert_eq!(s0.ref_count(), 1);
}

#[test]
fn test_shared_bytes_builder_from_aligned_slice() {
    let data = b"hello world".as_slice();
    let data2 = b"2".as_slice();
    let mut s0 = crate::shared_bytes_builder::SharedBytesBuilder::with_alignment(8);
    s0.extend_from_slice(data);
    s0.extend_from_slice(data2);

    assert_eq!(s0.align, 8);
    assert_eq!(
        s0.off as usize,
        core::mem::size_of::<crate::SharedBytesMeta>() + 4 + data.len() + data2.len()
    );
    assert_eq!(
        s0.capacity(),
        32 - core::mem::size_of::<crate::SharedBytesMeta>() - 4
    );

    let s0 = s0.build();
    assert_eq!(s0.len(), data.len() + data2.len());
    assert_eq!(&s0.as_slice()[..data.len()], data);
    assert_eq!(&s0.as_slice()[data.len()..], data2);
    assert_eq!(s0.ref_count(), 1);
}
