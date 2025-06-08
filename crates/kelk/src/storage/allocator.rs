use super::Offset;
/// TODO: refactor this code.
use super::error::Error;
use alloc::boxed::Box;
use kelk_env::StorageAPI;

#[derive(Debug, Clone)]
 struct Deallocated {
    pub offset: Offset,
    pub freed_offset: Offset,
    pub freed_length: u32,
    pub next: Option<Box<Deallocated>>,
}

pub(super) struct Allocator {
    offset: Offset,
    allocation_offset: Offset,
    deallocated_head: Option<Box<Deallocated>>,
}

impl Allocator {
    pub fn create(api: &dyn StorageAPI, offset: Offset) -> Result<Self, Error> {
        let allocation_offset = offset + 8;
        let data: &[u8; 4] = unsafe { core::mem::transmute(&allocation_offset) };
        api.write(offset, data)?; // allocation offset
        api.write(offset + 4, &[0; 4])?; // deallocation offset

        Ok(Self {
            offset,
            allocation_offset,
            deallocated_head: None,
        })
    }

    pub fn load(api: &dyn StorageAPI, offset: Offset) -> Result<Self, Error> {
        let mut data: [u8; 8] = [0; 8];
        api.read(offset, &mut data)?;

        let allocation_offset = unsafe { *(data[0..4].as_ptr() as *const Offset) };
        let deallocated_head_offset = unsafe { *(data[4..8].as_ptr() as *const Offset) };

        let mut deallocated_head: Option<Box<Deallocated>> = None;
        if deallocated_head_offset != 0 {
            let mut current = &mut deallocated_head;
            let mut next_offset = deallocated_head_offset;
            loop {
                let (freed_offset, freed_length, next_item_offset) =
                    Allocator::read_deallocated(api, &next_offset)?;
                *current = Some(Box::new(Deallocated {
                    offset: next_offset,
                    freed_offset,
                    freed_length,
                    next: None,
                }));
                current = &mut current.as_mut().unwrap().next;
                next_offset = next_item_offset;
                if next_offset == 0 {
                    break;
                }
            }
        }

        Ok(Self {
            offset,
            allocation_offset,
            deallocated_head,
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
                self.update_deallocation_head(api)?;
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
                        Self::write_deallocated_item(api, current)?;
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
        api.write(self.offset, data)?;

        Ok(cur_free_pos)
    }

    pub fn deallocate(
        &mut self,
        api: &dyn StorageAPI,
        offset: Offset,
        length: u32,
    ) -> Result<(), Error> {
        let item_offset = self.allocate(api, Self::size_of_deallocated_item())?;
        let item = Box::new(Deallocated {
            offset: item_offset,
            freed_offset: offset,
            freed_length: length,
            next: None,
        });

        self.deallocate_item(api, item)
    }

    fn deallocate_item(
        &mut self,
        api: &dyn StorageAPI,
        mut item: Box<Deallocated>,
    ) -> Result<(), Error> {
        match self.deallocated_head.as_mut() {
            None => {
                // List is empty, so make the new node both the head and tail
                self.deallocated_head = Some(item);

                self.update_deallocation_head(api)?;
            }
            Some(head) => {
                if head.freed_length > item.freed_length {
                    // Insert the new node at the beginning of the list
                    item.next = Some(head.clone());
                    Self::write_deallocated_item(api, &item)?;

                    self.deallocated_head = Some(item);
                    self.update_deallocation_head(api)?;
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
                        Self::write_deallocated_item(api, &item)?;

                        current.next = Some(item);
                        Self::write_deallocated_item(api, current)?;
                    } else {
                        // Insert the new node between two existing nodes
                        item.next = current.next.clone();
                        Self::write_deallocated_item(api, &item)?;

                        current.next = Some(item);
                        Self::write_deallocated_item(api, current)?;
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
        let mut buf: [u8; 12] = [0; 12];
        api.read(*offset, buf.as_mut_slice())?;

        let freed_offset = unsafe { *(buf[0..4].as_ptr() as *const Offset) };
        let freed_length = unsafe { *(buf[4..8].as_ptr() as *const u32) };
        let next_offset = unsafe { *(buf[8..12].as_ptr() as *const Offset) };

        Ok((freed_offset, freed_length, next_offset))
    }

    fn write_deallocated_item(api: &dyn StorageAPI, item: &Deallocated) -> Result<(), Error> {
        let mut buf: [u8; 12] = [0; 12];
        let next_offset = match &item.next {
            Some(next) => next.offset,
            None => 0,
        };

        unsafe {
            *(buf.as_mut_ptr() as *mut u32) = item.freed_offset;
            *(buf.as_mut_ptr().add(4) as *mut u32) = item.freed_length;
            *(buf.as_mut_ptr().add(8) as *mut u32) = next_offset;
        }

        Ok(api.write(item.offset, &buf)?)
    }

    fn update_deallocation_head(&self, api: &dyn StorageAPI) -> Result<(), Error> {
        let deallocated_head_offset = match &self.deallocated_head {
            Some(item) => item.offset,
            None => 0,
        };
        let data: &[u8; 4] = unsafe { core::mem::transmute(&deallocated_head_offset) };
        Ok(api.write(self.offset + 4, data)?)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::storage::{Storage, mock::mock_storage};

    fn check_deallocated_items(storage: &Storage, items: &[(Offset, Offset, u32)]) {
        let allocator = Allocator::load(storage.api.as_ref(), 0).unwrap();
        let mut index = 0;
        let mut current_opt = allocator.deallocated_head;
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
        let storage = mock_storage(1024);
        let mut allocator = Allocator::create(storage.api.as_ref(), 0).unwrap();

        let offset = allocator.allocate(storage.api.as_ref(), 32).unwrap();
        assert_eq!(offset, 8);
        allocator.deallocate(storage.api.as_ref(), 8, 4).unwrap();
        allocator.deallocate(storage.api.as_ref(), 12, 5).unwrap();
        allocator.deallocate(storage.api.as_ref(), 32, 1).unwrap();
        allocator.deallocate(storage.api.as_ref(), 33, 2).unwrap();

        check_deallocated_items(
            &storage,
            &[(64, 32, 1), (76, 33, 2), (40, 8, 4), (52, 12, 5)],
        );

        assert_eq!(allocator.allocate(storage.api.as_ref(), 1).unwrap(), 32);
        assert_eq!(allocator.allocate(storage.api.as_ref(), 1).unwrap(), 33);
        assert_eq!(allocator.allocate(storage.api.as_ref(), 9).unwrap(), 64);
        assert_eq!(allocator.allocate(storage.api.as_ref(), 12).unwrap(), 100);

        check_deallocated_items(
            &storage,
            &[(76, 33, 1), (88, 64, 3), (40, 8, 4), (52, 12, 5)],
        );
    }

    #[test]
    fn test_allocation() {
        let storage = mock_storage(1024);
        let mut allocator_1 = Allocator::create(storage.api.as_ref(), 0).unwrap();

        let offset = allocator_1.allocate(storage.api.as_ref(), 32).unwrap();
        assert_eq!(offset, 8);

        let allocator_2 = Allocator::load(storage.api.as_ref(), 0).unwrap();
        assert_eq!(allocator_1.allocation_offset, allocator_2.allocation_offset);
    }
}
