//! Storage trait to read and write primitives

use super::allocated::Allocated;
use super::codec::Codec;
use super::error::Error;
use super::Offset;
use alloc::boxed::Box;
use alloc::string::ToString;
use core::result::Result;
use kelk_env::StorageAPI;

macro_rules! impl_num {
    ($ty:ty, $size:literal, $read_fn:ident, $write_fn:ident) => {
        doc_comment! {
            concat!("reads ", stringify!($size), " byte(s) from storage file at the given `offset` and converts it to ", stringify!($ty),"."
            ),
            #[inline]
            pub fn $read_fn(&self, offset: u32) -> Result<Allocated<$ty>,Error> {
                self.read::<$ty>(offset)
            }
        }

        doc_comment! {
                concat!("converts ", stringify!($ty)," to ", stringify!($size), " byte(s) and writes it into storage file at the given `offset`."
                ),
            #[inline]
            pub fn $write_fn(&self, value: &Allocated<$ty>) -> Result<(),Error> {
                self.write(&value)
            }
        }
    };
}

/// Storage object
pub struct Storage {
    /// Storage APIs that are provided by the host
    api: Box<dyn StorageAPI>,

    stack_size: u16,
}

impl Storage {
    /// creates a new instance of storage
    pub fn create(api: Box<dyn StorageAPI>) -> Result<Self, Error> {
        api.write(0, &[1, 0])?; // version = 1
        api.write(2, &[0, 1])?; // stack size = 256
        api.write(4, &[0; 256 * 4])?; // stack
        api.write(1028, &[0, 0, 4, 8])?; // free storage pos

        let storage = Storage {
            api,
            stack_size: 256,
        };
        // let freed = StorageLinkedList::create(&storage, 0)?;
        // storage.freed = Some(freed);

        Ok(storage)
    }

    ///
    pub fn load(api: Box<dyn StorageAPI>) -> Result<Self, Error> {
        let ver = api.read(0, 2)?;
        let stack_size = api.read(2, 2)?;
        if !ver.eq(&[1, 0]) || !stack_size.eq(&[0, 1]) {
            return Err(Error::GenericError("invalid storage file".to_string()));
        }

        let storage = Storage {
            api,
            stack_size: 256,
        };

        Ok(storage)
    }

    pub(crate) fn api_mut(&mut self) -> &mut Box<dyn StorageAPI> {
        &mut self.api
    }

    ///
    pub fn allocate<T: Codec>(&self, data: T) -> Result<Allocated<T>, Error> {
        let mut free_pos = self.read_u32(1028)?;

        // Creating new allocation
        let allocated = Allocated::new(*free_pos.data(), data);

        // Updating allocation pos
        *free_pos.data_mut() += T::PACKED_LEN as u32;
        self.write_u32(&free_pos)?;

        Ok(allocated)
    }

    fn stack_offset(&self, stack_index: u16) -> Result<Offset, Error> {
        if stack_index > self.stack_size {
            return Err(Error::StackOverflow);
        }

        // stack_offset = (stack_index * 4) + 4
        let header_size = 4;
        Ok(((stack_index as usize * Offset::PACKED_LEN) + header_size) as Offset)
    }

    // pub(crate) fn fill_stack_at(&self, stack_index: u16, offset: Offset) -> Result<(), Error> {
    //     self.write_u32(self.stack_offset(stack_index)?, offset)
    // }

    // pub(crate) fn read_stack_at(&self, stack_index: u16) -> Result<Offset, Error> {
    //     self.read_u32(self.stack_offset(stack_index)?)
    // }

    impl_num!(u8, 1, read_u8, write_u8);
    impl_num!(u16, 2, read_u16, write_u16);
    impl_num!(u32, 4, read_u32, write_u32);
    impl_num!(u64, 8, read_u64, write_u64);
    impl_num!(u128, 8, read_u128, write_u128);

    impl_num!(i8, 1, read_i8, write_i8);
    impl_num!(i16, 2, read_i16, write_i16);
    impl_num!(i32, 4, read_i32, write_i32);
    impl_num!(i64, 8, read_i64, write_i64);
    impl_num!(i128, 16, read_i128, write_i128);

    impl_num!(bool, 1, read_bool, write_bool);

