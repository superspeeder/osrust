use lazy_static::lazy_static;
use spin::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    use x86_64::instructions::{nop, port::Port};

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }

    loop {
        nop();
    }
}

pub fn serial(base: u16) -> uart_16550::SerialPort {
    let mut port = unsafe { uart_16550::SerialPort::new(base) };
    port.init();
    port
}

lazy_static! {
    pub static ref SERIAL: Mutex<uart_16550::SerialPort> = Mutex::new(serial(0x3F8));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::debug_utils::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::debug_utils::_print("\n"));
    ($($arg:tt)*) => ($crate::debug_utils::_println(format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL.lock().write_fmt(args).unwrap();
}


#[doc(hidden)]
pub fn _println(args: core::fmt::Arguments) {
    use core::fmt::Write;
    let mut serial = SERIAL.lock();
    serial.write_fmt(args).unwrap();
    serial.write_char('\n').unwrap();
}
