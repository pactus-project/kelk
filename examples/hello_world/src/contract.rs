use crate::error::Error;
use kelk::alloc::string::String;
use kelk::context::Context;
use kelk::kelk_derive;
use kelk::storage::str::StorageString;

#[kelk_derive(instantiate)]
pub fn instantiate(ctx: Context, _: ()) -> Result<(), Error> {
    let mut storage_string = StorageString::create(ctx.storage, 64)?;
    ctx.storage.fill_stack_at(1, storage_string.offset())?;
    storage_string.set_string("hello world!")?;

    Ok(())
}

#[kelk_derive(process)]
pub fn process(ctx: Context, msg: String) -> Result<(), Error> {
    let storage_string_offset = ctx.storage.read_stack_at(1)?;
    let mut storage_string = StorageString::load(ctx.storage, storage_string_offset)?;
    storage_string.set_string(&msg)?;

    Ok(())
}

#[kelk_derive(query)]
pub fn query(ctx: Context, _: ()) -> Result<String, Error> {
    let storage_string_offset = ctx.storage.read_stack_at(1)?;
    let storage_string = StorageString::load(ctx.storage, storage_string_offset)?;
    let msg = storage_string.get_string()?;

    Ok(msg)
}

#[cfg(test)]
#[path = "./contract_test.rs"]
mod tests;
