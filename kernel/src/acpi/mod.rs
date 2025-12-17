use crate::binutil::checksum_bytes;
use crate::memory;
use crate::memory::physical_pointer;
use bootloader_api::BootInfo;
use core::marker::PhantomData;
use x86_64::PhysAddr;

mod global {
    pub(super) static mut FADT: Option<super::FADT<'static>> = None;
}

#[repr(C, packed)]
#[derive(Copy, Clone, bytemuck::NoUninit)]
pub struct RSDP {
    pub signature: [u8; 8],
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub rsdt_address: u32,
}

#[repr(C, packed)]
#[derive(Copy, Clone, bytemuck::NoUninit)]
pub struct XSDP {
    pub signature: [u8; 8],
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub rsdt_address: u32,
    pub length: u32,
    pub xsdt_address: u64,
    pub extended_checksum: u8,
    pub reserved: [u8; 3],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::NoUninit, Debug)]
pub struct SDTHeader {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,
}

#[repr(transparent)]
pub struct RSDT {
    header: SDTHeader,
}

#[repr(transparent)]
pub struct XSDT {
    header: SDTHeader,
}

pub struct SDTIterator<'a> {
    current_table_ptr: *const *const SDTHeader,
    iter_end: *const *const SDTHeader,
    extended: bool,
    _phantom: PhantomData<&'a SDTHeader>,
}

