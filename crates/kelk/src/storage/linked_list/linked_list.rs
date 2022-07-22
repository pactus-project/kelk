//! Storage Linked List, is a singly linked list that instead of using Random Access Memory (RAM),
//! it uses storage file. Therefore it's permanently store inside contract's storage.

use super::header::Header;
use super::node::{self, Node};
use crate::storage::allocated::{self, Allocated, LazyAllocated};
use crate::storage::codec::Codec;
use crate::storage::error::Error;
use crate::storage::{Offset, Storage};
use alloc::collections::btree_map::Entry::{Occupied, Vacant};
use alloc::collections::{BTreeMap, BTreeSet};
use core::iter::IntoIterator;
use core::marker::PhantomData;
use core::result::Result;

/// The instance of Storage Linked List
pub struct StorageLinkedList<'a, I: Codec> {
    storage: &'a Storage,
    header: LazyAllocated<'a, Header>,
    _phantom: PhantomData<I>,
}

impl<'a, I: Codec> StorageLinkedList<'a, I> {
    /// creates a new instance of Storage Linked List.
    pub fn create(storage: &'a Storage) -> Result<Allocated<Self>, Error> {
        let header = Header::new::<I>();
        let allocated_header = storage.allocate(header)?;

        Ok(Allocated::new(
            allocated_header.offset(),
            StorageLinkedList {
                storage,
                header: LazyAllocated::from_allocated(allocated_header),
                _phantom: PhantomData,
            },
        ))
    }

    /// load the Storage Linked List
    pub fn lazy_load(storage: &'a Storage, offset: Offset) -> Result<Allocated<Self>, Error> {
        Ok(Allocated::new(
            offset,
            StorageLinkedList {
                storage,
                header: LazyAllocated::from_offset(offset, storage),
                _phantom: PhantomData,
            },
        ))
    }

    /// pushes an item at the end of linked list.
    pub fn push_back(&mut self, item: I) -> Result<(), Error> {
        let allocated = self.storage.allocate(Node::new(item))?;
        let header = self.header.get_mut()?.data_mut();

        if header.count == 0 {
            header.head_offset = allocated.offset();
        } else {
            let mut tail: Allocated<Node<I>> = self.storage.read(header.tail_offset)?;
            tail.data_mut().next = allocated.offset();
            self.storage.write(&tail)?;
        }

        header.count += 1;
        header.tail_offset = allocated.offset();

        self.storage.write(&allocated)?;
        self.storage.write(self.header.get()?)?;

        Ok(())
    }
}

pub struct StorageLinkedListIter<'a, I> {
    storage: &'a Storage,
    cur_offset: Offset,
    _phantom: PhantomData<I>,
}

impl<'a, I: Codec> Iterator for StorageLinkedListIter<'a, I> {
    type Item = Allocated<Node<I>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_offset == 0 {
            None
        } else {
            let node: Allocated<Node<I>> = self.storage.read(self.cur_offset).unwrap();
            self.cur_offset = node.data().next;
            Some(node)
        }
    }
}

impl<'a, I: Codec> IntoIterator for &'a mut StorageLinkedList<'a, I> {
    type Item = Allocated<Node<I>>;
    type IntoIter = StorageLinkedListIter<'a, I>;

    fn into_iter(self) -> Self::IntoIter {
        let offset = self.header.get().unwrap().data().head_offset;
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
    use core::iter::IntoIterator;

    #[test]
    fn test_push_back() {
        let storage = mock_storage(4 * 1024);
        let mut linked_list = StorageLinkedList::<i32>::create(&storage).unwrap();
        linked_list.data_mut().push_back(1).unwrap();
        linked_list.data_mut().push_back(2).unwrap();
        linked_list.data_mut().push_back(3).unwrap();

        let mut iter = linked_list.data_mut().into_iter();
        let a = iter.next().unwrap();
        let b = iter.next().unwrap();
    }
}
