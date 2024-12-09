#[cfg(feature = "alloc")]
mod shared_bytes;
#[cfg(feature = "alloc")]
mod shared_bytes_builder;

mod stringdata;

#[cfg(feature = "macros")]
mod macros;

#[cfg(all(feature = "queue", feature = "alloc"))]
mod queue;

#[cfg(feature = "bytes_1")]
mod bytes_1;
