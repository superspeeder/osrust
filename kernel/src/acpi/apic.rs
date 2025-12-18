use core::intrinsics::{volatile_load, volatile_store};

use log::info;

use crate::{
    memory::{self, ERROR_ADDRESS},
    support::{CPU_FLAGS, CPUFlags},
};

static mut APIC_BASE: u64 = ERROR_ADDRESS;

fn get_base() -> u64 {
    unsafe { x86_64::registers::model_specific::ApicBase::MSR.read() }
}

#[allow(const_item_mutation)]
pub fn init() {
    if !CPU_FLAGS.contains(CPUFlags::APIC) {
        panic!("APIC is not available");
    }

    unsafe {
        x86_64::registers::model_specific::ApicBase::MSR.write(get_base());
        APIC_BASE = get_base();
        memory::map_identity(APIC_BASE..(APIC_BASE + 0x03F0));

        write_reg(0xF0, read_reg(0xF0) | 0x100);
    }
    info!("APIC initialized");
}

#[allow(dead_code)]
pub(super) unsafe fn acpi_reg_addr(offset: u64) -> u64 {
    unsafe { APIC_BASE + offset }
}

unsafe fn write_reg(offset: u64, data: u32) {
    unsafe {
        volatile_store((APIC_BASE + offset) as *mut u32, data);
    }
}

unsafe fn read_reg(offset: u64) -> u32 {
    unsafe { volatile_load((APIC_BASE + offset) as *mut u32) }
}

pub fn write_eoi() {
    unsafe {
        write_reg(0xB0, 0);
    }
}
