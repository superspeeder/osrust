use x86_64::instructions::port::PortWrite;

pub fn init() {
    unsafe {
        const divisor: u16 = (1193180u32 / 1000) as u16;
        u8::write_to_port(0x43, 0x36);
        u8::write_to_port(0x40, (divisor & 0xff) as u8);
        u8::write_to_port(0x40, ((divisor >> 8) & 0xff) as u8);
    }
}