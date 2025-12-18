use core::{
    intrinsics::{volatile_load, volatile_store},
    ptr::NonNull,
};

use acpi::{Handle, PciAddress, PhysicalMapping, aml::AmlError, platform::AcpiPlatform};
use log::{debug, info};
use x86_64::instructions::port::{PortRead, PortWrite};

use crate::{logger::LoggedAddress, memory};

static mut PLATFORM: Option<AcpiPlatform<AcpiHandler>> = None;

#[derive(Copy, Clone)]
pub(super) struct AcpiHandler;

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
            virtual_start: unsafe { NonNull::new_unchecked(physical_address as *mut _) },
            region_length: size,
            mapped_length: mapping.size() as usize,
            handler: self.clone(),
        }
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {
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

    fn read_pci_u8(&self, _address: PciAddress, _offset: u16) -> u8 {
        todo!()
    }

    fn read_pci_u16(&self, _address: PciAddress, _offset: u16) -> u16 {
        todo!()
    }

    fn read_pci_u32(&self, _address: PciAddress, _offset: u16) -> u32 {
        todo!()
    }

    fn write_pci_u8(&self, _address: PciAddress, _offset: u16, _value: u8) {
        todo!()
    }

    fn write_pci_u16(&self, _address: PciAddress, _offset: u16, _value: u16) {
        todo!()
    }

    fn write_pci_u32(&self, _address: PciAddress, _offset: u16, _value: u32) {
        todo!()
    }

    fn nanos_since_boot(&self) -> u64 {
        todo!()
    }

    fn stall(&self, _microseconds: u64) {
        todo!()
    }

    fn sleep(&self, _milliseconds: u64) {
        todo!()
    }

    fn create_mutex(&self) -> Handle {
        todo!()
    }

    fn acquire(&self, _mutex: Handle, _timeout: u16) -> Result<(), AmlError> {
        todo!()
    }

    fn release(&self, _mutex: Handle) {
        todo!()
    }
}

pub fn load_acpi(rsdp_addr: u64) {
    let acpi_handler = AcpiHandler;
    unsafe {
        debug!("RSDP is at {:?}", LoggedAddress::Physical(rsdp_addr));
        let tables = acpi::AcpiTables::from_rsdp(acpi_handler, rsdp_addr as usize)
            .expect("Failed to parse ACPI tables.");

        for (_physical_address, header) in tables.table_headers() {
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

pub(super) unsafe fn acpi_platform() -> &'static AcpiPlatform<AcpiHandler> {
    unsafe { PLATFORM.as_ref().expect("ACPI not initialized") }
}
