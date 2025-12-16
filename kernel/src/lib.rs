#![feature(abi_x86_interrupt)]
#![no_std] // don't link the Rust standard library

#[macro_use]
pub mod debug_utils;

pub mod interrupts;
pub mod gdt;
pub mod paging;

pub fn init() {
    // TODO: figure out how to make entering this gdt *not* trigger a gpf
    // gdt::init();
    paging::init();
    interrupts::init_idt();
    interrupts::init_pics();
    interrupts::pit::init();
    x86_64::instructions::interrupts::enable();
}