#![no_main]

use libfuzzer_sys::fuzz_target;

use bytedata::ByteData;

fuzz_target!(|input: ByteData| {
    let full_len = input.len();
    let owned = input.as_slice().to_vec();
    assert_eq!(owned.len(), full_len);
    let mut cloned_data = input.clone();
    assert_eq!(cloned_data.len(), full_len);
    let mut sliced_data = input.sliced(0..full_len);
    assert_eq!(sliced_data.len(), full_len);
    let fst_byte = input.first().copied();
    let lst_byte = input.as_slice().last().copied();
    assert_eq!(fst_byte, sliced_data.first().copied());
    assert_eq!(lst_byte, sliced_data.as_slice().last().copied());
    assert_eq!(fst_byte, owned.first().copied());
    assert_eq!(lst_byte, owned.last().copied());

    if !input.is_empty() {
        let mut sliced_data = input.sliced(0..1);
        assert_eq!(fst_byte, sliced_data.first().copied());
        assert_eq!(fst_byte, sliced_data.last());
        let mut sliced_data = input.sliced(full_len - 1..full_len);
        assert_eq!(lst_byte, sliced_data.first().copied());
        assert_eq!(lst_byte, sliced_data.last());
    }

    let mut owned_iter = owned.into_iter();
    let mut input_iter = input.iter();
    loop {
        match (owned_iter.next(), input_iter.next()) {
            (Some(owned_byte), Some(input_byte)) => {
                assert_eq!(owned_byte, *input_byte);
            }
            (None, None) => {
                break;
            }
            _ => {
                panic!("Different lengths");
            }
        }
    }
});
