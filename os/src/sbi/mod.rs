#![allow(unused)]

mod uart;
mod power_off;
mod mtime;

use power_off::power_off;
use uart::UART;

pub fn init_uart() {
    UART.exclusive_access().init();
}

pub fn init_timer() {
    mtime::init_timer();
}

pub fn console_putchar(c: u8) {
    UART.exclusive_access().send(c);
}

pub fn console_getchar() -> u8 {
    UART.exclusive_access().recv()
}

pub fn shutdown(failure: bool) -> ! {
    power_off(failure)
}