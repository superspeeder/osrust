pub mod boot_info;
pub mod general_purpose;

use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::PhysAddr;
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};
use crate::memory::FRAME_ALLOCATOR;

pub struct GeneralPurposeFrameAllocator {
    boot_memory_map: &'static MemoryRegions,

}

pub fn frame_allocator() -> &'static mut impl FrameAllocator<Size4KiB> {
    unsafe { FRAME_ALLOCATOR.as_mut().unwrap() }
}


