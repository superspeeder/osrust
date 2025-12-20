pub mod boot_info;
pub mod general_purpose;

use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};
use crate::memory::FRAME_ALLOCATOR;

pub fn frame_allocator() -> &'static mut impl FrameAllocator<Size4KiB> {
    unsafe { FRAME_ALLOCATOR.as_mut().unwrap() }
}


