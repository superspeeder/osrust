#![allow(internal_features)]
#![feature(abi_x86_interrupt)]
#![feature(core_intrinsics)]
#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

extern crate alloc;

use alloc::boxed::Box;
use kernel::{debug_utils::SERIAL, init, println};

use bootloader_api::config::Mapping;
use bootloader_api::{BootInfo, BootloaderConfig, entry_point};
use core::fmt::Write;
use core::intrinsics::volatile_store;
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

    unsafe {
        volatile_store(kernel::memory::ERROR_ADDRESS as *mut u8, 14);
    }

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
