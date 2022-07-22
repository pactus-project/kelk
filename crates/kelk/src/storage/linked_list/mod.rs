//! Storage Linked List
//!
//! Storage Linked List, is a singly linked list that instead of using Random Access Memory (RAM),
//! it uses storage file. Therefore it's permanently store inside contract's storage.
//!

use crate::storage::allocated::{Allocated, Offset};
use crate::storage::codec::Codec;
use crate::storage::error::Error;
use crate::storage::Storage;
use crate::Codec;
use core::iter::IntoIterator;
use core::marker::PhantomData;
use core::result::Result;

/// The instance of `StorageLinkedList`
pub struct StorageLinkedList<'a, T: Codec> {
    storage: &'a Storage,
    header: Allocated<Header>,
    _phantom: PhantomData<T>,
}

#[derive(Codec)]
pub(super) struct Node<T: Sized> {
    pub item: T,
    pub next: Offset,
}

impl<T: Sized> Node<T> {
    pub fn new(item: T) -> Self {
        Self { item, next: 0 }
    }
}

#[derive(Codec)]
pub(super) struct Header {
    pub count: u32,
    pub size_of_item: u16,
    pub head_offset: u32,
    pub tail_offset: u32,
}

impl Header {
    pub fn new<T: Codec>() -> Self {
        Self {
            count: 0,
            size_of_item: T::PACKED_LEN as u16,
            head_offset: 0,
            tail_offset: 0,
        }
    }
}

impl<'a, T: Codec> StorageLinkedList<'a, T> {
    /// Creates a new instance of `StorageLinkedList`.
    pub fn create(storage: &'a Storage) -> Result<Self, Error> {
        let header = storage.allocate(Header::new::<T>())?;
        storage.write(&header)?;

        Ok(StorageLinkedList {
            storage,
            header,
            _phantom: PhantomData,
        })
    }

    /// Loads the Storage Linked List at the given offset
    pub fn load(storage: &'a Storage, offset: Offset) -> Result<Self, Error> {
        let header = storage.read(offset)?;

        Ok(StorageLinkedList {
            storage,
            header,
            _phantom: PhantomData,
        })
    }

    /// Returns the offset of `StorageLinkedList` in the storage file.
    pub fn offset(&self) -> Offset {
        self.header.offset
    }

    /// Pushes an item at the end of linked list.
    pub fn push_back(&mut self, item: T) -> Result<(), Error> {
        let allocated_node = self.storage.allocate(Node::new(item))?;

        if self.header.data.count == 0 {
            self.header.data.head_offset = allocated_node.offset;
        } else {
            let mut tail: Allocated<Node<T>> = self.storage.read(self.header.data.tail_offset)?;
            tail.data.next = allocated_node.offset;
            self.storage.write(&tail)?;
        }
        self.storage.write(&allocated_node)?;

        self.header.data.count += 1;
        self.header.data.tail_offset = allocated_node.offset;
        self.storage.write(&self.header)
    }
}

pub struct StorageLinkedListIter<'a, T> {
    storage: &'a Storage,
    cur_offset: Offset,
    _phantom: PhantomData<T>,
}

impl<'a, T: Codec + 'a> Iterator for StorageLinkedListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_offset == 0 {
            None
        } else {
            let node: Allocated<Node<T>> = self.storage.read(self.cur_offset).unwrap();
            self.cur_offset = node.data.next;
            Some(&node.data.item)
        }
    }
}

impl<'a, T: Codec> IntoIterator for &'a mut StorageLinkedList<'a, T> {
    type Item = &'a T;
    type IntoIter = StorageLinkedListIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        let offset = self.header.data.head_offset;
        Self::IntoIter {
            storage: self.storage,
            cur_offset: offset,
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StorageLinkedList;
    use crate::storage::mock::mock_storage;
    use alloc::vec::Vec;

    #[test]
    fn test_push_back() {
        let storage = mock_storage(4 * 1024);
        let mut linked_list = StorageLinkedList::<i32>::create(&storage).unwrap();
        linked_list.push_back(1).unwrap();
        linked_list.push_back(2).unwrap();
        linked_list.push_back(3).unwrap();

        let iter = linked_list.into_iter();
        let all_items: Vec<i32> = iter.map(|n| *n).collect();
        assert!(all_items.eq(&[1, 2, 3]));
    }
}
