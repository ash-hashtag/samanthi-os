use uart_16550::SerialPort;

const QEMU_SHUTDOWN: (usize, u16) = (0x604, 0x2000);
const VIRTUAL_BOX_SHUTDOWN: (usize, u16) = (0x4004, 0x3400);
const LEGACY_QEMU_SHUTDOWN: (usize, u16) = (0xb004, 0x2000);

const KBD_CONTROLLER_REBOOT: (usize, u8) = (0x64, 0xFE);

// pub unsafe fn shutdown() {
//     SerialPort::new(QEMU_SHUTDOWN.0).send(QEMU_SHUTDOWN.1);
//     SerialPort::new(VIRTUAL_BOX_SHUTDOWN.0).send(VIRTUAL_BOX_SHUTDOWN.1);
//     SerialPort::new(LEGACY_QEMU_SHUTDOWN.0).send(LEGACY_QEMU_SHUTDOWN.1);
// }

// pub unsafe fn reboot() {
//     SerialPort::new(KBD_CONTROLLER_REBOOT.0).send(KBD_CONTROLLER_REBOOT.1);

//     shutdown();
// }
