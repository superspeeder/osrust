use core::intrinsics::volatile_load;

use acpi::sdt::hpet::HpetTable;
use log::info;

use crate::acpi::init::acpi_platform;
use crate::memory;

use crate::memory::ERROR_ADDRESS;

static mut HPET_BASE: u64 = ERROR_ADDRESS; // this is an error value which *should*

pub fn init() {
    unsafe {
        let (hpet_addr, head) = acpi_platform()
            .tables
            .table_headers()
            .find(|(_physical_address, header)| header.signature == acpi::sdt::Signature::HPET)
            .expect("HPET not supported");
        memory::map_identity(hpet_addr as u64..hpet_addr as u64 + head.length as u64);
        let hpet = &*(hpet_addr as *const HpetTable);
        info!("HPET Table: {:#?}", hpet);

        if hpet.base_address.address_space != 0 {
            unimplemented!(
                "HPET registers not mapped to system memory. This is currently not implemented."
            )
        }

        HPET_BASE = hpet.base_address.address;
        memory::map_identity(HPET_BASE..HPET_BASE + 0xFF);

        info!("HPET Current Value: {:?}", poll_hpet());
    }
}

pub fn poll_hpet() -> u64 {
    let period = unsafe { volatile_load(HPET_BASE as *const u64) } >> 32;
    let count = unsafe { volatile_load((HPET_BASE + 0xF0) as *const u64) };
    count * period
}
