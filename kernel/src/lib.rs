#![feature(abi_x86_interrupt)]
#![feature(stmt_expr_attributes)]
#![feature(core_intrinsics)]
#![allow(static_mut_refs)]
#![allow(internal_features)]
#![no_std] // don't link the Rust standard library

use bootloader_api::BootInfo;
use log::info;

#[macro_use]
pub mod debug_utils;

pub mod acpi;
pub mod binutil;
pub mod gdt;
pub mod interrupts;
mod logger;
pub mod memory;
pub mod allocator;

pub fn init(boot_info: &'static BootInfo) {
    unsafe { log::set_logger_racy(&logger::LOGGER).expect("Failed to configure logger") };
    unsafe { log::set_max_level_racy(log::LevelFilter::Debug) };
    print!("\x1b[2J\x1b[H");
    info!("Initialized Logger");

    gdt::init();
    interrupts::init_idt();
    interrupts::init_pics();
    interrupts::pit::init();
    x86_64::instructions::interrupts::enable();

    memory::init(boot_info);
    allocator::init_heap(memory::mapper(), memory::frame_allocator());

    acpi::init(boot_info);

    info!("Kernel initialized");
}
