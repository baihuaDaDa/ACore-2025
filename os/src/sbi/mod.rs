#![allow(unused)]

use crate::sbi::power_off::power_off;

mod uart;
mod power_off;

pub fn console_putchar(c: usize) {
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(c);
}

pub fn console_getchar() -> usize {
    #[allow(deprecated)]
    sbi_rt::legacy::console_getchar()
}

pub fn shutdown(failure: bool) -> ! {
    power_off(failure)
}

pub fn set_timer(timer: usize) {
    sbi_rt::set_timer(timer as _);
}
