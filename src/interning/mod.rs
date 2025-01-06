//! # Interning
//!
//! This module contains types to handle interning of (byte)strings.
//! 
//! The interning is done by storing the strings in a hash map, which allows for efficient storage and retrieval of strings.

mod byte_interning;
pub use byte_interning::ByteInterning;

mod r#static;
pub use r#static::StaticInterning;

//mod releasing;
//pub use releasing::ReleasingInterning;
