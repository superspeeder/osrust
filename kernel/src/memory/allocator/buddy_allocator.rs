use crate::klib::linked_list::{RawLinkedList, RawLinkedListNode};
use crate::memory::FRAME_ALLOCATOR;
use crate::memory::allocator::paged_pool::PoolAllocator;
use crate::memory::frame_allocator::frame_allocator;
use bitflags::bitflags;
use bootloader_api::BootInfo;
use bootloader_api::info::MemoryRegionKind;
use core::cmp::min;
use log::trace;
use x86_64::PhysAddr;
use x86_64::structures::paging::{FrameAllocator, PageSize, PhysFrame, Size1GiB, Size2MiB, Size4KiB};
use crate::logger::IntoLoggedAddress;

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
        self: &'static mut RawLinkedListNode<Self>,
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
                (&mut *self.left)
                    .get_block_of_size(size, node_allocator)
                    .or_else(|| (&mut *self.right).get_block_of_size(size, node_allocator))
            }
        } else if self.size > size {
            self.split(node_allocator);
            (&mut *self.left)
                .get_block_of_size(size, node_allocator)
                .or_else(|| (&mut *self.right).get_block_of_size(size, node_allocator))
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

    pub fn contains_frame(&self, frame: PhysFrame) -> bool {
        (self.block_ptr as u64) < frame.start_address().as_u64()
            && frame.size().ilog2() <= self.size as u32
    }

    pub fn mark_frame_used(
        &mut self,
        frame: PhysFrame<Size4KiB>,
        node_allocator: &mut PoolAllocator<Block>,
    ) {
        if self.size > 12 {
            let offset = frame.start_address().as_u64() - (self.block_ptr as u64);
            if offset < (1u64 << (self.size - 1)) {
                // left
                if self.left.is_null() {
                    unsafe { self.split(node_allocator) }
                }

                unsafe { (&mut *self.left).mark_frame_used(frame, node_allocator) }
            } else {
                // right
                if self.right.is_null() {
                    unsafe { self.split(node_allocator) }
                }

                unsafe { (&mut *self.right).mark_frame_used(frame, node_allocator) }
            }
        }
    }

    pub fn frame<T: PageSize>(&self) -> PhysFrame<T> {
        assert_eq!(self.size, T::SIZE.ilog2() as u8);
        PhysFrame::containing_address(PhysAddr::new(self.block_ptr as u64))
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
        let mut blocks: RawLinkedList<Block> = RawLinkedList::new();
        let mut node_source: PoolAllocator<Block> = PoolAllocator::new(frame_allocator());

        let first_open_frame = unsafe { FRAME_ALLOCATOR.as_mut().unwrap_unchecked() }
            .usable_frames()
            .nth(unsafe { FRAME_ALLOCATOR.as_ref().unwrap_unchecked().num_used() })
            .map(|f| f.start_address())
            .unwrap_or(PhysAddr::new(0));

        // TODO: mark used frames as used in the allocator

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
                        block_node.set_values(cursor as *mut u8, 30, BlockFlags::empty());
                        cursor += 1 << 30;
                    }
                } else if cursor & 0x1fffff == 0 {
                    // aligned to 2MiB
                    if region.end - cursor >= 1 << 21 {
                        let mut block_node = node_source.alloc(frame_allocator());
                        block_node.set_values(cursor as *mut u8, 21, BlockFlags::empty());
                        cursor += 1 << 21;
                    }
                } else if cursor & 0xfff == 0 {
                    // aligned to 4KiB
                    if region.end - cursor >= 1 << 12 {
                        let mut block_node = node_source.alloc(frame_allocator());
                        block_node.set_values(cursor as *mut u8, 12, BlockFlags::empty());
                        cursor += 1 << 12;
                    }
                }
            }
        }

        let mut counter = 0;
        for frame in unsafe { FRAME_ALLOCATOR.as_ref().unwrap().usable_frames() } {
            unsafe {
                if counter > FRAME_ALLOCATOR.as_ref().unwrap().num_used() {
                    break;
                }
            }

            for block in blocks.iter_mut() {
                if block.contains_frame(frame) {
                    block.mark_frame_used(frame, &mut node_source);
                    trace!("Reserved 4KiB frame {:?}", frame.start_address().into_log());
                    break;
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

    #[inline]
    pub fn alloc(&mut self, size: usize) -> &'static mut RawLinkedListNode<Block> {
        self.alloc_raw((size - 1).bit_width() as u8)
    }
}

unsafe impl FrameAllocator<Size4KiB> for BuddyAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let node = self.alloc_raw(12);
        Some(node.frame())
    }
}

unsafe impl FrameAllocator<Size2MiB> for BuddyAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size2MiB>> {
        let node = self.alloc_raw(21);
        Some(node.frame())
    }
}

unsafe impl FrameAllocator<Size1GiB> for BuddyAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size1GiB>> {
        let node = self.alloc_raw(30);
        Some(node.frame())
    }
}
