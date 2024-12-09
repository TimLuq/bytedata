mod bytedata;
mod stringdata;

#[cfg(feature = "alloc")]
mod shared_bytes;

#[cfg(feature = "queue")]
mod byte_queue;

#[cfg(feature = "queue")]
mod string_queue;
