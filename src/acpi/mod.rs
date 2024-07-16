pub mod power;

use x86_64::instructions::interrupts;

pub fn init() {}

pub fn setup_smp_prerequisities() {
    interrupts::disable();
}
