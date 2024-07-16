use core::sync::atomic::AtomicUsize;

static NETWORK_INTERRUPT_NO: AtomicUsize = AtomicUsize::new(0);

const RTL_NETWORK_INTERFACE: (u16, u16) = (0x8139, 0x10EC);

// pub fn get_network_device() -> Option<Box<PhyNetDevType>> {
//     let (device_id, vendor_id) = RTL_NETWORK_INTERFACE;
// }
