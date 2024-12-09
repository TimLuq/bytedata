#![no_main]

use libfuzzer_sys::{fuzz_target, arbitrary::{Arbitrary, Unstructured}};

use bytedata::ByteQueue;

fn is_same_as(data: &[u8], queue: &ByteQueue) -> bool {
    let mut offs = 0;
    for chunk in queue.chunks() {
        if chunk.as_slice() != &data[offs..offs + chunk.len()] {
            return false;
        }
        offs += chunk.len();
    }
    offs == data.len()
}

fuzz_target!(|input: &[u8]| {
    let uns = Unstructured::new(input);
    let mut queue = <ByteQueue as Arbitrary>::arbitrary_take_rest(uns).unwrap();
    let static_ref_data: Vec<u8> = queue.bytes().collect();
    assert_eq!(queue.len(), static_ref_data.len());
    if queue.is_empty() {
        return;
    }
    if !is_same_as(&static_ref_data, &queue) {
        let ref_data = bytedata::ByteStringRender::from_slice(&static_ref_data);
        panic!("queue is not the same as the input data (from start)\r\n    queue:    {queue:?}\r\n    ref_data: {ref_data:?}");
    }
    let mut ref_data = static_ref_data.clone();

    {
        let mut chunks = Vec::with_capacity(queue.chunk_len());
        for chunk in queue.chunks() {
            chunks.push(chunk.len());
        }
        let end = queue.len() - 1;
        let end_byte = queue.split_off(end);
        assert_eq!(end_byte.len(), 1);
        assert_eq!(queue.len(), end);
        queue.push_back(end_byte);
        assert_eq!(queue.len(), end + 1);
        
        if !is_same_as(&ref_data, &queue) {
            let ref_data = bytedata::ByteStringRender::from_slice(&ref_data);
            panic!("queue is not the same as the input data (after re-push)\r\n    queue:    {queue:?}\r\n    ref_data: {ref_data:?}\r\n   chunks:   {chunks:?}");
        }
    }

    {
        queue.push_front(b"test".as_ref());
        assert_eq!(queue.len(), ref_data.len() + 4);
        _ = queue.drain(0..5);
        assert_eq!(queue.len(), ref_data.len() - 1);
        let len = ref_data.len();
        core::mem::drop(ref_data.drain(..1));

        if !is_same_as(&ref_data, &queue) {
            let ref_data = bytedata::ByteStringRender::from_slice(&ref_data);
            panic!("queue is not the same as the input data (after push-drain)\r\n    queue:    {queue:?}\r\n    ref_data: {ref_data:?}");
        }

        let mut dr = ref_data.drain(..);
        let mut n = 0;
        let mut dr2 = queue.drain(..);
        while let Some(b) = dr2.next() {
            n += 1;
            let ref_b = dr.next().unwrap();
            if b != ref_b {
                let dr = dr.collect::<Vec<u8>>();
                let dr2 = dr2.collect::<Vec<u8>>();
                let part0 = &static_ref_data[..n];
                let ref_data = [part0, core::slice::from_ref(&ref_b), &dr];
                let ref_data = bytedata::MultiByteStringRender::new(&ref_data);
                let queue = [part0, core::slice::from_ref(&b), &dr2];
                let queue = bytedata::MultiByteStringRender::new(&queue);
                panic!("byte {n}/{len} differs, {b} != {ref_b}\r\n    queue:    {queue:?}\r\n    ref_data: {ref_data:?}");
                return;
            }
        }
    }
});
