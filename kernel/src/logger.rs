use core::fmt::{Debug, Formatter};
use log::{Level, Record};

pub(super) struct SerialLogger;

pub(super) static LOGGER: SerialLogger = SerialLogger;
static mut LOGGER_ALIGNMENT: usize = 0;
const LOGGER_ALIGNMENT_LOWER_THRESHOLD: isize = 24;

fn colorfor(level: log::Level) -> &'static str {
    match level {
        Level::Error => "\x1b[0;91m",
        Level::Warn => "\x1b[0;93m",
        Level::Info => "\x1b[0;92m",
        Level::Debug => "\x1b[0;96m",
        Level::Trace => "\x1b[0;90m",
    }
}

impl log::Log for SerialLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &Record) {
        let module_path = record.module_path().unwrap_or("<unknown>");
        let alignment = unsafe {
            let thisalign = module_path.len() as isize;
            let shift = thisalign - LOGGER_ALIGNMENT as isize;
            if shift > 0 {
                thisalign
            } else if -shift > LOGGER_ALIGNMENT_LOWER_THRESHOLD {
                thisalign
            } else {
                LOGGER_ALIGNMENT as isize
            }
        }.max(0) as usize;
        unsafe { LOGGER_ALIGNMENT = alignment; }

        println!(
            "\x1b[0;2;37m[{level_color}\x1b[1m{level:<5}\x1b[0;2;37m]\x1b[0;1;97m {module_path:<alignment$} \x1b[0;2;37m>\x1b[0;97m {args}\x1b[0m",
            module_path = module_path,
            level_color = colorfor(record.level()),
            level = record.level(),
            args = record.args(),
            alignment = alignment,
        );
    }

    fn flush(&self) {}
}

pub enum LoggedAddress {
    Physical(u64),
    Virtual(u64),
}

impl Debug for LoggedAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            LoggedAddress::Physical(addr) => {
                f.write_fmt(format_args!("\x1b[0;96;4m0x{:016x}\x1b[0;97m", addr))
            }
            LoggedAddress::Virtual(addr) => {
                f.write_fmt(format_args!("\x1b[0;95;4m0x{:016x}\x1b[0;97m", addr))
            }
        }
    }
}
