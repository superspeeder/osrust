#![feature(abi_x86_interrupt)]
#![feature(stmt_expr_attributes)]
#![no_std] // don't link the Rust standard library

use crate::memory::BootInfoFrameAllocator;
use bootloader_api::BootInfo;
use x86_64::structures::paging::OffsetPageTable;

#[macro_use]
pub mod debug_utils;

pub mod acpi;
pub mod binutil;
pub mod gdt;
pub mod interrupts;
pub mod memory;

static mut FRAME_ALLOCATOR: Option<BootInfoFrameAllocator> = None;
static mut MAPPER: Option<OffsetPageTable> = None;

fn init_memory(boot_info: &'static BootInfo) {
    unsafe {
        MAPPER = Some(memory::init(boot_info));
        FRAME_ALLOCATOR = Some(BootInfoFrameAllocator::init(&boot_info.memory_regions));
    }
}

pub fn init(boot_info: &'static BootInfo) {
    // TODO: figure out how to make entering this gdt *not* trigger a gpf
    // gdt::init();
    interrupts::init_idt();
    interrupts::init_pics();
    interrupts::pit::init();
    x86_64::instructions::interrupts::enable();

    init_memory(boot_info);
    acpi::init(boot_info);
}
