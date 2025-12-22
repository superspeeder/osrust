#![feature(abi_x86_interrupt)]
#![feature(stmt_expr_attributes)]
#![feature(core_intrinsics)]
#![feature(trusted_random_access)]
#![feature(slice_from_ptr_range)]
#![feature(uint_bit_width)]
#![feature(deref_pure_trait)]
#![feature(arbitrary_self_types)]
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
mod klib;
mod logger;
pub mod memory;
pub mod support;

pub fn init(boot_info: &'static BootInfo) {
    unsafe { log::set_logger_racy(&logger::LOGGER).expect("Failed to configure logger") };
    unsafe { log::set_max_level_racy(log::LevelFilter::Trace) };
    print!("\x1b[H\x1b[2J");
    info!("Initialized Logger");

    gdt::init();
    memory::init(boot_info);
    memory::allocator::init_heap(memory::mapper(), memory::frame_allocator::frame_allocator())
        .expect("Failed to initialize heap");

    acpi::init(boot_info);

    interrupts::init_idt();
    interrupts::disable_8259_pic();
    acpi::apic::init();
    acpi::hpet::init();
    acpi::pcie::init();
    interrupts::pit::init();
    x86_64::instructions::interrupts::enable();

    info!("Kernel initialized");
}
