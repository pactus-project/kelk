use crate::storage::codec::Codec;
use crate::Codec;

#[derive(Codec)]
pub struct Node<I: Sized> {
    item: I,
    pub(super) next: u32,
}

impl<I: Sized> Node<I> {
    pub fn new(item: I) -> Self {
        Self { item, next: 0 }
    }

    pub fn item(&self) -> &I {
        &self.item
    }
}
