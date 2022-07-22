use crate::storage::codec::Codec;
use crate::Codec;
use core::mem::size_of;

#[derive(Codec)]
pub(super) struct Header {
    pub count: u32,
    pub size_of_item: u16,
    pub head_offset: u32,
    pub tail_offset: u32,
}

impl Header {
    pub fn new<I: Sized>() -> Self {
        Self {
            count: 0,
            size_of_item: size_of::<I>() as u16,
            head_offset: 0,
            tail_offset: 0,
        }
    }
}
