#![feature(abi_x86_interrupt)]
#![no_std] // don't link the Rust standard library

use bootloader_api::BootInfo;
use crate::memory::BootInfoFrameAllocator;

#[macro_use]
pub mod debug_utils;

pub mod interrupts;
pub mod gdt;
pub mod memory;
pub mod acpi;

static mut FRAME_ALLOCATOR: Option<BootInfoFrameAllocator> = None;

pub fn init(boot_info: &BootInfo) {
    // TODO: figure out how to make entering this gdt *not* trigger a gpf
    // gdt::init();
    interrupts::init_idt();
    interrupts::init_pics();
    interrupts::pit::init();
    x86_64::instructions::interrupts::enable();

    let mut mapper = unsafe { memory::init(boot_info) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_regions)
    };
    unsafe {
        FRAME_ALLOCATOR = Some(frame_allocator);
    }
}

