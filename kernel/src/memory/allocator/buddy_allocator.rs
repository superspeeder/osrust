use crate::klib::linked_list::{RawLinkedList, RawLinkedListNode};
use crate::memory::FRAME_ALLOCATOR;
use crate::memory::allocator::paged_pool::{PagedPool, PoolAllocator};
use crate::memory::frame_allocator::frame_allocator;
use arrayvec::ArrayVec;
use bitflags::bitflags;
use bootloader_api::BootInfo;
use bootloader_api::info::MemoryRegionKind;
use core::cmp::min;
use x86_64::PhysAddr;

#[repr(C)]
pub struct Block {
    block_ptr: *mut u8,
    left: *mut RawLinkedListNode<Block>,
    right: *mut RawLinkedListNode<Block>,
    size: u8,
    flags: BlockFlags,
}

bitflags! {
    #[derive(Copy, Clone)]
    pub struct BlockFlags: u8 {
        const USED = 1;
    }
}

impl Block {
    unsafe fn of(
        block_ptr: *mut u8,
        left: *mut RawLinkedListNode<Block>,
        right: *mut RawLinkedListNode<Block>,
        size: u8,
        flags: BlockFlags,
    ) -> Self {
        Self {
            block_ptr,
            left,
            right,
            size,
            flags,
        }
    }

    pub(self) fn set_values(&mut self, start: *mut u8, size: u8, flags: BlockFlags) {
        self.block_ptr = start;
        self.size = size;
        self.flags = flags;
    }

    unsafe fn merge_children(&mut self, unused_list: &mut RawLinkedList<Block>) {
        unsafe {
            (*self.left).value.reset();
            (*self.right).value.reset();
            unused_list.append(&mut *self.left);
            unused_list.append(&mut *self.right);
            self.left = core::ptr::null_mut();
            self.right = core::ptr::null_mut();
        }
    }

    unsafe fn split(&mut self, node_allocator: &mut PoolAllocator<Block>) {
        unsafe {
            let mut left = node_allocator.alloc(frame_allocator());
            left.set_values(self.block_ptr, self.size - 1, self.flags);
            let mut right = node_allocator.alloc(frame_allocator());
            right.set_values(
                self.block_ptr.offset(1 << (self.size - 1)),
                self.size - 1,
                self.flags,
            );
            self.left = &raw mut *left;
            self.right = &raw mut *right;
        }
    }

    unsafe fn get_block_of_size(
        self: &mut RawLinkedListNode<Self>,
        size: u8,
        node_allocator: &mut PoolAllocator<Block>,
    ) -> Option<&'static mut RawLinkedListNode<Block>> {
        let block = if self.flags.contains(BlockFlags::USED) {
            None
        } else if !(self.left.is_null() && self.right.is_null()) {
            if self.size == size {
                self.flags |= BlockFlags::USED;
                return Some(self);
            } else {
                self.left
                    .get_block_of_size(size, node_allocator)
                    .or_else(|| self.right.get_block_of_size(size, node_allocator))
            }
        } else if self.size > size {
            self.split(node_allocator);
            self.left
                .get_block_of_size(size, node_allocator)
                .or_else(|| self.right.get_block_of_size(size, node_allocator))
        } else {
            None
        };

        if let Some(block) = block.as_ref() {
            if (&*self.left).flags.contains(BlockFlags::USED)
                && (&*self.right).flags.contains(BlockFlags::USED)
            {
                self.flags |= BlockFlags::USED;
            }
        }

        block
    }

    /// This resets the current block.
    /// While the operation itself is not unsafe, using this may create unsafe conditions elsewhere.
    /// This should only be used during block frees
    fn reset(&mut self) {
        self.block_ptr = core::ptr::null_mut();
        self.left = core::ptr::null_mut();
        self.right = core::ptr::null_mut();
        self.size = 0;
        self.flags = BlockFlags::empty();
    }
}

const BUDDYALLOC_MAX_SIZE_LOG2: u8 = 30; // 1 GiB
const BUDDYALLOC_MIN_SIZE_LOG2: u8 = 12; // 4 KiB

pub const BUDDYALLOC_MAX_SIZE: u64 = 1 << BUDDYALLOC_MAX_SIZE_LOG2;
pub const BUDDYALLOC_MIN_SIZE: u64 = 1 << BUDDYALLOC_MIN_SIZE_LOG2;

pub struct BuddyAllocator {
    node_source: PoolAllocator<Block>,
    boot_info: &'static BootInfo,
    blocks: RawLinkedList<Block>,
}

impl BuddyAllocator {
    pub fn new(boot_info: &'static BootInfo, skip_frames: usize) -> Self {
        let first_open_frame = unsafe { FRAME_ALLOCATOR.unwrap_unchecked() }
            .usable_frames()
            .nth(skip_frames)
            .map(|f| f.start_address())
            .unwrap_or(PhysAddr::new(0));

        let mut blocks = RawLinkedList::new();
        let mut node_source = PoolAllocator::new(frame_allocator());

        let regions = boot_info.memory_regions.iter().filter(|r| {
            r.kind == MemoryRegionKind::Usable && r.start + r.end < first_open_frame.as_u64()
        });

        for region in regions {
            let mut cursor = min(region.start, first_open_frame.as_u64());
            while cursor < region.end {
                if cursor & 0x3fffffff == 0 {
                    // aligned to 1 GiB

                    if region.end - cursor >= BUDDYALLOC_MAX_SIZE {
                        let mut block_node = node_source.alloc(frame_allocator());
                        block_node.set_values()
                    }
                } else if cursor & 0x1fffff == 0 {
                    // aligned to 2MiB
                }
            }
        }

        Self {
            node_source,
            boot_info,
            blocks,
        }
    }

    fn alloc_raw(&mut self, size: u8) -> &'static mut RawLinkedListNode<Block> {
        for block in self.blocks.iter_mut() {
            if let Some(allocation) =
                unsafe { block.get_block_of_size(size, &mut self.node_source) }
            {
                return allocation;
            }
        }

        panic!("Failed to allocate memory");
    }

    pub fn alloc(&mut self, size: usize) -> &'static mut RawLinkedListNode<Block> {
        self.alloc_raw(size.bit_width() as u8)
    }
}
