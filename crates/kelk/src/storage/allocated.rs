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

    pub(crate) fn offset(&self) -> Offset {
        match self {
            Self::Allocated(allocated) => allocated.offset(),
            Self::Offset((offset, _)) => *offset,
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
    pub fn update_data<F>(&mut self, mut f: F)
    where
        F: FnMut(&T)->T,
    {
        self.data = f(&self.data);
    }

    ///
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}


#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_update() {
        let mut a = Allocated::<i32>{offset:1, data: 2};
        a.update_data(|x|x+1);
        assert_eq!(a.data(), &3);
        assert_eq!(a.offset(), 1);
    }
}