pub mod pit;

use lazy_static::lazy_static;
use pic8259::ChainedPics;
use x86_64::instructions::hlt;
use x86_64::instructions::port::PortWrite;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        // unsafe {
        //     idt.double_fault
        //         .set_handler_fn(double_fault_handler)
        //         .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        // }
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.general_protection_fault.set_handler_fn(gpf_handler);

        idt[32].set_handler_fn(timer_interrupt_handler);
        for i in 33..48 {
            idt[i].set_handler_fn(nothing_isr);
        }

        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

pub fn init_pics() {
    // unsafe { PICS.lock().initialize() };
    unsafe {
        u8::write_to_port(0x20, 0x11);
        u8::write_to_port(0xA0, 0x11);
        u8::write_to_port(0x21, 0x20);
        u8::write_to_port(0xA1, 0x28);
        u8::write_to_port(0x21, 0x04);
        u8::write_to_port(0xA1, 0x02);
        u8::write_to_port(0x21, 0x01);
        u8::write_to_port(0xA1, 0x01);
        u8::write_to_port(0x21, 0x0);
        u8::write_to_port(0xA1, 0x0);
    }
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: PageFaultErrorCode,
) {
    println!("EXCEPTION: PAGE FAULT AT 0x{:016X}\n{:#?}", Cr2::read_raw(), stack_frame);
    loop {
        hlt();
    }
}

extern "x86-interrupt" fn gpf_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
    println!("EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}", stack_frame);
    loop {
        hlt();
    }
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn nothing_isr(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}