use crate::logger::LoggedAddress;
use crate::memory;
use crate::memory::ERROR_ADDRESS;
use crate::support::{CPU_FLAGS, CPUFlags};
use acpi::aml::AmlError;
use acpi::platform::{AcpiPlatform, InterruptModel, PciConfigRegions};
use acpi::sdt::hpet::HpetTable;
use acpi::{Handle, PciAddress, PhysicalMapping};
use bootloader_api::BootInfo;
use core::intrinsics::{volatile_load, volatile_store};
use core::ptr::NonNull;
use log::{debug, info};
use x86_64::instructions::port::{PortRead, PortWrite};

#[derive(Copy, Clone)]
struct AcpiHandler;

impl acpi::Handler for AcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        debug!(
            "Requested memory map of {:?}, len {:?}",
            LoggedAddress::Physical(physical_address as u64),
            size
        );
        let mapping =
            memory::map_identity(physical_address as u64..(physical_address + size) as u64);

        PhysicalMapping {
            physical_start: physical_address,
            virtual_start: NonNull::new_unchecked(physical_address as *mut _),
            region_length: size,
            mapped_length: mapping.size() as usize,
            handler: self.clone(),
        }
    }

    fn unmap_physical_region<T>(region: &PhysicalMapping<Self, T>) {
        // TODO: do anything here
    }

    fn read_u8(&self, address: usize) -> u8 {
        unsafe { volatile_load(address as *const _) }
    }

    fn read_u16(&self, address: usize) -> u16 {
        unsafe { volatile_load(address as *const _) }
    }

    fn read_u32(&self, address: usize) -> u32 {
        unsafe { volatile_load(address as *const _) }
    }

    fn read_u64(&self, address: usize) -> u64 {
        unsafe { volatile_load(address as *const _) }
    }

    fn write_u8(&self, address: usize, value: u8) {
        unsafe { volatile_store(address as *mut _, value) };
    }

    fn write_u16(&self, address: usize, value: u16) {
        unsafe { volatile_store(address as *mut _, value) };
    }

    fn write_u32(&self, address: usize, value: u32) {
        unsafe { volatile_store(address as *mut _, value) };
    }

    fn write_u64(&self, address: usize, value: u64) {
        unsafe { volatile_store(address as *mut _, value) };
    }

    fn read_io_u8(&self, port: u16) -> u8 {
        unsafe { u8::read_from_port(port) }
    }

    fn read_io_u16(&self, port: u16) -> u16 {
        unsafe { u16::read_from_port(port) }
    }

    fn read_io_u32(&self, port: u16) -> u32 {
        unsafe { u32::read_from_port(port) }
    }

    fn write_io_u8(&self, port: u16, value: u8) {
        unsafe { u8::write_to_port(port, value) }
    }

    fn write_io_u16(&self, port: u16, value: u16) {
        unsafe { u16::write_to_port(port, value) }
    }

    fn write_io_u32(&self, port: u16, value: u32) {
        unsafe { u32::write_to_port(port, value) }
    }

    fn read_pci_u8(&self, address: PciAddress, offset: u16) -> u8 {
        todo!()
    }

    fn read_pci_u16(&self, address: PciAddress, offset: u16) -> u16 {
        todo!()
    }

    fn read_pci_u32(&self, address: PciAddress, offset: u16) -> u32 {
        todo!()
    }

    fn write_pci_u8(&self, address: PciAddress, offset: u16, value: u8) {
        todo!()
    }

    fn write_pci_u16(&self, address: PciAddress, offset: u16, value: u16) {
        todo!()
    }

    fn write_pci_u32(&self, address: PciAddress, offset: u16, value: u32) {
        todo!()
    }

    fn nanos_since_boot(&self) -> u64 {
        todo!()
    }

    fn stall(&self, microseconds: u64) {
        todo!()
    }

    fn sleep(&self, milliseconds: u64) {
        todo!()
    }

    fn create_mutex(&self) -> Handle {
        todo!()
    }

    fn acquire(&self, mutex: Handle, timeout: u16) -> Result<(), AmlError> {
        todo!()
    }

    fn release(&self, mutex: Handle) {
        todo!()
    }
}

pub fn init(boot_info: &'static BootInfo) {
    let acpi_handler = AcpiHandler;
    unsafe {
        let rsdp_addr = boot_info.rsdp_addr.into_option().unwrap() as usize;
        debug!("RSDP is at {:?}", LoggedAddress::Physical(rsdp_addr as u64));
        let tables = acpi::AcpiTables::from_rsdp(acpi_handler, rsdp_addr)
            .expect("Failed to parse ACPI tables.");

        for (physical_address, header) in tables.table_headers() {
            debug!("Found ACPI table: {:?}", header.signature.as_str());
        }

        debug!("ACPI Revision: {:?}", tables.rsdp_revision);

        PLATFORM = Some(AcpiPlatform::new(tables, acpi_handler).unwrap());
        debug!(
            "ACPI Power Profile: {:?}",
            PLATFORM.as_ref().unwrap().power_profile
        );
        PLATFORM.as_ref().unwrap().enter_acpi_mode().unwrap();
        debug!("ACPI Enabled");

        info!(
            "Interrupt Mode: {:?}",
            PLATFORM.as_ref().unwrap().interrupt_model
        )
    }
}

pub fn setup_hpet() {
    unsafe {
        let (hpet_addr, head) = PLATFORM
            .as_ref()
            .unwrap()
            .tables
            .table_headers()
            .find(|(physical_address, header)| header.signature == acpi::sdt::Signature::HPET)
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
        info!("HPET Current Value: {:?}", poll_hpet());
    }
}

pub fn poll_hpet() -> u64 {
    let period = unsafe { volatile_load(HPET_BASE as *const u64) } >> 32;
    let count = unsafe { volatile_load((HPET_BASE + 0xF0) as *const u64) };
    count * period
}

static mut PLATFORM: Option<AcpiPlatform<AcpiHandler>> = None;
static mut APIC_BASE: u64 = ERROR_ADDRESS;
static mut HPET_BASE: u64 = ERROR_ADDRESS; // this is an error value which *should*

fn get_apic_base() -> u64 {
    unsafe { x86_64::registers::model_specific::ApicBase::MSR.read() }
}

pub fn setup_apic() {
    if !CPU_FLAGS.contains(CPUFlags::APIC) {
        panic!("APIC is not available");
    }

    unsafe {
        x86_64::registers::model_specific::ApicBase::MSR.write(get_apic_base());
        APIC_BASE = get_apic_base();
        memory::map_identity(APIC_BASE..(APIC_BASE + 0x03F0));

        write_reg(0xF0, read_reg(0xF0) | 0x100);
    }
    info!("APIC initialized");
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
