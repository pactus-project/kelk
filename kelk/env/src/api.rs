//! Defining the Kelk API trait.

use crate::error::Error;
use alloc::vec::Vec;

/// the storage APIs that should be provided by the host
pub trait KelkAPI {
    /// This API requests the host to read data from the storage file
    /// at the given `offset` up to the given `length`.
    fn read<'a>(&self, offset: u32, length: u32) -> Result<Vec<u8>, Error>;

    /// This API requests the host to write `data` into the storage file
    /// at the given `offset`
    fn write(&self, offset: u32, data: &[u8]) -> Result<(), Error>;

    /// This API requests the host to return the associated value to the given
    /// `param_id`.
    fn get_param<'a>(&self, param_id: u32) -> Result<Vec<u8>, Error>;
}
