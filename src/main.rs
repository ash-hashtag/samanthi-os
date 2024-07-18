#![no_std]
#![no_main]
#![allow(dead_code, unused)]
#![feature(custom_test_frameworks)]
#![test_runner(samanthi::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(panic_info_message)]
#![feature(const_mut_refs)]
extern crate alloc;
extern crate core;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::{string::ToString, vec::Vec};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use samanthi::drivers::pci::detect_devices;
use samanthi::task::executor::Executor;
use samanthi::task::keyboard::init_memory_fs;
use samanthi::task::simple_executor::SimpleExecutor;
use samanthi::task::{keyboard, Task};
use samanthi::{
    allocator, hlt_loop,
    memory::{self, translate_addr},
    println,
};
use samanthi::{logging, serial_println};
use vga::writers::{Graphics320x200x256, GraphicsWriter, PrimitiveDrawing};
use x86_64::{
    structures::paging::{Page, PageTable, Translate},
    VirtAddr,
};

entry_point!(kernal_main);

fn kernal_main(boot_info: &'static BootInfo) -> ! {
    samanthi::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator);

    logging::init();

    log::info!("Booted Into Samanthi");
    log::info!("Heap Initialized");

    detect_devices();

    init_memory_fs();

    // use x86_64::registers::control::Cr4;

    // let cr4_flags = Cr4::read();

    // println!("CR4 Flags: {:?}", cr4_flags);

    // let mut executor = SimpleExecutor::new();

    // let graphics = Graphics320x200x256::new();
    // graphics.set_mode();
    // graphics.clear_screen(9);
    // graphics.draw_line((0, 0), (200, 200), 13);

    let mut executor = Executor::new();
    // executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

    // #[cfg(test)]
    // test_main();

    // hlt_loop()
}

async fn async_number() -> u32 {
    69
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("Panic: {}", info);
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
