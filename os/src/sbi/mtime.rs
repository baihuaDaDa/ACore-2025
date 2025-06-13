use core::arch::global_asm;
use core::ptr::{read_volatile, write_volatile};
use crate::config::{MMIO_CLINT_BASE, MTIME_OFFSET, MTIMECMP_OFFSET, CORE_NUM, CLOCK_FREQ, TICKS_PER_SEC};
use riscv::register::{mie, mip, mhartid, mtvec, mstatus, mscratch};
use crate::timer::get_time;

global_asm!(include_str!("time.S"));

/// 0,1,2: for callee save
/// 3: for mtimecmp addr
/// 4: for time interval
#[unsafe(link_section = ".bss.stack")]
pub static mut M_TIME_SCRATCH: [[usize; 5]; CORE_NUM] = [[0; 5]; CORE_NUM];

pub fn init_timer() {
    unsafe extern "C" {
        fn __time_handler();
    }
    let hart_id = mhartid::read();
    unsafe {
        mtvec::write(__time_handler as usize, mtvec::TrapMode::Direct); // set M-mode trap handler
        let scratch = &mut M_TIME_SCRATCH[hart_id];
        scratch[3] = (MMIO_CLINT_BASE + MTIMECMP_OFFSET + 8 * hart_id) as usize; // set mtimecmp addr
        scratch[4] = CLOCK_FREQ / TICKS_PER_SEC; // set time interval
        mscratch::write(scratch.as_mut_ptr() as usize); // set mscratch to point to M_TIME_SCRATCH[hart_id]
        write_volatile(scratch[3] as *mut usize, scratch[4] + get_time()); // set initial mtimecmp value
        mstatus::set_mie(); // enable M-mode interrupt
        mie::set_mtimer(); // enable machine timer interrupt
    }
}