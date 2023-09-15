# bytedata

A library for working with byte arrays in a `const`-friendly manner.

When the `alloc` feature is enabled this crate provides `SharedBytes` which operates like `Arc<[u8]>` but with a representation that allows for `const` references to the bytes and zero-copy subslicing.
The `SharedBytes` type is then also added as one additional representation of `ByteData`.

## Features

### alloc

Enables runtime allocation of byte arrays on the heap.
This allows for dynamic allocation of byte arrays which are exposed as `SharedBytes` and can be wrapped as `ByteData::Shared(_)`.

### macros

Exposes the `concat_bytes_static!` and `concat_str_static!` macros.

These macros allow for concatenation of static byte arrays and strings that are based on `const` values and not necessarily literals.

### bytes_1

Enables integration with the `bytes` crate (version `>=1.2, <2`).
This allows for conversion between `SharedBytes` and `bytes::Bytes` types.
Where possible no bytes will be cloned, which means that `ByteData::Static(_)` will map to `bytes::Bytes::from_static`,
and that `<bytes::Bytes as From<SharedBytes>>::from` will return a `bytes::Bytes` object that is just a wrapper and still share the bytes as normal without any copy.

There is, however, a possibility that the internal vtable or structure of `bytes::Bytes` changes in the future, in which case the zero-copy may break or segfault.
If this happens you can enable the feature `bytes_1_safe` which will always cause the bytes to be cloned when converting to and from `bytes::Bytes` without the use of any internal structures.

### bytes_1_safe

Enables integration with the `bytes` crate (version `>=1.2, <2`) in a safe manner.
This will cause the bytes to always be cloned when converting to and from `bytes::Bytes`.

For zero-copy conversion between `SharedBytes` and `bytes::Bytes`, use the `bytes_1` feature instead unless it is broken.
