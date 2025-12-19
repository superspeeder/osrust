#[repr(C)]
pub struct Block {
    memory: *mut u8,
    tree: TreePointer,
    size: usize,
}

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct TreePointer(*mut u8);

impl TreePointer {
    pub fn init(memory: *mut u8, block_size: usize, min_allocation: usize) -> Self {
        let size = tree_size(block_size, min_allocation);
        for i in 0..size {
            unsafe {
                *(memory.offset(i as isize)) = 0xFF;
            }
        }

        Self(memory)
    }

    pub fn is_allocated_direct(&self, index: usize) -> bool {
        let byte = index / 8;
        let bit = index % 8;
        unsafe { *self.0.offset(byte as isize) & (1 << bit) != 0 }
    }

    pub fn is_free_to_allocate(&self, index: usize, total_layers: usize) -> bool {
        let layer = (index + 1).ilog2();
        

        todo!()

    }
}

pub trait BlockAllocator {
    fn allocate_block(&mut self) -> Block;
}

pub trait TreeAllocator {
    fn allocate_tree(&mut self, block_size: usize) -> TreePointer;
}

impl Block {
    pub fn memory_mut(&mut self) -> &'static mut [u8] {
        unsafe {
            core::slice::from_mut_ptr_range(
                self.memory..self.memory.byte_offset(self.size as isize),
            )
        }
    }

    pub fn tree(&self) -> TreePointer {
        self.tree
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn allocate_segment(&mut self, size: usize) {
        assert!(size <= self.size);
    }
}

pub enum TreeAllocatorType<'a> {
    ///
    /// Places the tree at the end of a block
    ///
    Internal,

    ///
    /// A tree allocator which allocated memory for the trees outside the allocated blocks.
    ///
    /// The passed allocator is expected to use some form of dynamic allocation. If dynamic allocation is not available, use the [Internal] allocator type.
    ///
    External(&'a (dyn TreeAllocator + 'static)),
}

pub struct BuddyAllocator<'a, T: BlockAllocator> {
    ///
    /// Minimum allowed allocation size. This determines allocation alignment too.
    ///
    /// This must be a power of 2
    ///
    minimum_allocation_size: usize,

    ///
    /// Maximum size of a memory block
    ///
    /// This must be a power of 2
    ///
    maximum_block_size: usize,

    ///
    /// Allocation method for trees.
    ///
    tree_allocator: TreeAllocatorType<'a>,

    ///
    /// Allocator for memory blocks
    ///
    block_allocator: T,

    ///
    /// The actual blocks of memory
    ///
    blocks: [Block; 128],
}

const fn tree_size(block_size: usize, min_allocation_size: usize) -> usize {
    debug_assert!(block_size & (block_size - 1) == 0);
    debug_assert!(min_allocation_size & (min_allocation_size - 1) == 0);

    let layer_count = block_size.ilog2() - min_allocation_size.ilog2();
    debug_assert!(layer_count < 63);
    1 << (layer_count - 2)
}
