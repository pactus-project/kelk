//!
//!
use super::codec::Codec;
use super::error::Error;
use super::Offset;
use super::Storage;

///
pub struct Allocated<T: Codec> {
    offset: Offset,
    data: T,
}

///
pub enum LazyAllocated<'a, T: Codec> {
    ///
    Offset((Offset, &'a Storage)),
    ///
    Allocated(Allocated<T>),
}

impl<'a, T: Codec> LazyAllocated<'a, T> {
    pub(crate) fn from_allocated(allocated: Allocated<T>) -> Self {
        Self::Allocated(allocated)
    }

    pub(crate) fn from_offset(offset: Offset, storage: &'a Storage) -> Self {
        Self::Offset((offset, storage))
    }

    fn read(&mut self) -> Result<(), Error> {
        if let LazyAllocated::Offset((offset, storage)) = self {
            *self = LazyAllocated::Allocated(storage.read(*offset)?);
        }
        Ok(())
    }

    pub(crate) fn get(&mut self) -> Result<&Allocated<T>, Error> {
        self.read()?;
        if let LazyAllocated::Allocated(allocated) = self {
            Ok(allocated)
        } else {
            unreachable!()
        }
    }

    pub(crate) fn get_mut(&mut self) -> Result<&mut Allocated<T>, Error> {
        self.read()?;
        if let LazyAllocated::Allocated(allocated) = self {
            Ok(allocated)
        } else {
            unreachable!()
        }
    }
}

impl<T: Codec> Allocated<T> {
    ///
    pub fn new(offset: Offset, data: T) -> Self {
        Allocated { offset, data }
    }
    ///
    pub fn offset(&self) -> Offset {
        self.offset
    }

    ///
    pub fn data(&self) -> &T {
        &self.data
    }

    ///
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}
