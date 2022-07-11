//! Imported WASM functions
//!
//! Contract actors can call this imported function to interact with the
//! blockchain and the storage file.

use crate::alloc::vec::Vec;
use crate::api::KelkAPI;
use crate::error::Error;

#[cfg(not(test))]
#[link(wasm_import_module = "zarb")]
extern "C" {
    /// writes data at given offset of storage file.
    ///
    /// # Arguments
    ///
    /// `offset` is the offset of data in the storage file.
    /// `ptr` is the location in sandbox memory where data should be read from.
    /// `len` is the length of data.
    ///
    /// If the operation is successful it returns 0, otherwise it reruns the error code.
    fn write_storage(offset: u32, ptr: u32, len: u32) -> i32;
    /// reads data from the given offset of storage file.
    ///
    /// # Arguments
    ///
    /// `offset` is the offset of data in the storage file.
    /// `ptr` is the location in sandbox memory where data should be written to.
    /// `len` is the length of data.
    ///
    /// If the operation is successful it returns 0, otherwise it reruns the error code.
    fn read_storage(offset: u32, ptr: u32, len: u32) -> i32;
    /// gets parameter value from the host.
    ///
    /// # Arguments
    ///
    /// `param_id` is the parameter ID that is known for the host.
    /// `ptr` is the location in sandbox memory where data should be written to.
    /// `len` is the length of data.
    ///
    /// If the operation is successful it returns 0, otherwise it reruns the error code.
    fn get_param(param_id: u32, ptr: u32, len: u32) -> i32;
}

pub(crate) struct Kelk {}


impl KelkAPI for Kelk {
    fn write(&self, offset: u32, data: &[u8]) -> Result<(), Error> {
        let ptr = data.as_ptr() as u32;
        let len = data.len() as u32;

        let code = unsafe { write_storage(offset, ptr, len) };
        if code != 0 {
            return Err(Error::HostError(code));
        }
        Ok(())
    }

    fn read<'a>(&self, offset: u32, len: u32) -> Result<Vec<u8>, Error> {
        let vec = crate::alloc::vec![0; len as usize];
        let ptr = vec.as_ptr() as u32;

        let code = unsafe { read_storage(offset, ptr, len) };
        if code != 0 {
            return Err(Error::HostError(code));
        }
        Ok(vec.to_vec())
    }

    fn get_param<'a>(&self, param_id: u32) -> Result<Vec<u8>, Error> {
        let len = 32; // maximum size of parameter value is 32 bytes
        let vec = crate::alloc::vec![0; len as usize];
        let ptr = vec.as_ptr() as u32;

        let code = unsafe { get_param(param_id, ptr, len) };
        if code != 0 {
            return Err(Error::HostError(code));
        }
        Ok(vec.to_vec())
    }
}

/// For testing
#[cfg(test)]
pub unsafe fn write_storage(_offset: u32, _ptr: u32, _len: u32) -> i32 {
    0
}

/// For testing
#[cfg(test)]
pub unsafe fn read_storage(_offset: u32, _ptr: u32, _len: u32) -> i32 {
    0
}


/// For testing
#[cfg(test)]
pub unsafe fn get_param(_param_id: u32, _ptr: u32, _len: u32) -> i32 {
    0
}
