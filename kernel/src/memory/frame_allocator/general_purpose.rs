use crate::memory::allocator::buddy_allocator::BuddyAllocator;
use arrayvec::ArrayVec;
use x86_64::structures::paging::{PhysFrame, Size4KiB};

pub struct GeneralPurposeFrameAllocator {
    /// Maximum 256 preused frames from before the general purpose allocator could start.
    /// Almost all of these will likely be used either for the initial set of page tables or for the
    /// initial free list page.
    preused_frames: ArrayVec<PhysFrame<Size4KiB>, 256>,
    buddy_allocator: BuddyAllocator,
}
