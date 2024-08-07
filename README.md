# bytedata

A library for working with byte slices in a `const`-friendly manner.

Provides the main types `ByteData` and `StringData` which are wrappers around byte sequences and string sequences respectively.

When the `alloc` feature is enabled this crate provides `SharedBytes` which operates like `Arc<[u8]>` but with a representation that allows for `const` references to the bytes and zero-copy subslicing.
The `SharedBytes` type is then also added as one additional representation of `ByteData`.

## Features

### alloc

Enables runtime allocation of byte arrays on the heap.
This allows for dynamic allocation of byte arrays which are exposed as `SharedBytes` and can be wrapped using `ByteData::from_shared`.

### chunk

This feature is now built-in and activating it does nothing.

Previously enabled runtime representation of small byte arrays inline as `ByteChunk` which is one representation of `ByteData`.
This allows for optimized storage of small byte arrays that are less than or equal to 14 bytes in size.

### macros

Exposes the `concat_bytes_static!` and `concat_str_static!` macros.

These macros allow for concatenation of static byte arrays and strings that are based on `const` values and not necessarily literals.

### bytes_1

Enables integration with the `bytes` crate (version `>=1.7.1, <2`).
This allows for conversion between `SharedBytes` and `bytes::Bytes` types.
Where possible no bytes will be cloned, which means that `ByteData::from_static` will map to `bytes::Bytes::from_static`,
and that `<bytes::Bytes as From<SharedBytes>>::from` will return a `bytes::Bytes` object that is just a wrapper and still share the bytes as normal without any copying.

There is, however, a possibility that the internal vtable or structure of `bytes::Bytes` changes in the future or results in a different ordered ABI, in which case the zero-copy may break or segfault.
If this happens you can enable the feature `bytes_1_safe` which will always cause the bytes to be cloned when converting to and from `bytes::Bytes` without the use of any internal structures.

### bytes_1_safe

Enables integration with the `bytes` crate (version `>=1.7.1, <2`) in a safe manner.
This will cause the bytes to always be cloned when converting to and from `bytes::Bytes`.

For zero-copy conversion between `SharedBytes` and `bytes::Bytes`, use the `bytes_1` feature instead - unless it is broken.

### http-body_04

Enables integration with the `http-body` crate (version `>=0.4.5, <0.5`).
The trait `http_body::Body` is then implemented for `ByteData` and `SharedBytes` (if `alloc` feature is used).

Since `http_body::Body` is the trait reexported as `hyper::HttpBody` in the `hyper` crate, this feature by extension also enables integration with `hyper`.

### http-body_1

Enables integration with the `http-body` crate (version `>=1.0.0, <2`).
The trait `http_body::Body` is then implemented for `ByteData` and `SharedBytes` (if `alloc` feature is used).

### queue

Enables the `ByteQueue`/`StringQueue` types which are queues of `ByteData`/`StringData` objects that can be pushed to and popped from.
Unless the `alloc` feature is enabled, the queue will be limited to a maximum size of 8 elements.

### nom_7

Enables integration with the `nom` crate (version `>=7, <8`).
This allows for `ByteData`, `StringData`, `ByteQueue`, and `StringQueue` data to be parsed using `nom` parsers.
