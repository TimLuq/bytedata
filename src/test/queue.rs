fn is_same_as(src: &[u8], dst: &crate::ByteQueue<'_>) -> bool {
    let mut offs = 0;
    for chunk in dst.chunks() {
        if chunk.as_slice() != &src[offs..offs + chunk.len()] {
            return false;
        }
        offs += chunk.len();
    }
    offs == src.len()
}

#[test]
#[allow(clippy::panic)]
fn byte_queue_test() {
    // b"AAl\x00\x00\x00\x00A\x00\x99\x01\x00\x84\x9c\x9c\x99\x00\x00\x00\x00\x00\x00\x00\x00\x00"
    static FULL_REF: &[u8] =
        b"AAl\x00\x00\x00\x00A\x99\x01\x00\x84\x9c\x9c\x99\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
    static EX_DATA: &[&[u8]] = &[
        b"AA",
        b"l\x00\x00\x00\x00",
        b"A",
        b"\x99",
        b"\x01\x00\x84",
        b"\x9c",
        b"\x9c",
        b"\x99",
        b"\x00",
        b"\x00",
        b"\x00",
        b"\x00",
        b"\x00",
        b"\x00",
        b"\x00",
        b"\x00",
        b"\x00",
        b"\x00",
    ];

    let mut queue = crate::ByteQueue::new();
    for data in EX_DATA {
        queue.push_back(*data);
    }
    if !is_same_as(FULL_REF, &queue) {
        let ref_data = crate::ByteStringRender::from_slice(FULL_REF);
        panic!("queue is not the same as the input data (from start)\r\n    queue:    {queue:?}\r\n    ref_data: {ref_data:?}");
    }

    let end = queue.len() - 1;
    let end_byte = queue.split_off(end);
    assert_eq!(end_byte.len(), 1);
    assert_eq!(queue.len(), end);
    queue.push_back(end_byte);
    assert_eq!(queue.len(), end + 1);

    if !is_same_as(FULL_REF, &queue) {
        let ref_data = crate::ByteStringRender::from_slice(FULL_REF);
        panic!("queue is not the same as the input data (from end)\r\n    queue:    {queue:?}\r\n    ref_data: {ref_data:?}");
    }
}
