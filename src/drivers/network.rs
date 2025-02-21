use core::sync::atomic::AtomicUsize;

use volatile::Volatile;
use x86_64::{
    instructions::port::Port,
    structures::paging::{FrameAllocator, Mapper, Size4KiB},
};

use crate::{
    allocator::{map_physical_to_virtual_address, unmap_virtual_address},
    println,
};

use super::{
    pci::{search_device, PCIConfigRegister, PCIDevice, PCIDeviceQuery},
    virtio::VirtioBlockDevice,
};

static NETWORK_INTERRUPT_NO: AtomicUsize = AtomicUsize::new(0);

const RTL_NETWORK_INTERFACE: (u16, u16) = (0x8139, 0x10EC);

const VIRTIO_NET_VENDOR_ID_AND_DEVICE_ID: (u16, u16) = (0x1af4, 0x1000);

// pub fn get_network_device() -> Option<Box<PhyNetDevType>> {
//     let (device_id, vendor_id) = RTL_NETWORK_INTERFACE;
// }

pub fn get_virtio_network_device() -> Option<VirtioBlockDevice> {
    let (vendor_id, device_id) = VIRTIO_NET_VENDOR_ID_AND_DEVICE_ID;
    let device = search_device(vendor_id, device_id)?;

    Some(VirtioBlockDevice::new(device))
}

pub struct VirtioNetworkDevice {
    pci_device: PCIDevice,
}

impl VirtioNetworkDevice {
    pub fn find_mmio_bar(
        &self,
        frame_allocator: &mut impl FrameAllocator<Size4KiB>,
        mapper: &mut impl Mapper<Size4KiB>,
    ) {
        for bar in self.pci_device.bars {
            if bar != 0 && bar & 6 == 0 {
                let virtual_address: u64 = 0x_4444_4440_0000;

                let offset = map_physical_to_virtual_address(
                    bar as u64,
                    virtual_address,
                    frame_allocator,
                    mapper,
                );

                let base_address = virtual_address + offset;
                unsafe {
                    // let ptr = (virtual_address + offset) as *mut u32;
                    // let initial_value = *ptr;

                    // *ptr = 0xFFFFFFFF;
                    // let value = unsafe { *ptr };
                    // log::info!(
                    //     "inital value from bar={} {}, final value {}",
                    //     bar,
                    //     initial_value,
                    //     value
                    // );

                    let magic_address = (base_address + 0x0) as *const u32;
                    let magic = core::ptr::read_volatile(magic_address);

                    // let magic = *((base_address + 0x0) as *const u32);

                    let control_address = (base_address + 0x14) as *mut u32;
                    let device_status = core::ptr::read_volatile(control_address);
                    log::info!("Device Status: {}, magic: {}", device_status, magic);
                    if device_status == 0 {
                        core::ptr::write_volatile(control_address, 0x0);
                        core::ptr::write_volatile(control_address, 0x1);
                        core::ptr::write_volatile(control_address, 0x3);
                    }
                    let device_status = *((base_address + 0x14) as *const u32);
                    log::info!("Device Status: {}", device_status);
                }

                unmap_virtual_address(virtual_address, mapper);
            }
        }
    }

    pub fn is_mmio_enabled(&self) {
        let value = unsafe {
            PCIDeviceQuery::MMIO.query(
                self.pci_device.bus,
                self.pci_device.dev,
                self.pci_device.func,
            )
        };

        log::info!("MMIO config value {:x}", value);
    }
}
