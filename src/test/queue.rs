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

#[test]
#[allow(clippy::panic)]
fn byte_queue_append_test() {
    static A_DATA: &[&[u8]] = &[
        b"a0", b"a1", b"a2", b"a3", b"a4", b"a5", b"a6", b"a7", b"a8", b"a9", b"aA", b"aB", b"aC",
        b"aD", b"aE", b"aF",
    ];
    static B_DATA: &[&[u8]] = &[
        b"b0", b"b1", b"b2", b"b3", b"b4", b"b5", b"b6", b"b7", b"b8", b"b9", b"bA", b"bB", b"bC",
        b"bD", b"bE", b"bF", b"bG", b"bH", b"bI",
    ];

    let mut a_queue = crate::ByteQueue::new();
    for data in A_DATA {
        a_queue.push_back(*data);
    }
    for (chunk, data) in a_queue.chunks().zip(A_DATA) {
        assert_eq!(chunk.as_slice(), *data);
    }

    let mut b_queue = crate::ByteQueue::new();
    for data in B_DATA {
        b_queue.push_back(*data);
    }
    for (chunk, data) in b_queue.chunks().zip(B_DATA) {
        assert_eq!(chunk.as_slice(), *data);
    }

    a_queue.append(b_queue);

    let mut i = 0;
    while let Some(chunk) = a_queue.pop_front() {
        if i < A_DATA.len() {
            assert_eq!(chunk.as_slice(), A_DATA[i]);
        } else {
            assert_eq!(chunk.as_slice(), B_DATA[i - A_DATA.len()]);
        }
        i += 1;
    }
    assert_eq!(i, A_DATA.len() + B_DATA.len());
}