pub enum SystemDescriptorTable<'a> {
    Unknown(&'a SDTHeader),
    MADT(&'a MADT),
    FADT(&'a FADT1),
    FADT2(&'a FADT2),
}

pub enum FADT<'a> {
    V1(&'a FADT1),
    V2(&'a FADT2),
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct ProcessorLocalAPIC {
    pub processor_id: u8,
    pub apic_id: u8,
    pub flags: u32,
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct IOAPIC {
    pub id: u8,
    _reserved: u8,
    pub address: u32,
    pub global_system_interrupt_base: u32,
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct IOAPICInterruptSourceOverride {
    pub bus_source: u8,
    pub irq_source: u8,
    pub global_system_interrupt: u32,
    pub flags: u16,
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct IOAPICNonMaskableInterruptSource {
    pub nmi_source: u8,
    _reserved: u8,
    pub flags: u16,
    pub global_system_interrupt: u32,
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct LocalAPICNonMaskableInterrupts {
    pub processor_id: u8,
    pub flags: u16,
    pub lint_entry: u8,
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct LocalAPICAddressOverride {
    _reserved: u16,
    pub address: u64,
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct ProcessorLocalX2APIC {
    _reserved: u16,
    pub x2apic_id: u32,
    pub flags: u32,
    pub acpi_id: u32,
}

pub trait MADTCallback {
    fn processor_local_apic(&mut self, processor_local_apic: &ProcessorLocalAPIC);
    fn io_apic(&mut self, io_apic: &IOAPIC);
    fn io_apic_interrupt_source_override(
        &mut self,
        io_apic_interrupt_source_override: &IOAPICInterruptSourceOverride,
    );
    fn io_apic_nmi_source(&mut self, io_apic_nmi_source: &IOAPICNonMaskableInterruptSource);
    fn local_apic_nmis(&mut self, local_apic_nmis: &LocalAPICNonMaskableInterrupts);
    fn local_apic_address_override(
        &mut self,
        local_apic_address_override: &LocalAPICAddressOverride,
    );
    fn processor_local_x2apic(&mut self, processor_local_x2apic: &ProcessorLocalX2APIC);
}

#[repr(C)]
pub struct MADT {
    header: SDTHeader,
    pub local_apic_address: u32,
    pub flags: u32,
}

#[repr(C, packed)]
struct MADTEntryHeader {
    entry_type: u8,
    record_length: u8,
}

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct PowerManagementProfile(u8);

impl PowerManagementProfile {
    pub const UNSPECIFIED: Self = Self(0);
    pub const DESKTOP: Self = Self(1);
    pub const MOBILE: Self = Self(2);
    pub const WORKSTATION: Self = Self(3);
    pub const ENTERPRISE_SERVER: Self = Self(4);
    pub const SOHO_SERVER: Self = Self(5);
    pub const APLLIANCE_PC: Self = Self(6);
    pub const PERFORMANCE_SERVER: Self = Self(7);

    pub fn is_valid(self) -> bool {
        self.0 > 7
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct FADT1 {
    header: SDTHeader,
    pub firmware_control: u32,
    pub dsdt: u32,
    _reserved: u8,
    pub preferred_power_management_profile: PowerManagementProfile,
    pub sci_interrupt: u16,
    pub smi_command_port: u32,
    pub acpi_enable: u8,
    pub acpi_disable: u8,
    pub s4bios_req: u8,
    pub pstate_control: u8,
    pub pm1a_event_block: u32,
    pub pm1b_event_block: u32,
    pub pm1a_control_block: u32,
    pub pm1b_control_block: u32,
    pub pm2_control_block: u32,
    pub pm_timer_block: u32,
    pub gpe0_block: u32,
    pub gpe1_block: u32,
    pub pm1_event_length: u8,
    pub pm1_control_length: u8,
    pub pm2_control_length: u8,
    pub pm_timer_length: u8,
    pub gpe0_length: u8,
    pub gpe1_length: u8,
    pub gpe1_base: u8,
    pub cstate_control: u8,
    pub worse_c2_latency: u16,
    pub worse_c3_latency: u16,
    pub flush_size: u16,
    pub flush_stride: u16,
    pub duty_offset: u8,
    pub duty_width: u8,
    pub day_alarm: u8,
    pub month_alarm: u8,
    pub century: u8,
}

// used when ACPI version is > 2.0
#[repr(C)]
#[derive(Debug)]
pub struct FADT2 {
    header: SDTHeader,
    pub firmware_control: u32,
    pub dsdt: u32,
    _reserved: u8,
    pub preferred_power_management_profile: PowerManagementProfile,
    pub sci_interrupt: u16,
    pub smi_command_port: u32,
    pub acpi_enable: u8,
    pub acpi_disable: u8,
    pub s4bios_req: u8,
    pub pstate_control: u8,
    pub pm1a_event_block: u32,
    pub pm1b_event_block: u32,
    pub pm1a_control_block: u32,
    pub pm1b_control_block: u32,
    pub pm2_control_block: u32,
    pub pm_timer_block: u32,
    pub gpe0_block: u32,
    pub gpe1_block: u32,
    pub pm1_event_length: u8,
    pub pm1_control_length: u8,
    pub pm2_control_length: u8,
    pub pm_timer_length: u8,
    pub gpe0_length: u8,
    pub gpe1_length: u8,
    pub gpe1_base: u8,
    pub cstate_control: u8,
    pub worse_c2_latency: u16,
    pub worse_c3_latency: u16,
    pub flush_size: u16,
    pub flush_stride: u16,
    pub duty_offset: u8,
    pub duty_width: u8,
    pub day_alarm: u8,
    pub month_alarm: u8,
    pub century: u8,

    pub boot_architecture_flags: u16,
    _reserved2: u8,
    pub flags: u32,
    pub reset_reg: GenericAddressStructure,

    pub reset_value: u8,
    _reserved3: [u8; 3],

    pub x_firmware_control: u64,
    pub x_dsdt: u64,

    pub x_pm1a_event_block: GenericAddressStructure,
    pub x_pm1b_event_block: GenericAddressStructure,
    pub x_pm1a_control_block: GenericAddressStructure,
    pub x_pm1b_control_block: GenericAddressStructure,
    pub x_pm2_control_block: GenericAddressStructure,
    pub x_pm_timer_block: GenericAddressStructure,
    pub x_gpe0_block: GenericAddressStructure,
    pub x_gpe1_block: GenericAddressStructure,
}

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct AddressSpace(u8);

impl AddressSpace {
    pub const SYSTEM_MEMORY: AddressSpace = AddressSpace(0);
    pub const SYSTEM_IO: AddressSpace = AddressSpace(1);
    pub const PCI_CONFIGURATION_SPACE: AddressSpace = AddressSpace(2);
    pub const EMBEDDED_CONTROLLER: AddressSpace = AddressSpace(3);
    pub const SYSTEM_MANAGEMENT_BUS: AddressSpace = AddressSpace(4);
    pub const SYSTEM_CMOS: AddressSpace = AddressSpace(5);
    pub const PCI_DEVICE_BAR_TARGET: AddressSpace = AddressSpace(6);
    pub const IPMI: AddressSpace = AddressSpace(7);
    pub const GPIO: AddressSpace = AddressSpace(8);
    pub const GENERIC_SERIAL_BUS: AddressSpace = AddressSpace(9);
    pub const PLATFORM_COMMUNICATION_CHANNEL: AddressSpace = AddressSpace(0x0A);
}

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct AccessSize(u8);

impl AccessSize {
    pub const UNDEFINED: AccessSize = AccessSize(0);
    pub const BYTE: AccessSize = AccessSize(1);
    pub const WORD: AccessSize = AccessSize(2);
    pub const DWORD: AccessSize = AccessSize(3);
    pub const QWORD: AccessSize = AccessSize(4);
}

#[repr(C)]
#[derive(Debug)]
pub struct GenericAddressStructure {
    pub address_space: AddressSpace,
    pub bit_width: u8,
    pub bit_offset: u8,
    pub access_size: AccessSize,
    pub address: u64,
}

pub fn init(boot_info: &'static BootInfo) {
    if let bootloader_api::info::Optional::Some(rsdp_addr) = boot_info.rsdp_addr {
        unsafe {
            let rsdp: &'static RSDP = memory::physical_ref(PhysAddr::new(rsdp_addr));
            rsdp.validate().expect("Invalid RSDP structure");

            println!("ACPI Revision: {:?}", rsdp.revision);

            if rsdp.revision >= 2 {
                let xsdp: &'static XSDP = memory::physical_ref(PhysAddr::new(rsdp_addr));
                xsdp.validate().expect("Invalid XSDP structure");
                init_from_xsdt(memory::physical_ref(PhysAddr::new(xsdp.xsdt_address)));
            } else {
                init_from_rsdt(memory::physical_ref(PhysAddr::new(
                    rsdp.rsdt_address as u64,
                )));
            }
        }
    }
}

fn init_from_xsdt(xsdt: &'static XSDT) {
    xsdt.validate().expect("Invalid XSDT structure");
    println!("ACPI: Reading XSDT at 0x{:016X}", &raw const *xsdt as u64);
    for table in xsdt.iter() {
        match table {
            SystemDescriptorTable::Unknown(header) => unsafe {
                println!(
                    "ACPI: Unknown system descriptor table: \"{}\"",
                    core::str::from_utf8_unchecked(&header.signature)
                );
            },
            SystemDescriptorTable::MADT(madt) => {
                init_from_madt(madt);
            }
            SystemDescriptorTable::FADT(fadt) => {
                init_from_fadt(fadt);
            }
            SystemDescriptorTable::FADT2(fadt2) => {
                init_from_fadt2(fadt2);
            }
        }
    }
}

fn init_from_rsdt(rsdt: &'static RSDT) {
    rsdt.validate().expect("Invalid RSDT structure");
    println!("ACPI: Reading RSDT at 0x{:016X}", &raw const *rsdt as u64);
    for table in rsdt.iter() {
        match table {
            SystemDescriptorTable::Unknown(header) => unsafe {
                println!(
                    "ACPI: Unknown system descriptor table: \"{}\"",
                    core::str::from_utf8_unchecked(&header.signature)
                );
            },
            SystemDescriptorTable::MADT(madt) => {
                init_from_madt(madt);
            }
            SystemDescriptorTable::FADT(fadt) => {
                init_from_fadt(fadt);
            }
            SystemDescriptorTable::FADT2(fadt) => {
                init_from_fadt2(fadt);
            }
        }
    }
}

fn init_from_fadt(fadt: &'static FADT1) {
    println!("ACPI: Found FADT: {:#?}", fadt);
    unsafe {
        global::FADT = Some(FADT::V1(fadt));
    };
}

fn init_from_fadt2(fadt: &'static FADT2) {
    println!("ACPI: Found FADT: {:#?}", fadt);
    unsafe {
        global::FADT = Some(FADT::V2(fadt));
    };
}

struct MadtProcessor;

impl MADTCallback for MadtProcessor {
    fn processor_local_apic(&mut self, processor_local_apic: &ProcessorLocalAPIC) {
        println!(
            "ACPI: MADT: Processor Local APIC {:#?}",
            processor_local_apic
        );
    }

    fn io_apic(&mut self, io_apic: &IOAPIC) {
        println!("ACPI: MADT: IO APIC {:#?}", io_apic);
    }

    fn io_apic_interrupt_source_override(
        &mut self,
        io_apic_interrupt_source_override: &IOAPICInterruptSourceOverride,
    ) {
        println!(
            "ACPI: MADT: IO APIC Interrupt Source Override {:#?}",
            io_apic_interrupt_source_override
        );
    }

    fn io_apic_nmi_source(&mut self, io_apic_nmi_source: &IOAPICNonMaskableInterruptSource) {
        println!("ACPI: MADT: IO APIC NMI Source {:#?}", io_apic_nmi_source);
    }

    fn local_apic_nmis(&mut self, local_apic_nmis: &LocalAPICNonMaskableInterrupts) {
        println!("ACPI: MADT: Local APIC NMIs {:#?}", local_apic_nmis);
    }

    fn local_apic_address_override(
        &mut self,
        local_apic_address_override: &LocalAPICAddressOverride,
    ) {
        println!(
            "ACPI: MADT: Local APIC Address Override {:#?}",
            local_apic_address_override
        );
    }

    fn processor_local_x2apic(&mut self, processor_local_x2apic: &ProcessorLocalX2APIC) {
        println!(
            "ACPI: MADT: Processor Local X2APIC {:#?}",
            processor_local_x2apic
        );
    }
}

fn init_from_madt(madt: &'static MADT) {
    madt.validate().expect("Invalid MADT structure");
    let mut processor: MadtProcessor = MadtProcessor;
    madt.enumerate_tables(&mut processor);
}

impl RSDP {
    fn validate(&self) -> Option<()> {
        if self.signature != *b"RSD PTR " {
            return None;
        }

        if checksum_bytes(self) != 0 {
            return None;
        }

        Some(())
    }
}

impl XSDP {
    fn validate(&self) -> Option<()> {
        if self.signature != *b"RSD PTR " {
            return None;
        }

        if checksum_bytes(self) != 0 {
            return None;
        }

        Some(())
    }
}

impl SDTHeader {
    pub fn validate(&self, signature: &'static [u8; 4]) -> Option<()> {
        if self.signature != *signature {
            return None;
        }

        self.validate_checksum()
    }

    pub fn validate_checksum(&self) -> Option<()> {
        if self.length < size_of::<Self>() as u32 {
            return None;
        }

        let mut sum = 0u8;
        let base = core::ptr::from_ref(self) as *const u8;
        for i in 0..self.length {
            unsafe { sum = sum.wrapping_add(*base.offset(i as isize)) }
        }

        if sum != 0 {
            return None;
        }

        Some(())
    }
}

impl RSDT {
    pub fn validate(&self) -> Option<()> {
        self.header.validate(table_signatures::RSDT)
    }

    pub fn iter(&self) -> SDTIterator<'_> {
        SDTIterator {
            current_table_ptr: unsafe {
                core::ptr::from_ref(self).byte_offset(size_of::<Self>() as isize) as *const _
            },
            iter_end: unsafe {
                core::ptr::from_ref(self).byte_offset(self.header.length as isize) as *const _
            },
            extended: false,
            _phantom: PhantomData,
        }
    }
}

impl XSDT {
    pub fn validate(&self) -> Option<()> {
        self.header.validate(table_signatures::XSDT)
    }

    pub fn iter(&self) -> SDTIterator<'_> {
        SDTIterator {
            current_table_ptr: unsafe {
                core::ptr::from_ref(self).byte_offset(size_of::<Self>() as isize) as *const _
            },
            iter_end: unsafe {
                core::ptr::from_ref(self).byte_offset(self.header.length as isize) as *const _
            },
            extended: true,
            _phantom: PhantomData,
        }
    }
}

impl<'a> Iterator for SDTIterator<'a> {
    type Item = SystemDescriptorTable<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.iter_end.addr() <= self.current_table_ptr.addr() {
            return None;
        }

        let table_addr = unsafe { PhysAddr::new((*self.current_table_ptr) as u64) };
        let table_pointer: *const SDTHeader = unsafe { physical_pointer(table_addr) };
        println!("ACPI: Reading SDT at 0x{:016X}", table_pointer as u64);

        let rv = unsafe {
            let header = &*table_pointer;
            header
                .validate_checksum()
                .expect("Invalid ACPI table checksum");

            println!(
                "ACPI: Found SDT with signature \"{}\"",
                core::str::from_utf8_unchecked(&header.signature)
            );

            match &header.signature {
                table_signatures::MADT => Some(SystemDescriptorTable::MADT(
                    &*(table_pointer as *const MADT),
                )),
                table_signatures::FADT => {
                    if self.extended {
                        Some(SystemDescriptorTable::FADT2(
                            &*(table_pointer as *const FADT2),
                        ))
                    } else {
                        Some(SystemDescriptorTable::FADT(
                            &*(table_pointer as *const FADT1),
                        ))
                    }
                }
                _ => Some(SystemDescriptorTable::Unknown(header)),
            }
        };

        unsafe {
            self.current_table_ptr = self.current_table_ptr.offset(1);
        }

        rv
    }
}

impl MADT {
    pub fn validate(&self) -> Option<()> {
        self.header.validate(table_signatures::MADT)
    }

    pub fn enumerate_tables<T: MADTCallback>(&self, processor: &mut T) {
        let mut ptr = unsafe {
            (self as *const _ as *const MADTEntryHeader).byte_offset(size_of::<Self>() as isize)
        };
        let end = unsafe {
            (self as *const _ as *const MADTEntryHeader).byte_offset(self.header.length as isize)
        };

        while ptr.addr() < end.addr() {
            unsafe {
                match (*ptr).entry_type {
                    0 => processor.processor_local_apic(
                        &*(ptr.byte_offset(size_of::<MADTEntryHeader>() as isize) as *const _),
                    ),
                    1 => processor.io_apic(
                        &*(ptr.byte_offset(size_of::<MADTEntryHeader>() as isize) as *const _),
                    ),
                    2 => processor.io_apic_interrupt_source_override(
                        &*(ptr.byte_offset(size_of::<MADTEntryHeader>() as isize) as *const _),
                    ),
                    3 => processor.io_apic_nmi_source(
                        &*(ptr.byte_offset(size_of::<MADTEntryHeader>() as isize) as *const _),
                    ),
                    4 => processor.local_apic_nmis(
                        &*(ptr.byte_offset(size_of::<MADTEntryHeader>() as isize) as *const _),
                    ),
                    5 => processor.local_apic_address_override(
                        &*(ptr.byte_offset(size_of::<MADTEntryHeader>() as isize) as *const _),
                    ),
                    9 => processor.processor_local_x2apic(
                        &*(ptr.byte_offset(size_of::<MADTEntryHeader>() as isize) as *const _),
                    ),
                    type_id => {
                        println!("Encountered unknown ACPI MADT entry type {:?}.", type_id);
                    }
                }
            }

            ptr = unsafe { ptr.byte_offset((*ptr).record_length as isize) };
        }
    }
}

pub mod table_signatures {
    pub const MADT: &'static [u8; 4] = b"APIC";
    pub const BGRT: &'static [u8; 4] = b"BGRT";
    pub const BERT: &'static [u8; 4] = b"BERT";
    pub const CPEP: &'static [u8; 4] = b"CPEP";
    pub const DSDT: &'static [u8; 4] = b"DSDT";
    pub const ECDT: &'static [u8; 4] = b"ECDT";
    pub const EINJ: &'static [u8; 4] = b"EINJ";
    pub const ERST: &'static [u8; 4] = b"ERST";
    pub const FADT: &'static [u8; 4] = b"FACP";
    pub const FACS: &'static [u8; 4] = b"FACS";
    pub const HEST: &'static [u8; 4] = b"HEST";
    pub const MSCT: &'static [u8; 4] = b"MSCT";
    pub const MPST: &'static [u8; 4] = b"MPST";
    pub const PMTT: &'static [u8; 4] = b"PMTT";
    pub const PSDT: &'static [u8; 4] = b"PSDT";
    pub const RASF: &'static [u8; 4] = b"RASF";
    pub const RSDT: &'static [u8; 4] = b"RSDT";
    pub const SBST: &'static [u8; 4] = b"SBST";
    pub const SLIT: &'static [u8; 4] = b"SLIT";
    pub const SRAT: &'static [u8; 4] = b"SRAT";
    pub const SSDT: &'static [u8; 4] = b"SSDT";
    pub const XSDT: &'static [u8; 4] = b"XSDT";
}
