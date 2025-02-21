pub mod fixed_size_block;
pub mod linked_list;

extern crate alloc;
extern crate core;

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use linked_list_allocator::LockedHeap;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags,
        PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

use self::{
    fixed_size_block::FixedSizeBlockAllocator,
    linked_list::{LinkedListAllocator, Locked},
};

// #[global_allocator]
// static ALLOCATOR: LockedHeap = LockedHeap::empty();
#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());
// #[global_allocator]
// static ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 1024 * 1024;

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,

    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;

        let heap_start_page = Page::containing_address(heap_start);

        let heap_end_page = Page::containing_address(heap_end);

        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
        // ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
    }

    log::info!(
        "Heap Initialized from {}-{}",
        HEAP_START,
        HEAP_SIZE + HEAP_START
    );
    Ok(())
}

fn align_up(addr: usize, align: usize) -> usize {
    // let remainder = addr % align;
    // if remainder == 0 {
    //     addr // addr alreadyaddr aligned
    // } else {
    //     addr - remainder + align
    // }

    (addr + align - 1) & !(align - 1)

    // let offset = (addr as *const u8).align_offset(align);
    // addr + offset
}

pub fn map_physical_to_virtual_address(
    physical_address: u64,
    virtual_address: u64,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    mapper: &mut impl Mapper<Size4KiB>,
) -> u64 {
    use x86_64::structures::paging::PageTableFlags as Flags;
    let virt_addr = VirtAddr::new(virtual_address);
    let page = Page::containing_address(virt_addr);
    let frame = PhysFrame::containing_address(PhysAddr::new(physical_address));
    let offset = physical_address - frame.start_address().as_u64();

    log::info!(
        "mapping phys: {:x} to {:x}, physcial frame start address: {:?}, virtual frame start address: {:?}",
        physical_address,
        virtual_address,
        frame.start_address(),
        page.start_address(),
    );

    let flags = Flags::PRESENT | Flags::WRITABLE;
    let map_to_result = unsafe { mapper.map_to(page, frame, flags, frame_allocator) };
    map_to_result.expect("map_to failed").flush();

    offset
}

pub fn unmap_virtual_address(virtual_address: u64, mapper: &mut impl Mapper<Size4KiB>) {
    mapper
        .unmap(Page::containing_address(VirtAddr::new(virtual_address)))
        .expect("unmap failed");
}
