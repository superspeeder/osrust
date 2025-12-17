#![feature(abi_x86_interrupt)]
#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

extern crate alloc;

use alloc::boxed::Box;
use kernel::{debug_utils::SERIAL, init, println};

use bootloader_api::config::Mapping;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use core::fmt::Write;
use x86_64::instructions::hlt;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    init(boot_info);

    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    loop {
        hlt();
    }
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let _ = writeln!(SERIAL.lock(), "\x1b[0;91mPANIC: {info}\x1b[0m");
    loop {
        hlt();
    }
}
