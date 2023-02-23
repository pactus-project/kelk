use core::borrow::{Borrow, BorrowMut};
use core::cell::{Ref, RefCell};

use super::error::Error;
use super::Offset;

use alloc::boxed::Box;

use alloc::rc::Rc;
use kelk_env::{BlockchainAPI, StorageAPI};

#[derive(Debug, Clone)]
pub(self) struct Deallocated {
    pub offset: Offset,
    pub freed_offset: Offset,
    pub freed_length: u32,
    pub prev: Option<Box<Deallocated>>,
    pub next: Option<Box<Deallocated>>,
    pub updated: bool,
}

pub(self) struct Allocator {
    allocation_offset: Offset,
    deallocated_head: Option<Box<Deallocated>>,
}

impl Allocator {
    pub fn create(api: &dyn StorageAPI, offset: &Offset) -> Result<Self, Error> {
        let allocation_offset = offset + 8;
        let data: &[u8; 4] = unsafe { core::mem::transmute(&allocation_offset) };
        api.write(*offset, data)?; // allocation offset
        api.write(*offset + 4, &[0; 4])?; // deallocation offset

        Ok(Self {
            allocation_offset,
            deallocated_head: None,
        })
    }

    pub fn load(api: &dyn StorageAPI, offset: &Offset) -> Result<Self, Error> {
        let mut data: [u8; 8] = [0; 8];
        api.read(*offset, data.as_mut_slice())?;

        let allocation_offset = unsafe { *(data[0..4].as_ptr() as *const Offset) };
        let deallocated_head_offset = unsafe { *(data[4..8].as_ptr() as *const Offset) };

        let mut prv_deallocated: Option<Box<Deallocated>> = None;
        let mut deallocated = None;

        let mut offset = deallocated_head_offset;
        while offset != 0 {
            let (freed_offset, freed_length, next_offset) =
                Allocator::read_deallocated(api, &offset)?;
            let cur_deallocated = Some(Box::new(Deallocated {
                offset,
                freed_offset,
                freed_length,
                prev: prv_deallocated.clone(),
                next: None,
                updated: false,
            }));

            if deallocated.is_none() {
                deallocated = cur_deallocated.clone();
            }

            match prv_deallocated.as_mut() {
                Some(mut item) => {
                    let mut b = item.clone();
                    let c = &mut *b.clone();
                    let mut d = &mut *b;
                    d.next = cur_deallocated;
                }
                None => {
                    prv_deallocated = cur_deallocated;
                }
            };

            offset = next_offset;
        }

        Ok(Self {
            allocation_offset,
            deallocated_head: deallocated,
        })
    }

    pub fn allocate(&mut self, api: &dyn StorageAPI, length: u32) -> Result<Offset, Error> {
        let mut deallocated_opt = self.deallocated_head.clone();
        while let Some(mut deallocated) = deallocated_opt {
            if deallocated.freed_length >= length {
                if let Some(prv_deallocated) = deallocated.prev.as_mut() {
                    prv_deallocated.next = deallocated.next.clone();
                    prv_deallocated.updated = true;
                }

                if let Some(mut nxt_deallocated) = deallocated.clone().next {
                    nxt_deallocated.prev = deallocated.prev.clone();
                }

                if deallocated.freed_length == length {
                    self.deallocate(api, deallocated.offset, Self::size_of_deallocated_item())?;
                } else {
                    deallocated.freed_length = deallocated.freed_length - length;
                    self.deallocate_item(api, deallocated.clone())?;
                }

                return Ok(deallocated.freed_offset);
            }

            deallocated_opt = deallocated.next.clone();
        }

        let cur_free_pos = self.allocation_offset;
        self.allocation_offset += length;

        // Updating allocation pos
        let data: &[u8; 4] = unsafe { core::mem::transmute(&self.allocation_offset) };
        api.write(self.allocation_offset, data)?;

        Ok(cur_free_pos)
    }

    pub fn deallocate(
        &mut self,
        api: &dyn StorageAPI,
        offset: Offset,
        length: u32,
    ) -> Result<(), Error> {
        let item = Box::new(Deallocated {
            offset: self.allocation_offset,
            freed_offset: offset,
            freed_length: length,
            prev: None,
            next: None,
            updated: true,
        });

        self.allocation_offset += Self::size_of_deallocated_item();

        self.deallocate_item(api, item)
    }

    pub fn deallocate_item(
        &mut self,
        api: &dyn StorageAPI,
        mut item: Box<Deallocated>,
    ) -> Result<(), Error> {
        match self.deallocated_head.as_mut() {
            None => {
                // List is empty, so make the new node both the head and tail
                self.deallocated_head = Some(item);
            }
            Some(head) => {
                // Find the position to insert the new node, based on the key
                let mut current = head;
                let mut end_of_list = false;
                loop {
                    if current.freed_length >= item.freed_length {
                        break;
                    }
                    match current.next.as_ref() {
                        Some(mut node) => current = current.next.as_mut().unwrap(),
                        None => {
                            end_of_list = true;
                            break;
                        }
                    };
                }

                if end_of_list {
                    // Insert the new node at the end of the list
                    item.prev = Some(current.clone());
                    current.next = Some(item.clone());
                } else if let Some(mut prev) = current.prev.clone() {
                    // Insert the new node between two existing nodes
                    item.prev = Some(prev.clone());
                    //item.next = Some(current.clone());
                    prev.next = Some(item.clone());
                    current.prev = Some(item.clone());
                } else {
                    // Insert the new node at the beginning of the list
                    current.prev = Some(item.clone());
                    item.next = Some(current.clone());
                    self.deallocated_head = Some(item.clone());
                }
            }
        }

        Ok(())
    }

    fn size_of_deallocated_item() -> u32 {
        12
    }

    fn read_deallocated(
        api: &dyn StorageAPI,
        offset: &Offset,
    ) -> Result<(Offset, u32, Offset), Error> {
        let mut data: [u8; 12] = [0; 12];
        api.read(*offset, data.as_mut_slice())?;

        let freed_offset = unsafe { *(data[0..4].as_ptr() as *const Offset) };
        let freed_length = unsafe { *(data[4..8].as_ptr() as *const u32) };
        let next_offset = unsafe { *(data[8..12].as_ptr() as *const Offset) };

        Ok((freed_offset, freed_length, next_offset))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::storage::mock::mock_storage;

    fn check_items(allocated: &Allocator, items: &[(Offset, Offset, u32)]) {
        let mut index = 0;
        let mut current_opt = allocated.deallocated_head.clone();
        while let Some(current) = current_opt.as_ref() {
            assert_eq!(current.offset, items[index].0);
            assert_eq!(current.freed_offset, items[index].1);
            assert_eq!(current.freed_length, items[index].2);

            index += 1;
            current_opt = current.next.clone();
        }

        assert_eq!(items.len(), index);
    }

    #[test]
    fn test_deallocate() {
        let storage_1 = mock_storage(1024);
        let mut allocator = Allocator::create(storage_1.api.as_ref(), &0).unwrap();

        let offset = allocator.allocate(storage_1.api.as_ref(), 32).unwrap();
        assert_eq!(offset, 8);
        allocator.deallocate(storage_1.api.as_ref(), 8, 4).unwrap();
        allocator.deallocate(storage_1.api.as_ref(), 12, 5).unwrap();
        allocator.deallocate(storage_1.api.as_ref(), 32, 1).unwrap();

        check_items(&allocator, &[(64, 32, 1), (40, 8, 4), (52, 12, 5)]);
    }
}
