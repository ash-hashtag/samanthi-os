use uart_16550::SerialPort;

pub struct UART {
    pub port_0: SerialPort,
    pub port_1: SerialPort,
}
