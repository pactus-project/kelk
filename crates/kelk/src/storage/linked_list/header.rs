use crate::storage::codec::Codec;
use crate::Codec;

#[derive(Codec)]
pub(super) struct Header {
    pub count: u32,
    pub size_of_item: u16,
    pub head_offset: u32,
    pub tail_offset: u32,
}

impl Header {
    pub fn new<I: Codec>() -> Self {
        Self {
            count: 0,
            size_of_item: I::PACKED_LEN as u16,
            head_offset: 0,
            tail_offset: 0,
        }
    }
}
