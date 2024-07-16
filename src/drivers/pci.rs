use spin::Mutex;

extern crate alloc;
use alloc::vec::Vec;
use x86_64::instructions::port::Port;

use bit_field::BitField;

use crate::println;

/// Refers to the address of PCI data port in PCI config space.
const PCI_DATA_PORT: u16 = 0xCFC;

/// Refers to the address of PCI address port in PCI config space.
const PCI_ADDRESS_PORT: u16 = 0xCF8;

/// Refers to the base address which is ORed with device specific ID, bus and function
const PCI_BASE_ADDR: usize = 0x80000000;

/// number of bus lines in PCI
const MAX_BUS: usize = 256;

/// number of devices per each bus
const MAX_DEVICES_PER_BUS: usize = 32;

/// number of functions per device
const MAX_FUNCTIONS_PER_DEVICE: usize = 8;

/// if this flag is set, then the device is a multi-function device
const FLAG_MULTIFUNCTION_DEVICE: usize = 0x80;

type OnEntryCallback = fn(bus: u8, dev: u8, func: u8);

#[derive(Clone)]
pub struct PCIConfigRegister {
    pub address_line: Port<u32>,
    pub data_line: Port<u32>,
    pub dev_addr: u32,
}

impl PCIConfigRegister {
    #[inline]
    fn get_address(bus: u8, dev: u8, func: u8, offset: u8) -> u32 {
        // https://wiki.osdev.org/PCI#Configuration_Space_Access_Mechanism_.231
        PCI_BASE_ADDR as u32
            | ((bus as u32) << 16)
            | ((dev as u32) << 11 as u32)
            | ((func as u32) << 8 as u32)
            | ((offset as u32) & 0xFC)
    }

    pub fn new(bus: u8, dev: u8, func: u8, offset: u8) -> Self {
        PCIConfigRegister {
            address_line: Port::new(PCI_ADDRESS_PORT),
            data_line: Port::new(PCI_DATA_PORT),
            dev_addr: Self::get_address(bus, dev, func, offset),
        }
    }

    pub unsafe fn read_config(&mut self) -> u32 {
        self.address_line.write(self.dev_addr);
        self.data_line.read()
    }

    pub unsafe fn write_config(&mut self, data: u32) {
        self.address_line.write(self.dev_addr);
        self.data_line.write(data);
    }
}

/// Handles different types of queries on PCI devices.
pub enum PCIDeviceQuery {
    DeviceID,
    VendorID,
    HeaderType,
}

impl PCIDeviceQuery {
    pub unsafe fn query(&self, bus: u8, dev: u8, func: u8) -> u16 {
        match self {
            Self::VendorID => PCIConfigRegister::new(bus, dev, func, 0x00)
                .read_config()
                .get_bits(0..16) as u16,
            Self::DeviceID => PCIConfigRegister::new(bus, dev, func, 0x00)
                .read_config()
                .get_bits(16..32) as u16,
            Self::HeaderType => PCIConfigRegister::new(bus, dev, func, 0x0C)
                .read_config()
                .get_bits(16..24) as u16,
        }
    }
}

#[derive(Clone)]
pub struct PCIDevice {
    pub bus: u8,
    pub dev: u8,
    pub func: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub bars: [u32; 6],
}

impl PCIDevice {
    pub unsafe fn new(bus: u8, dev: u8, func: u8) -> Self {
        let vendor_id = PCIDeviceQuery::VendorID.query(bus, dev, func);
        let device_id = PCIDeviceQuery::DeviceID.query(bus, dev, func);

        let mut bars: [u32; 6] = [0; 6];

        for idx in 0..6 {
            let offset = 0x10 + ((idx as u8) << 2);
            let mut config_reg = PCIConfigRegister::new(bus, dev, func, offset);
            bars[idx] = config_reg.read_config();
        }

        Self {
            bus,
            dev,
            func,
            vendor_id,
            device_id,
            bars,
        }
    }

    pub fn set_bus_mastering(&self) -> Self {
        self.clone()
    }
}

static PCI_DEVICES: Mutex<Vec<PCIDevice>> = Mutex::new(Vec::new());

pub fn search_device(vendor_id: u16, device_id: u16) -> Option<PCIDevice> {
    PCI_DEVICES
        .lock()
        .iter()
        .find(|&x| x.vendor_id == vendor_id && x.device_id == device_id)
        .cloned()
}

struct PCIDeviceProber;

impl PCIDeviceProber {
    pub unsafe fn probe_bus(bus: u8, callback: OnEntryCallback) {
        for dev in 0..MAX_DEVICES_PER_BUS {
            Self::probe_device(bus, dev as u8, callback);
        }
    }

    pub unsafe fn probe(callback: OnEntryCallback) {
        for bus in 0..MAX_BUS {
            Self::probe_bus(bus as u8, callback);
        }
    }

    unsafe fn probe_device(bus: u8, dev: u8, callback: fn(u8, u8, u8)) {
        let vendor_id = PCIDeviceQuery::VendorID.query(bus, dev, 0);
        if Self::is_empty(vendor_id) {
            return;
        }

        callback(bus, dev, 0);

        let header_type = PCIDeviceQuery::HeaderType.query(bus, dev, 0);

        if Self::is_multi_function(header_type) {
            for func in 1..MAX_FUNCTIONS_PER_DEVICE {
                let vendor_id = PCIDeviceQuery::VendorID.query(bus, dev, func as u8);
                if !Self::is_empty(vendor_id) {
                    callback(bus, dev, func as u8);
                }
            }
        }
    }

    fn is_empty(vendor_id: u16) -> bool {
        vendor_id == 0xFFFF
    }

    fn is_multi_function(header_type: u16) -> bool {
        header_type & FLAG_MULTIFUNCTION_DEVICE as u16 != 0
    }
}

fn on_device_callback(bus: u8, dev: u8, func: u8) {
    let pci_device = unsafe { PCIDevice::new(bus, dev, func) };

    println!(
        "New PCI device added. bus={:x}, dev={:x}, func={:x}
         vendor_id={:x}, device_id={:x}",
        pci_device.bus, pci_device.dev, pci_device.func, pci_device.vendor_id, pci_device.device_id
    );

    PCI_DEVICES.lock().push(pci_device);
}

pub fn detect_devices() {
    unsafe { PCIDeviceProber::probe(on_device_callback) };
}
