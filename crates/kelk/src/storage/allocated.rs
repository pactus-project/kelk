//!
//!

use super::codec::Codec;

/// is an alias for representing the offset of the allocated space inside the storage file.
pub type Offset = u32;

///
#[derive(Debug)]
pub(crate) struct Allocated<T: Codec> {
    pub offset: Offset,
    pub data: T,
}

impl<T: Codec> Allocated<T> {
    ///
    pub fn new(offset: Offset, data: T) -> Self {
        Allocated { offset, data }
    }
}

