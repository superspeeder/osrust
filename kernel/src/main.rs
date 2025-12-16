#![feature(abi_x86_interrupt)]
#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use kernel::{debug_utils::SERIAL, init, memory, println, PHYSICAL_MEMORY_OFFSET};

use bootloader_api::config::Mapping;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use core::fmt::Write;
use kernel::memory::BootInfoFrameAllocator;
use x86_64::instructions::hlt;
use x86_64::structures::paging::{Page, Translate};
use x86_64::VirtAddr;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    println!("Entered kernel with boot info: {boot_info:?}");
    init(boot_info);


    loop {
        hlt();
    }
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let _ = writeln!(SERIAL.lock(), "PANIC: {info}");
    loop {
        hlt();
    }
}
