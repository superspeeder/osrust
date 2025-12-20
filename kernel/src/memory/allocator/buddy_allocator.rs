use crate::klib::linked_list::{RawLinkedList, RawLinkedListNode};
use crate::memory::allocator::paged_pool::{PagedPool, PoolAllocator};
use crate::memory::frame_allocator::frame_allocator;
use arrayvec::ArrayVec;
use bitflags::bitflags;
use bootloader_api::BootInfo;

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

    fn set_values(&mut self, start: *mut u8, size: u8, flags: BlockFlags) {
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
        if self.flags.contains(BlockFlags::USED) {
            None
        } else if !(self.left.is_null() && self.right.is_null()) {
            if self.size == size {
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
        }
    }

    pub fn mark_used(&mut self) {
        self.flags |= BlockFlags::USED;
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
    pub fn new(boot_info: &'static BootInfo) -> Self {
        Self {
            node_source: PoolAllocator::new(frame_allocator()),
            boot_info,
            blocks: RawLinkedList::new(),
        }
    }

    pub fn alloc(&mut self, size: u8) -> RawLinkedListNode<Block> {}
}
