//! Storage String

use super::{Offset, Storage, error::Error, vec::StorageVec};
use alloc::string::{String, ToString};

/// Storage String
pub struct StorageString<'a> {
    // A vector of UTF-8 bytes,
    vec: StorageVec<'a, u8>,
}

impl<'a> StorageString<'a> {
    /// creates and store a new instance of `StorageString` at the given offset
    pub fn create(storage: &'a Storage, capacity: u32) -> Result<Self, Error> {
        let vec = StorageVec::create(storage, capacity)?;

        Ok(StorageString { vec })
    }

    /// load the `StorageString`
    pub fn load(storage: &'a Storage, offset: Offset) -> Result<Self, Error> {
        let vec = StorageVec::load(storage, offset)?;

        Ok(StorageString { vec })
    }
    /// Returns the offset of `StorageString` in the storage file.
    pub fn offset(&self) -> Offset {
        self.vec.offset()
    }


    /// Returns the length of the `StorageString`.
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Returns `true` if this `StorageString` has a length of zero, and `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Sets the contents of the `StorageString` to the provided string slice.
    pub fn set_string(&mut self, str: &str) -> Result<(), Error> {
        self.vec.set_bytes(str.as_bytes())
    }

    /// Returns the contents of the `StorageString` as a `String`.
    pub fn get_string(&self) -> Result<String, Error> {
        let bytes = self.vec.get_bytes()?;
        let str = String::from_utf8(bytes).map_err(|err| Error::GenericError(err.to_string()))?;
        Ok(str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::mock::mock_storage;

    #[test]
    fn test_string() {
        let storage = mock_storage(1024);
        let mut str_1 = StorageString::create(&storage, 16).unwrap();
        str_1.set_string("foo").unwrap();

        let str_2 = StorageString::load(&storage, str_1.offset()).unwrap();
        assert_eq!("foo", str_2.get_string().unwrap());
    }
}
