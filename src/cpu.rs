use futures_util::Future;
use x86_64::instructions::port::Port;

const IO_WAIT_PORT: u16 = 0x80;
