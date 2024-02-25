#![no_std]
#![no_main]
#![allow(dead_code, unused)]
#![feature(custom_test_frameworks)]
#![test_runner(samanthi::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(panic_info_message)]
extern crate core;
use core::panic::PanicInfo;

use samanthi::{hlt_loop, println};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    samanthi::init();
    let s = "Hello World!";
    println!("{} {}", s.as_ptr() as usize, s);

    #[cfg(test)]
    test_main();
    println!("End Of Execution");

    hlt_loop()
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Panic: {}", info);
    hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    samanthi::test_panic_handler(info)
}

#[test_case]
fn test_trivial() {
    assert_eq!(1, 1);
}
