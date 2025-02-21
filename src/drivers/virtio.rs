use x86_64::{
    structures::paging::{FrameAllocator, OffsetPageTable, Page, PhysFrame, Translate},
    PhysAddr, VirtAddr,
};

use crate::memory::{create_mapping, BootInfoFrameAllocator};

use super::pci::PCIDevice;

pub struct VirtioBlockDevice {
    device: PCIDevice,
    bar: u64,
}

impl VirtioBlockDevice {
    pub fn new(device: PCIDevice) -> Self {
        Self { device, bar: 0 }
    }

    pub fn setup(&self) {
        // size_t bar = 0;
        // uint32_t bar0 = <read_bar0>
        // uint32_t type = (bar0 >> 1) & 0x3;
        // if(typ == 0) {
        // // 64 bit
        // uint32_t bar1 = <read_bar1>
        // bar = (bar0 & 0xFFFFFFF0) | ((uint64_t)bar1 << (uint64_t)32);
        // } else if (typ == 2) {
        // // 32 bit memory I/O
        // bar = bar0 & 0xFFFFFFF0;
        // } else {
        // kpanic("Unsupported type: %d",type);
        // }

        let bar0 = self.device.bars[0];
        let bar1 = self.device.bars[1];
        let bar4 = self.device.bars[4];

        let typ = (bar0 >> 1) & 0x3;

        let mut bar = 0u64;

        if typ == 0 {
            bar = (((bar0 as u64) & 0xFFFFFFF0) + (((bar1 as u64) & 0xFFFFFFFF) << 32));

            // bar = ((bar0 as u64) & 0xFFFFFFF0) | ((bar1 as u64) << 32);
            log::info!("bar is 64 bit");
        } else if typ == 2 {
            bar = (bar0 as u64) & 0xFFFFFFF0;
            log::info!("bar is 32 bit");
        } else {
            log::error!("unsupported type {}", typ);
        }
        if bar != 0 {
            unsafe {
                log::info!("Device Status reading from bar 0x{:x}", bar);
                let value = *(bar as *const u32);
                log::info!("Device Status from bar 0x{:x}: 0x{:x}", bar, value);
            }
        }

        // unsafe {
        //     *((bar0 + 0x12) as *mut u32) = 0x01;
        //     *((bar0 + 0x12) as *mut u32) = 0x03;
        //     *((bar0 + 0x0e) as *mut u32) = 0x00000000;
        //     *((bar0 + 0x08) as *mut u32) = 0x00000100;
        //     *((bar0 + 0x0e) as *mut u32) = 0x00000001;
        //     *((bar0 + 0x08) as *mut u32) = 0x00000120;

        //     log::info!("Device Status: 0x{:x}", *(bar1 as *const u32));
        //     log::info!("Device Status: 0x{:x}", *(bar4 as *const u32));
        // }
    }

    pub fn map_bars_to_virtual_addresses(
        &self,
        mapper: &mut OffsetPageTable<'static>,
        frame_allocator: &mut BootInfoFrameAllocator,
    ) {
        for bar in self.device.bars {
            if bar != 0 {
                let virt_addr = VirtAddr::new(bar as u64);
                let page = Page::containing_address(virt_addr);
                if mapper.translate_addr(page.start_address()).is_some() {
                    log::error!(
                        "virtual address is already in use 0x{:x}",
                        page.start_address(),
                    );
                    continue;
                }

                let phys_frame = PhysFrame::containing_address(PhysAddr::new(bar as u64));

                create_mapping(page, phys_frame, mapper, frame_allocator);
            }
        }

        let bar0 = self.device.bars[0];
        let bar1 = self.device.bars[1];
        let bar4 = self.device.bars[4];

        let typ = (bar0 >> 1) & 0x3;

        let mut bar = 0u64;

        if typ == 0 {
            bar = (((bar0 as u64) & 0xFFFFFFF0) + (((bar1 as u64) & 0xFFFFFFFF) << 32));

            // bar = ((bar0 as u64) & 0xFFFFFFF0) | ((bar1 as u64) << 32);
            log::info!("bar is 64 bit");
        } else if typ == 2 {
            bar = (bar0 as u64) & 0xFFFFFFF0;
            log::info!("bar is 32 bit");
        } else {
            log::error!("unsupported type {}", typ);
        }
        if bar != 0 {
            let virt_addr = VirtAddr::new(bar as u64);
            let page = Page::containing_address(virt_addr);
            if mapper.translate_addr(page.start_address()).is_some() {
                log::error!(
                    "virtual address is already in use 0x{:x}",
                    page.start_address(),
                );
                return;
            }

            let phys_frame = PhysFrame::containing_address(PhysAddr::new(bar as u64));

            create_mapping(page, phys_frame, mapper, frame_allocator);

            unsafe {
                log::info!("Device Status reading from bar 0x{:x}", bar);
                let value = *(bar as *const u32);
                log::info!("Device Status from bar 0x{:x}: 0x{:x}", bar, value);
            }
        }
    }
}
