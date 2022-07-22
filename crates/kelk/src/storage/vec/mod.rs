//! Storage Vector
//!
//! Storage Vector, is a Vector or Array that instead of using Random Access Memory (RAM),
//! it uses storage file. Therefore it's permanently store inside contract's storage.
//!


use super::allocated::{Allocated, Offset};
use crate::storage::codec::Codec;
use crate::storage::error::Error;
use crate::storage::Storage;
use crate::Codec;
use core::marker::PhantomData;
use core::result::Result;

/// The instance of Storage Vector
pub struct StorageVec<'a, T: Codec> {
    storage: &'a Storage,
    header: Allocated<Header>,
    _phantom: PhantomData<T>,
}
#[derive(Codec)]
pub(super) struct Header {
    pub count: u32,
    pub capacity: u32,
    pub value_len: u16,
    pub data_offset: Offset,
}

impl Header {
    pub fn new<T: Codec>(capacity: u32) -> Self {
        Self {
            value_len: T::PACKED_LEN as u16,
            count: 0,
            capacity,
            data_offset: 0,
        }
    }
}

impl<'a, T: Codec> StorageVec<'a, T> {
    /// creates and store a new instance of Storage Vector at the given offset
    pub fn create(storage: &'a Storage, capacity: u32) -> Result<Self, Error> {
        let mut header = storage.allocate(Header::new::<T>(capacity))?;
        let a = PhantomVec::<T, capacity>::new();
        let mut phantom = storage.allocate(a);
        //storage.allocate(data)
        storage.write(&header)?;

        Ok(StorageVec {
            storage,
            header,
            _phantom: PhantomData,
        })
    }

    /// load the Storage Vector
    pub fn load(storage: &'a Storage, offset: u32) -> Result<Self, Error> {
        let header = storage.read(offset)?;

        Ok(StorageVec {
            storage,
            header,
            _phantom: PhantomData,
        })
    }
    /// Returns the offset of `StorageVector` in the storage file.
    pub fn offset(&self) -> Offset {
        self.header.offset
    }

    /// Returns the number of elements in the vector, also referred to as its ‘length’.
    pub fn len(&self) -> u32 {
        self.get_header().unwrap().count // TODO?
    }

    /// Returns true if the vector contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Appends an element to the back of a vector.
    pub fn push(&mut self, value: T) -> Result<(), Error> {
        let header = self.get_header()?;
        if header.count >= header.capacity {
            return Err(Error::OutOfCapacity);
        }

        let offset = self.get_item_offset(header.count)?;
        let item = Allocated::new(offset, value);
        self.storage.write(&item)?;

        header.count += 1;
        self.update_header()
    }

    /// Returns an element at the given index or None if out of bounds..
    pub fn get(&self, index: u32) -> Result<Option<T>, Error> {
        let header = self.header.get()?.data;
        if index >= header.count {
            return Ok(None);
        }

        let offset = self.get_item_offset(index)?;
        let item = self.storage.read(offset)?;
        Ok(Some(item.data))
    }

    fn get_item_offset(&self, index: u32) -> Result<Offset, Error> {
        Ok(self.offset()
            + Header::PACKED_LEN as u32
            + (index * self.get_header()?.value_len as u32))
    }

    fn get_header(&self) -> Result<Header, Error> {
        Ok(self.header.get_mut()?.data)
    }

    fn update_header(&self) -> Result<(), Error> {
        self.storage.write(self.header.get()?)
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::mock::mock_storage;

    use super::*;
    use core::mem::size_of;

    #[test]
    fn test_size() {
        assert_eq!(16, size_of::<Header>());
    }

    #[test]
    fn test_header() {
        let storage = mock_storage(1024);
        StorageVec::<i32>::create(&storage, 512, 16).unwrap();
        let header: Header = storage.read_struct(512).unwrap();
        assert_eq!(header.boom, 0xb3000000);
        assert_eq!(header.reserved, 0);
        assert_eq!(header.value_len, 4);
        assert_eq!(header.count, 0);
        assert_eq!(header.capacity, 16);
    }

    #[test]
    fn test_vector() {
        let storage = mock_storage(1024);
        let mut vec = StorageVec::<i32>::create(&storage, 512, 16).unwrap();
        assert_eq!(None, vec.get(0).unwrap());
        assert!(vec.is_empty());

        vec.push(10).unwrap();
        vec.push(11).unwrap();
        vec.push(12).unwrap();

        assert_eq!(3, vec.len());
        assert_eq!(Some(10), vec.get(0).unwrap());
        assert_eq!(Some(11), vec.get(1).unwrap());
        assert_eq!(Some(12), vec.get(2).unwrap());
        assert_eq!(None, vec.get(3).unwrap());
    }

    #[test]
    fn test_load() {
        let storage = mock_storage(1024);
        let mut vec = StorageVec::<i32>::create(&storage, 512, 128).unwrap();
        vec.push(1).unwrap();

        let vec = StorageVec::<i32>::lazy_load(&storage, 512).unwrap();
        let header: Header = storage.read_struct(512).unwrap();
        assert_eq!(header.boom, 0xb3000000);
        assert_eq!(header.reserved, 0);
        assert_eq!(header.value_len, 4);
        assert_eq!(header.count, 1);
        assert_eq!(header.capacity, 128);
        assert_eq!(Some(1), vec.get(0).unwrap());
    }

    #[test]
    fn test_capacity() {
        let storage = mock_storage(1024);
        let mut vec = StorageVec::<i32>::create(&storage, 0, 4).unwrap();

        vec.push(1).unwrap();
        vec.push(2).unwrap();
        vec.push(3).unwrap();
        vec.push(4).unwrap();
        assert!(vec.push(5).is_err());
    }
}
