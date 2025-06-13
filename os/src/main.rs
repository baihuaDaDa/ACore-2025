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

use riscv::register::{mstatus, mepc, pmpaddr0, pmpcfg0, satp};

use core::arch::{asm, global_asm};
global_asm!(include_str!("entry.asm"));

/// initialize SBI and enter S-mode from M-mode.
#[unsafe(no_mangle)]
pub fn rust_boot() -> ! {
    unsafe { mstatus::set_mpp(mstatus::MPP::Supervisor) }; // set MPP to S-mode for privilege change
    mepc::write(rust_main as usize); // set the entry point of S-mode
    satp::write(0); // disable paging in S-mode
    pmpaddr0::write(0x3fffffffffffffusize); // define a full range of physical memory
    pmpcfg0::write(0xf); // set full physical memory access (R|W|X|NAPOT) for S-mode
    unsafe { asm!(
        // set machine exception delegation registers
        "csrw medeleg, {medeleg}",
        "csrw mideleg, {mideleg}",
        medeleg = in(reg) 0xffff,
        mideleg = in(reg) 0xffff,
    )};
    sbi::init_timer();
    unsafe { asm!(
        // return to S-mode
        "mret",
        options(noreturn),
    )}
}

/// kernel entry
#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();
    sbi::init_uart();
    println!("[kernel] Hello, world!");
    mm::init();
    task::add_initproc();
    println!("after initproc!");
    trap::init();
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

