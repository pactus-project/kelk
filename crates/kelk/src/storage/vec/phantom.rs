use crate::storage::codec::Codec;
use alloc::boxed::Box;
use core::marker::PhantomData;

/// A phantom vector that "act like" they are a vector of type `T` with size of `capacity`.
///
/// Using `PhantomVec` we can allocate storage without memory allocation.
pub struct PhantomVec<T: Codec, const N: usize> {
    _phantom: PhantomData<T>,
}

impl<T: Codec, const N: usize> PhantomVec<T, N> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: Codec, const N: usize> Codec for PhantomVec<T, N> {
    const PACKED_LEN: usize = N;

    #[inline]
    fn to_bytes(&self) -> Box<&[u8]> {
        unimplemented!()
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> Self {
        unimplemented!()
    }
}
