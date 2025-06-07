#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate bitflags;
extern crate alloc;

#[path = "boards/qemu.rs"]
mod board;

mod lang_items;
mod sbi;
#[macro_use]
mod console;
mod sync;
mod trap;
mod syscall;
mod config;
mod task;
mod timer;
mod mm;
mod fs;
mod drivers;

use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));

#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    sbi::init();
    clear_bss();
    println!("[kernel] Hello, world!");
    mm::init();
    task::add_initproc();
    println!("after initproc!");
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    fs::list_apps();
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}

fn clear_bss() {
    unsafe extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe { (a as *mut u8).write_volatile(0) }
    });
}

