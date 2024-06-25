//! # Queue
//!
//! This module contains the queue data structure and its iterators.
//!
//! The queue is a list of byte slices, which allows for efficient appending and consuming of byte data.

mod byte_queue;
mod string_queue;

mod byte_iter;
mod char_iter;
mod chunk_iter;

mod linked_iter;
mod linked_node_data;
#[cfg(feature = "alloc")]
mod linked_node_leaf;
mod linked_root;

pub use byte_iter::ByteIter;
pub use byte_queue::ByteQueue;
pub use char_iter::CharIter;
pub use chunk_iter::{ChunkIter, StrChunkIter};
pub use linked_iter::LinkedIter;
pub use string_queue::StringQueue;
