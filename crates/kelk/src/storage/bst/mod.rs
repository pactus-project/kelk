//! Storage Binary Search Tree
//!
//! Storage Binary Search Tree, is a Binary Search Tree or BST that instead of using Random Access Memory (RAM),
//! it uses storage file. Therefore it's permanently stored inside contract's storage.

mod header;
mod node;

use self::header::Header;
use self::node::Node;
use crate::storage::codec::Codec;
use crate::storage::error::Error;
use crate::storage::{Offset, Storage};
use core::marker::PhantomData;
use core::result::Result;

/// The instance of Storage Binary Search Tree
pub struct StorageBST<'a, K, V>
where
    K: Codec + Ord,
    V: Codec,
{
    storage: &'a Storage,
    // Offset of the header in the storage file.
    header_offset: Offset,
    // In memory instance of the header.
    // Any change in the header should be flushed into the storage file
    header: Header,
    _phantom: PhantomData<(K, V)>,
}

impl<'a, K, V> StorageBST<'a, K, V>
where
    K: Codec + Ord,
    V: Codec,
{
    /// Creates a new instance of `StorageBST`.
    pub fn create(storage: &'a Storage) -> Result<Self, Error> {
        let header_offset = storage.allocate(Header::PACKED_LEN)?;
        let header = Header::new::<K, V>();
        storage.write(header_offset, &header)?;

        Ok(StorageBST {
            storage,
            header_offset,
            header,
            _phantom: PhantomData,
        })
    }

    /// Try to load the `StorageBST` at the given offset in the storage file.
    pub fn load(storage: &'a Storage, offset: Offset) -> Result<Self, Error> {
        let header: Header = storage.read(offset)?;

        debug_assert_eq!(header.key_len, K::PACKED_LEN as u16);
        debug_assert_eq!(header.value_len, V::PACKED_LEN as u16);

        Ok(StorageBST {
            storage,
            header_offset: offset,
            header,
            _phantom: PhantomData,
        })
    }

    /// Returns the offset of `StorageBST` in the storage file.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn offset(&self) -> Offset {
        self.header_offset
    }

    /// Returns the number of elements in the `StorageBST`.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn len(&self) -> u32 {
        self.header.items
    }

    /// Returns `true` if the `StorageBST` contains no elements.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Removes a key from the `StorageBST`, returning the value at the key if the key was previously in the `StorageBST`.
    pub fn remove(&mut self, key: &K) -> Result<Option<V>, Error> {
        if self.header.items == 0 {
            return Ok(None);
        }

        let mut offset = self.header.root_offset;
        let mut parent_offset: Option<Offset> = None;
        let mut node: Node<K, V> = self.storage.read(offset)?;
        let mut is_right = false;

        while !node.key.eq(key) {
            parent_offset = Some(offset);
            if node.key.gt(key) {
                is_right = true;
                offset = node.right;
            } else if node.key.lt(key) {
                is_right = false;
                offset = node.left;
            } else {
                break;
            }

            if offset.eq(&0) {
                return Ok(None);
            }

            node = self.storage.read(offset)?;
        }
        let result = node.value;

        // Case 1: If node to be deleted has only one child
        if node.left.eq(&0) {
            offset = node.right;
        } else if node.right.eq(&0) {
            offset = node.left;
        } else {
            // Case 2: If node to be deleted has two children
            // Get the inorder successor (smallest in the right subtree)
            let mut min_val = node.right;
            let mut min_node: Node<K, V> = self.storage.read(min_val)?;
            while !min_node.left.eq(&0) {
                min_val = min_node.left;
                min_node = self.storage.read(min_val)?;
            }

            // Copy the inorder successor's content to this node
            node.key = min_node.key;
            node.value = min_node.value;
            // Delete the inorder successor
            offset = min_node.right;
            parent_offset = Some(min_val);
            is_right = false;
        }

        // Update parent's child
        if let Some(parent) = parent_offset {
            let mut parent_node: Node<K, V> = self.storage.read(parent)?;
            if is_right {
                parent_node.left = offset;
            } else {
                parent_node.right = offset;
            }
            self.storage.write(parent, &parent_node)?;
        } else {
            self.header.root_offset = offset;
        }

        //self.storage.deallocate(..)?; // TODO: complete me
        self.header.items -= 1;
        self.storage.write(self.header_offset, &self.header)?;

        Ok(Some(result))
    }

    /// Inserts a key-value pair into the tree.
    /// If the `StorageBST` did not have this key present, None is returned.
    /// If the `StorageBST` did have this key present, the value is updated, and the old value is returned.
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, Error> {
        if self.header.items == 0 {
            // create a root node
            let offset = self.storage.allocate(Node::<K, V>::PACKED_LEN)?;
            let root = Node::new(key, value);

            self.header.items = 1;
            self.header.root_offset = offset;

            self.storage.write(self.header_offset, &self.header)?;
            self.storage.write(offset, &root)?;
            Ok(None)
        } else {
            let mut offset = self.header.root_offset;
            let mut node: Node<K, V> = self.storage.read(offset)?;

            loop {
                // if the node's key is less then the key
                if node.key.lt(&key) {
                    if node.left.eq(&0) {
                        let new_offset = self.storage.allocate(Node::<K, V>::PACKED_LEN)?;
                        let new_node = Node::new(key, value);

                        // update header
                        self.header.items += 1;
                        self.storage.write(self.header_offset, &self.header)?;

                        // update parent node
                        node.left = new_offset;
                        self.storage.write(offset, &node)?;

                        // write new node
                        self.storage.write(new_offset, &new_node)?;
                        return Ok(None);
                    }
                    offset = node.left;
                }
                // if the node's key is greater then the key
                else if node.key.gt(&key) {
                    if node.right.eq(&0) {
                        let new_offset = self.storage.allocate(Node::<K, V>::PACKED_LEN)?;
                        let new_node = Node::new(key, value);

                        // update header
                        self.header.items += 1;
                        self.storage.write(self.header_offset, &self.header)?;

                        // update parent node
                        node.right = new_offset;
                        self.storage.write(offset, &node)?;

                        // write new node
                        self.storage.write(new_offset, &new_node)?;
                        return Ok(None);
                    }
                    offset = node.right;
                }
                // if the node's key is equal with the key
                else {
                    let old_value = node.value;
                    node.value = value;

                    // node exists, update value
                    self.storage.write(offset, &node)?;
                    return Ok(Some(old_value));
                }
                node = self.storage.read(offset)?;
            }
        }
    }

    /// Finds the value corresponding to the key in the `StorageBST` .
    /// If the key is found, the value is returned. If the key is not found, `None` is returned.
    pub fn find(&self, key: &K) -> Result<Option<V>, Error> {
        if self.header.items == 0 {
            return Ok(None);
        }

        let mut offset = self.header.root_offset;
        let mut node: Node<K, V> = self.storage.read(offset)?;

        loop {
            // if the node's key is less then the key
            if node.key.lt(key) {
                if node.left.eq(&0) {
                    return Ok(None);
                }
                offset = node.left;
            }
            // if the node's key is greater then the key
            else if node.key.gt(key) {
                if node.right.eq(&0) {
                    return Ok(None);
                }
                offset = node.right;
            }
            // if the node's key is equal with the key
            else {
                return Ok(Some(node.value));
            }
            node = self.storage.read(offset)?;
        }
    }

    /// Returns true if the tree contains a value for the specified key.
    pub fn contains_key(&self, key: &K) -> Result<bool, Error> {
        Ok(self.find(key)?.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::mock::mock_storage;

    #[test]
    fn test_bst() {
        let storage = mock_storage(1024);
        let mut bst_1 = StorageBST::<i32, i64>::create(&storage).unwrap();

        assert!(bst_1.is_empty());
        assert_eq!(None, bst_1.insert(1, 10).unwrap());
        assert_eq!(None, bst_1.insert(3, 30).unwrap());
        assert_eq!(None, bst_1.insert(2, 20).unwrap());
        assert_eq!(Some(10), bst_1.insert(1, 100).unwrap());

        let bst_2 = StorageBST::<i32, i64>::load(&storage, bst_1.offset()).unwrap();
        assert_eq!(3, bst_2.len());
        assert_eq!(Some(20), bst_2.find(&2).unwrap());
        assert_eq!(None, bst_2.find(&4).unwrap());
        assert_eq!(Some(30), bst_2.find(&3).unwrap());
        assert_eq!(Some(100), bst_2.find(&1).unwrap());

        let bst_3 = StorageBST::<i32, i64>::load(&storage, bst_2.offset()).unwrap();
        assert!(!bst_3.contains_key(&-1).unwrap());
        assert!(bst_3.contains_key(&2).unwrap());
        assert!(!bst_3.contains_key(&4).unwrap());
    }

    #[test]
    fn test_remove() {
        let storage = mock_storage(1024);
        let mut bst_1 = StorageBST::<i32, i64>::create(&storage).unwrap();

        // insert some key-value pairs
        assert_eq!(None, bst_1.insert(1, 10).unwrap());
        assert_eq!(None, bst_1.insert(3, 30).unwrap());
        assert_eq!(None, bst_1.insert(2, 20).unwrap());

        // remove a key-value pair
        assert_eq!(Some(10), bst_1.remove(&1).unwrap());
        assert_eq!(None, bst_1.find(&1).unwrap());
        assert_eq!(2, bst_1.len());

        // remove a key-value pair that doesn't exist
        assert_eq!(None, bst_1.remove(&5).unwrap());

        // remove all key-value pairs
        assert_eq!(Some(30), bst_1.remove(&3).unwrap());
        assert_eq!(Some(20), bst_1.remove(&2).unwrap());

        let bst_2 = StorageBST::<i32, i64>::load(&storage, bst_1.offset()).unwrap();
        assert_eq!(0, bst_2.len());
        assert!(bst_2.is_empty());
    }
}
