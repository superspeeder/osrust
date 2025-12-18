use bootloader_api::BootInfo;

pub mod hpet;
pub mod init;
pub mod apic;
pub(crate) mod pcie;

pub fn init(boot_info: &'static BootInfo) {
    init::load_acpi(boot_info.rsdp_addr.into_option().expect("RSDP Address not passed by bootloader"));
}


