#![no_std]
#![no_main]
#![allow(dead_code, unused)]
#![feature(custom_test_frameworks)]
#![test_runner(samanthi::test_runner)]
#![reexport_test_harness_main = "test_main"]
// #![feature(panic_info_message)]
// #![feature(const_mut_refs)]
extern crate alloc;
extern crate core;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::{string::ToString, vec::Vec};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use samanthi::drivers::network::get_virtio_network_device;
use samanthi::drivers::pci::detect_devices;
use samanthi::memory::create_mapping;
use samanthi::task::executor::Executor;
use samanthi::task::keyboard::init_memory_fs;
use samanthi::task::simple_executor::SimpleExecutor;
use samanthi::task::{keyboard, Task};
use samanthi::vga_buffer::{
    Buffer, Color, ColorCode, ScreenChar, BUFFER_HEIGHT, BUFFER_WIDTH, WRITER,
};
use samanthi::{
    allocator, hlt_loop,
    memory::{self, translate_addr},
    println,
};
use samanthi::{logging, serial_println};
use vga::writers::{Graphics320x200x256, GraphicsWriter, PrimitiveDrawing};
use x86_64::structures::paging::PhysFrame;
use x86_64::PhysAddr;
use x86_64::{
    structures::paging::{Page, PageTable, Translate},
    VirtAddr,
};

entry_point!(kernal_main);

fn kernal_main(boot_info: &'static BootInfo) -> ! {
    samanthi::init();
    logging::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator);

    log::info!("Booted Into Samanthi");

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
    log::info!("Keyboard handler initialized");

    let device = get_virtio_network_device().unwrap();
    device.map_bars_to_virtual_addresses(&mut mapper, &mut frame_allocator);
    device.setup();

    // {
    //     {
    //         WRITER.lock().print_frame_buffer_address();
    //     }
    //     let virt_addr = 0x4444_0000;

    //     let page = Page::containing_address(VirtAddr::new(virt_addr));
    //     let phys_addr = 0xb8000;
    //     let phys_frame = PhysFrame::from_start_address(PhysAddr::new(phys_addr)).unwrap();

    //     create_mapping(page, phys_frame, &mut mapper, &mut frame_allocator);

    //     let phys_addr = mapper.translate_addr(VirtAddr::new(virt_addr)).unwrap();
    //     log::info!(
    //         "Mapped virt: {:x} to phys: {:x}",
    //         virt_addr,
    //         phys_addr.as_u64()
    //     );
    //     unsafe {
    //         let mut buffer = &mut *(virt_addr as *mut Buffer);
    //         // let mut buffer = &mut *(phys_addr.as_u64() as *mut Buffer);

    //         let c = ScreenChar {
    //             ascii_character: 'a' as u8,
    //             color_code: ColorCode::new(Color::Blue, Color::Green),
    //         };

    //         for x in 0..BUFFER_WIDTH {
    //             for y in 0..BUFFER_HEIGHT {
    //                 buffer.chars[y][x].write(c);
    //             }
    //         }
    //     }
    // }
    // device.find_mmio_bar(&mut frame_allocator, &mut mapper);

    // device.is_mmio_enabled();
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
