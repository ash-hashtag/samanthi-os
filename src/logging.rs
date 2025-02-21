use core::fmt::Write;

use log::{LevelFilter, Metadata, Record};
use spin::Mutex;

use crate::serial_println;
use lazy_static::lazy_static;
extern crate alloc;
use alloc::string::String;

pub struct KernelLogger;

lazy_static! {
    pub static ref LOGS: Mutex<String> = Mutex::new(String::new());
}

impl log::Log for KernelLogger {
    #[inline]
    fn enabled(&self, _meta: &Metadata) -> bool {
        // TOOD: Add level based filtering
        true
    }

    fn log(&self, record: &Record) {
        // let level = record.level();

        // if level <= LevelFilter::Trace {
        serial_println!(
            "{:8} {:5} {}",
            record.target(),
            record.level(),
            record.args()
        );

        {
            let mut s = LOGS.lock();
            s.write_fmt(format_args!(
                "{:8} {:5} {}\n",
                record.target(),
                record.level(),
                record.args()
            ))
            .unwrap();
        }
        // }

        // if level <= LevelFilter::Info {
        // serial_print!(
        //     "{:20} {:5} {}",
        //     record.target(),
        //     record.level(),
        //     record.args()
        // );
        // }
    }

    fn flush(&self) {
        // TODO: Will be used in future for dmesg
    }
}

static KERNEL_LOGGER: KernelLogger = KernelLogger;

pub fn init() {
    // unuse the result
    let _ = log::set_logger(&KERNEL_LOGGER);
    log::set_max_level(LevelFilter::Debug);
}
