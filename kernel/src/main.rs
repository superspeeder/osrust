#![feature(abi_x86_interrupt)]
#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use kernel::{init, debug_utils::SERIAL, println, print};

use bootloader_api::{entry_point, BootInfo};
use core::fmt::Write;
use x86_64::instructions::hlt;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    println!("Entered kernel with boot info: {boot_info:?}");
    init();

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
