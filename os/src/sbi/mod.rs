#![allow(unused)]

mod uart;
mod power_off;

use core::fmt;
use core::fmt::Write;
use power_off::power_off;
use uart::UART;

pub fn console_putchar(c: u8) {
    UART.exclusive_access().send(c);
}

pub fn console_getchar() -> u8 {
    UART.exclusive_access().recv()
}

pub fn shutdown(failure: bool) -> ! {
    power_off(failure)
}

pub fn set_timer(timer: usize) {
    sbi_rt::set_timer(timer as _);
}
