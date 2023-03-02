


use super::error::Error;
use super::Offset;

use alloc::boxed::Box;


use kelk_env::{StorageAPI};

#[derive(Debug, Clone)]
pub(self) struct Deallocated {
    pub offset: Offset,
    pub freed_offset: Offset,
    pub freed_length: u32,
    pub next: Option<Box<Deallocated>>,
}

pub(super) struct Allocator {
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
                next: None,
            }));

            if deallocated.is_none() {
                deallocated = cur_deallocated.clone();
            }

            match prv_deallocated.as_mut() {
                Some(item) => {
                    let mut b = item.clone();
                    let _c = &mut *b.clone();
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
        if let Some(head) = self.deallocated_head.as_mut() {
            if head.freed_length >= length {
                let freed_offset = head.freed_offset;
                let offset = head.offset;
                head.freed_length -= length;
                if head.freed_length == 0 {
                    self.deallocated_head = head.next.clone();
                    self.deallocate(api, offset, Self::size_of_deallocated_item())?;
                }
                return Ok(freed_offset);
            } else {
                let mut next_next: Option<Box<Deallocated>> = None;
                let mut current = head;
                let mut not_found = false;
                loop {
                    match current.next.as_mut() {
                        Some(node) => {
                            next_next = node.next.clone();
                            if node.freed_length >= length {
                                break;
                            }

                            current = current.next.as_mut().unwrap();
                        }
                        None => {
                            not_found = current.freed_length < length;
                            break;
                        }
                    };
                }

                if not_found {
                    // Nothing to do
                } else if let Some(mut next) = current.next.clone() {
                    let freed_offset = next.freed_offset;
                    let offset = next.offset;
                    next.freed_length -= length;
                    if current.freed_length == 0 {
                        self.deallocate(api, offset, Self::size_of_deallocated_item())?;
                    } else if current.freed_length > next.freed_length {
                        current.next = next_next;
                        self.deallocate_item(api, next)?;
                    }
                    return Ok(freed_offset);
                }
            }
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
            next: None,
        });

        self.allocation_offset += Self::size_of_deallocated_item();

        self.deallocate_item(api, item)
    }

    fn deallocate_item(
        &mut self,
        _api: &dyn StorageAPI,
        mut item: Box<Deallocated>,
    ) -> Result<(), Error> {
        match self.deallocated_head.as_mut() {
            None => {
                // List is empty, so make the new node both the head and tail
                self.deallocated_head = Some(item);
            }
            Some(head) => {
                if head.freed_length > item.freed_length {
                    // Insert the new node at the beginning of the list
                    item.next = Some(head.clone());
                    self.deallocated_head = Some(item);
                } else {
                    // Find the position to insert the new node, based on the key
                    let mut current = head;
                    let mut add_to_tail = false;
                    loop {
                        match current.next.as_mut() {
                            Some(node) => {
                                if node.freed_length >= item.freed_length {
                                    break;
                                }

                                current = current.next.as_mut().unwrap();
                            }
                            None => {
                                add_to_tail = current.freed_length < item.freed_length;
                                break;
                            }
                        };
                    }

                    if add_to_tail {
                        // Insert the new node at the end of the list
                        current.next = Some(item);
                    } else {
                        // Insert the new node between two existing nodes
                        item.next = current.next.clone();
                        current.next = Some(item.clone());
                    }
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

    fn check_deallocated_items(allocated: &Allocator, items: &[(Offset, Offset, u32)]) {
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
        allocator.deallocate(storage_1.api.as_ref(), 33, 2).unwrap();

        check_deallocated_items(
            &allocator,
            &[(64, 32, 1), (76, 33, 2), (40, 8, 4), (52, 12, 5)],
        );

        assert_eq!(allocator.allocate(storage_1.api.as_ref(), 1).unwrap(), 32);
        assert_eq!(allocator.allocate(storage_1.api.as_ref(), 1).unwrap(), 33);
        assert_eq!(allocator.allocate(storage_1.api.as_ref(), 9).unwrap(), 64);
        assert_eq!(allocator.allocate(storage_1.api.as_ref(), 12).unwrap(), 100);

        check_deallocated_items(
            &allocator,
            &[(76, 33, 1), (88, 64, 3), (40, 8, 4), (52, 12, 5)],
        );
    }
}
