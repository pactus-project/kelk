use crate::storage::Offset;
use crate::storage::codec::Codec;
use crate::Codec;

#[derive(Codec)]
pub struct Node<I: Sized> {
    item: I,
    next: Offset,
}

impl<I: Sized> Node<I> {
    pub fn new(item: I) -> Self {
        Self { item, next: 0 }
    }

    #[inline]
    pub fn item(&self) -> &I {
        &self.item
    }

    #[inline]
    pub fn next(&self) -> &Offset {
        &self.next
    }

    #[inline]
    pub fn update_next(&mut self, next: Offset) -> &Self{
        self.next = next;
        self
    }
}
