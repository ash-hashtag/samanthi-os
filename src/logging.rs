use log::{LevelFilter, Metadata, Record};

use crate::serial_println;

pub struct KernelLogger;

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
