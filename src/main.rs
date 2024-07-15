#![no_std]
#![no_main]
#![allow(dead_code, unused)]
#![feature(custom_test_frameworks)]
#![test_runner(samanthi::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(panic_info_message)]
extern crate alloc;
extern crate core;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::{string::ToString, vec::Vec};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use samanthi::{
    allocator, hlt_loop,
    memory::{self, translate_addr},
    println,
};
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

    let s = "From Kernel";
    println!("{} {}", s.as_ptr() as usize, s);

    #[cfg(test)]
    test_main();

    // let i4_table = unsafe { active_level_4_table(phys_mem_offset) };

    // for (i, entry) in i4_table.iter().enumerate() {
    //     if !entry.is_unused() {
    //         println!("L4 Entry {}: {:?}", i, entry);

    //         let phys = entry.frame().unwrap().start_address();
    //         let virt = phys.as_u64() + boot_info.physical_memory_offset;
    //         let ptr = VirtAddr::new(virt).as_mut_ptr();

    //         let i3_table: &PageTable = unsafe { &*ptr };

    //         for (i, entry) in i3_table.iter().enumerate() {
    //             if !entry.is_unused() {
    //                 println!("  L3 Entry {}: {:?}", i, entry);
    //             }
    //         }
    //     }
    // }

    // let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    // let addresses = [
    //     0xb8000,
    //     0x201008,
    //     0x0100_0020_1210,
    //     boot_info.physical_memory_offset,
    // ];

    // for &address in &addresses {
    //     let virt = VirtAddr::new(address);

    //     // let phys = unsafe { translate_addr(virt, phys_mem_offset) };
    //     let phys = mapper.translate_addr(virt);
    //     println!("{:?} -> {:?}", virt, phys);
    // }

    // let mut strings = Vec::new();

    // for i in 0..10 {
    //     strings.push(i.to_string());
    // }

    // for s in strings {
    //     println!("{}", s);
    // }

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
