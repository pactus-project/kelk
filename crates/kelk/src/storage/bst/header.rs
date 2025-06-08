use crate::{
    Codec,
    storage::{Offset, codec::Codec},
};

#[derive(Codec)]
pub(super) struct Header {
    // Number of items in the `StorageBST`, only really used by len().
    pub items: u32,
    // How many bytes is the key when it is packed with the `Codec`.
    pub key_len: u16,
    // How many bytes is the value when it is packed with the `Codec`.
    pub value_len: u16,
    // Offset of the root item in the storage file.
    // It set to zero when the `StorageBST` is empty.
    pub root_offset: Offset,
}

impl Header {
    pub fn new<K: Codec, V: Codec>() -> Self {
        Self {
            items: 0,
            key_len: K::PACKED_LEN as u16,
            value_len: V::PACKED_LEN as u16,
            root_offset: 0,
        }
    }
}
