//! Modules definition for storage libraries

pub mod bst;
pub mod codec;
pub mod error;
pub mod linked_list;
pub mod mock;
pub mod storage;
pub mod vec;
pub mod str;

pub use storage::Storage;

/// is an alias for representing the offset of the allocated space inside the storage file.
pub type Offset = u32;
