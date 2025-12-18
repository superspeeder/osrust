pub mod frame_allocator;
pub mod allocator;

use core::iter::TrustedRandomAccessNoCoerce;
use crate::logger::{IntoLoggedAddress, LoggedAddress};
use bootloader_api::BootInfo;
use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use core::ops::Range;
use log::{debug, info, trace, warn};
use x86_64::structures::paging::frame::{PhysFrameRange, PhysFrameRangeInclusive};
use x86_64::structures::paging::mapper::{CleanUp, MapToError, MapperFlush};
use x86_64::structures::paging::page_table::PageTableLevel;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size1GiB,
    Size2MiB, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};
use crate::memory::frame_allocator::boot_info::BootInfoFrameAllocator;

static mut PHYSICAL_OFFSET: VirtAddr = VirtAddr::new(0);
static mut PAGE_TABLE: Option<OffsetPageTable<'static>> = None;
static mut FRAME_ALLOCATOR: Option<BootInfoFrameAllocator> = None;

// This is an invalid address since this is not a canonical address and has enough free space in its address space to withstand an offset without an integer overflow.
// This address should theoretically never be mapped which is the only reason this is safe to use as an error.
// When used in the kernel, this address will cause the kernel to panic.
pub const ERROR_ADDRESS: u64 = (!0u64) ^ (1 << 63 | 0x00FF_FFFF_FFFF);

pub fn mapper() -> &'static mut OffsetPageTable<'static> {
    unsafe { PAGE_TABLE.as_mut().unwrap() }
}

pub fn init(boot_info: &'static BootInfo) {
    unsafe {
        PHYSICAL_OFFSET =
            VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap_or(0));
        info!("Physical Memory Offset: {:?}", PHYSICAL_OFFSET);
        PAGE_TABLE = Some(OffsetPageTable::new(
            active_level_4_table(PHYSICAL_OFFSET),
            PHYSICAL_OFFSET,
        ));
        FRAME_ALLOCATOR = Some(BootInfoFrameAllocator::init(&boot_info.memory_regions));
        info!("Initialized Page Table");
        // <dyn Mapper<Size2MiB>>::map_to(PAGE_TABLE.as_mut().unwrap_unchecked(), Page::containing_address(VirtAddr::new(0)), PhysFrame::containing_address());
    }
}

// This function is unsafe because it expects that it is safe to remove any page mapping in the given range.
pub unsafe fn force_map_region(start: VirtAddr, physical_range: Range<u64>) {
    unsafe {
        // mapper().clean_up_addr_range()
    }
}

pub fn map_region(start: VirtAddr, physical_range: Range<u64>) {
    let mut cursor: u64 = 0;
    while cursor < physical_range.size() as u64 {
        // if the cursor is part of the first 4KiB of a 1GiB huge page and the requested range is at least 1 GiB, map the huge page.
        if (start.as_u64() + cursor) & 0x3ffff000 == 0 && physical_range.size() >= 0x40000000 {
            let virt_cursor = VirtAddr::new(start.as_u64() + cursor);
            let phys_cursor = PhysAddr::new(physical_range.start + cursor);
            unsafe {
                let page = Page::<Size1GiB>::containing_address(virt_cursor);
                let frame = PhysFrame::<Size1GiB>::containing_address(phys_cursor);

                // The page is already mapped as expected
                if let Ok(frame) = mapper().translate_page(page) && frame.start_address() == phys_cursor {
                    continue;
                }

                // Attempt to map a 1GiB page
                match mapper().map_to(
                    page,
                    frame,
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::HUGE_PAGE,
                    frame_allocator::frame_allocator(),
                ) {
                    Ok(mapped_frame) => {
                        trace!(
                            "Mapped 1GiB at {:?} -> {:?}",
                            page.start_address().into_log(),
                            frame.start_address().into_log(),
                        );
                        continue;
                    }
                    Err(MapToError::FrameAllocationFailed) => {
                        panic!("Failed to allocate frame for page tables");
                    }
                    Err(MapToError::PageAlreadyMapped(existing_frame)) => {
                        if existing_frame.start_address() != frame.start_address() {
                            panic!(
                                "Cannot identity map page at {:?}, already mapped to different address {:?}",
                                LoggedAddress::Virtual(frame.start_address().as_u64()),
                                LoggedAddress::Physical(existing_frame.start_address().as_u64())
                            )
                        } else {
                            trace!(
                                "Not mapping page at {:?}: Already mapped.",
                                LoggedAddress::Virtual(frame.start_address().as_u64())
                            )
                        }
                    }
                    Err(MapToError::ParentEntryHugePage) => {
                        panic!(
                            "Physical memory mapper believes that you can have a 512GiB wide page, this is not true and indicates some deeper issue with the kernel."
                        );
                    }
                }
            }
        }

        if (start.as_u64() + cursor) & 0x1fffff == 0 && physical_range.size() >= 0x200000 {
            let virt_cursor = VirtAddr::new(start.as_u64() + cursor);
            let phys_cursor = PhysAddr::new(physical_range.start + cursor);
            unsafe {
                let page = Page::<Size2MiB>::containing_address(virt_cursor);
                let frame = PhysFrame::<Size2MiB>::containing_address(phys_cursor);
                match mapper().map_to(
                    page,
                    frame,
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::HUGE_PAGE,
                    frame_allocator::frame_allocator(),
                ) {
                    Ok(mapped_frame) => {
                        trace!(
                            "Mapped 2MiB at {:?} -> {:?}",
                            page.start_address().into_log(),
                            frame.start_address().into_log(),
                        );
                    }
                    _ => todo!(),
                }
            }
        }
    }
}