    /// reads `T` from the storage file at the given `offset`.
    /// Note that struct `T` should be `Codec`.
    #[inline]
    pub(crate) fn read<T: Codec>(&self, offset: u32) -> Result<Allocated<T>, Error> {
        let bytes = self.api.read(offset, T::PACKED_LEN as u32)?;
        let data = T::from_bytes(&bytes);
        Ok(Allocated::new(offset, data))
    }

    /// writes `T` to the storage file at the given `offset`.
    /// Note that `T` should be `Codec`.
    #[inline]
    pub(crate) fn write<T: Codec>(&self, allocated: &Allocated<T>) -> Result<(), Error> {
        let data = allocated.data().to_bytes();
        Ok(self.api.write(allocated.offset(), &data)?)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::storage::{allocated::Allocated, mock::mock_storage};

    #[test]
    fn test_negative_integers() {
        let mock = mock_storage(1024 * 1024);

        let a1: Allocated<i8> = mock.allocate(1).unwrap();
        let a2: Allocated<i16> = mock.allocate(2).unwrap();
        let a3: Allocated<i32> = mock.allocate(3).unwrap();
        let a4: Allocated<i64> = mock.allocate(4).unwrap();
        let a5: Allocated<i128> = mock.allocate(5).unwrap();

        mock.write_i8(&a1).unwrap();
        mock.write_i16(&a2).unwrap();
        mock.write_i32(&a3).unwrap();
        mock.write_i64(&a4).unwrap();
        mock.write_i128(&a5).unwrap();

        assert_eq!(mock.read_i8(a1.offset()).unwrap().data(), &1);
        assert_eq!(mock.read_i8(a2.offset()).unwrap().data(), &2);
        assert_eq!(mock.read_i8(a3.offset()).unwrap().data(), &3);
        assert_eq!(mock.read_i8(a4.offset()).unwrap().data(), &4);
        assert_eq!(mock.read_i8(a5.offset()).unwrap().data(), &5);
    }

    #[test]
    fn test_unsigned_integers() {
        let mock = mock_storage(15);

        let a1: Allocated<u8> = mock.allocate(1).unwrap();
        let a2: Allocated<u16> = mock.allocate(2).unwrap();
        let a3: Allocated<u32> = mock.allocate(3).unwrap();
        let a4: Allocated<u64> = mock.allocate(4).unwrap();
        let a5: Allocated<u128> = mock.allocate(5).unwrap();

        mock.write_u8(&a1).unwrap();
        mock.write_u16(&a2).unwrap();
        mock.write_u32(&a3).unwrap();
        mock.write_u64(&a4).unwrap();
        mock.write_u128(&a5).unwrap();

        assert_eq!(mock.read_u8(a1.offset()).unwrap().data(), &1);
        assert_eq!(mock.read_u8(a2.offset()).unwrap().data(), &2);
        assert_eq!(mock.read_u8(a3.offset()).unwrap().data(), &3);
        assert_eq!(mock.read_u8(a4.offset()).unwrap().data(), &4);
        assert_eq!(mock.read_u8(a5.offset()).unwrap().data(), &5);
    }

    // #[test]
    // fn test_bool() {
    //     let mock = mock_storage(1);

    //     mock.write_bool(0, true).unwrap();
    //     assert!(mock.read_bool(0).unwrap());
    // }

    // #[test]
    // fn test_struct() {
    //     #[derive(Debug, PartialEq)]
    //     struct Test {
    //         foo: i16,
    //         bar: i8,
    //         zoo: i32,
    //     }

    //     let storage = mock_storage(64);
    //     let foo_1 = Test {
    //         foo: 123,
    //         bar: 7,
    //         zoo: 1024,
    //     };

    //     storage.write_struct::<Test>(13, &foo_1).unwrap();
    //     let foo_2 = storage.read_struct::<Test>(13).unwrap();
    //     assert_eq!(foo_1, foo_2);
    // }

    // #[test]
    // fn test_sting() {
    //     let storage = mock_storage(64);
    //     storage.write_string(0, "foooo", 16).unwrap();
    //     storage.write_string(0, "foo", 16).unwrap();
    //     let expected = storage.read_string(0, 16).unwrap();
    //     assert_eq!(expected, "foo".to_string())
    // }
}
