use crate::acpi::init::acpi_platform;
use crate::logger::LoggedAddress;
use crate::memory;
use crate::memory::ERROR_ADDRESS;
use acpi::sdt::mcfg::Mcfg;
use log::{debug, info};

static mut PCIE_BASE_ADDR: u64 = ERROR_ADDRESS;

pub fn init() {
    unsafe {
        let (mcfg_addr, head) = acpi_platform()
            .tables
            .table_headers()
            .find(|(_physical_address, header)| header.signature == acpi::sdt::Signature::MCFG)
            .expect("PCIe is not supported");

        let mcfg = &*(mcfg_addr as *const Mcfg);
        let entries = mcfg.entries();
        if entries.len() != 1 {
            unimplemented!(
                "Non-contiguous PCIe configuration space/multiple PCI segment groups are not yet supported"
            );
        }

        let entry = entries.first().unwrap_unchecked();
        PCIE_BASE_ADDR = entry.base_address;
        debug!(
            "Found PCIe configuration space at {:?}",
            LoggedAddress::Physical(PCIE_BASE_ADDR)
        );
        memory::map_identity(
            PCIE_BASE_ADDR..(PCIE_BASE_ADDR + (entry.bus_number_end as u64 - entry.bus_number_start as u64 + 1) * 4096),
        );
    }
}