pub fn map_identity(range: Range<u64>) -> PhysFrameRangeInclusive {
    let range = PhysFrame::range_inclusive(
        PhysFrame::containing_address(PhysAddr::new(range.start)),
        PhysFrame::containing_address(PhysAddr::new(range.end - 1)),
    ); // avoid mapping an extra frame when it's not necessary
    trace!(
        "Identity mapping range {:?}:{:?}",
        LoggedAddress::Physical(range.start.start_address().as_u64()),
        LoggedAddress::Physical(range.end.start_address().as_u64() + range.end.size())
    );

    for frame in range {
        unsafe {
            match PAGE_TABLE.as_mut().unwrap().identity_map(
                frame,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                FRAME_ALLOCATOR.as_mut().unwrap(),
            ) {
                Ok(mapped_frame) => {
                    trace!(
                        "Mapping 4KiB at {:?} -> {:?}",
                        LoggedAddress::Physical(frame.start_address().as_u64()),
                        LoggedAddress::Virtual(frame.start_address().as_u64())
                    );
                    mapped_frame.flush();
                }
                Err(e) => match e {
                    MapToError::FrameAllocationFailed => {
                        panic!("Failed to allocate frame for page tables");
                    }
                    MapToError::ParentEntryHugePage => {
                        warn!(
                            "Not mapping page at {:?}: Included in mapped huge page (unknown if mapping is correct).",
                            LoggedAddress::Virtual(frame.start_address().as_u64())
                        )
                    }
                    MapToError::PageAlreadyMapped(existing_frame) => {
                        if existing_frame.start_address() != frame.start_address() {
                            panic!(
                                "Cannot identity map page at {:?}, already mapped to different address {:?}",
                                LoggedAddress::Virtual(frame.start_address().as_u64()),
                                LoggedAddress::Physical(existing_frame.start_address().as_u64())
                            )
                        } else {
                            trace!(
                                "Not mapping page at {:?}: Already mapped.",
                                LoggedAddress::Virtual(frame.start_address().as_u64())
                            )
                        }
                    }
                },
            }
        }
    }

    range
}

// pub unsafe fn init(boot_info: &BootInfo) -> OffsetPageTable<'static> {
//     unsafe {
//         PHYSICAL_OFFSET = VirtAddr::new(
//             if let bootloader_api::info::Optional::Some(v) = boot_info.physical_memory_offset {
//                 v
//             } else {
//                 0
//             },
//         );
//     }
//
//     unsafe {
//         let level_4_table = active_level_4_table(PHYSICAL_OFFSET);
//         OffsetPageTable::new(level_4_table, PHYSICAL_OFFSET)
//     }
// }
//
// #[inline]
// pub unsafe fn physical_pointer<T: Sized>(phys: PhysAddr) -> *mut T {
//     unsafe { (phys.as_u64() + PHYSICAL_OFFSET.as_u64()) as *mut T }
// }
//
// #[inline]
// pub unsafe fn physical_ref<T: Sized>(phys: PhysAddr) -> &'static mut T {
//     unsafe { &mut *physical_pointer(phys) }
// }
//

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}


// /// Creates an example mapping for the given page to frame `0xb8000`.
// pub fn create_example_mapping(
//     page: Page,
//     mapper: &mut OffsetPageTable,
//     frame_allocator: &mut impl FrameAllocator<Size4KiB>,
// ) {
//     use x86_64::structures::paging::PageTableFlags as Flags;
//
//     let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
//     let flags = Flags::PRESENT | Flags::WRITABLE;
//
//     let map_to_result = unsafe {
//         // FIXME: this is not safe, we do it only for testing
//         mapper.map_to(page, frame, flags, frame_allocator)
//     };
//     map_to_result.expect("map_to failed").flush();
// }

