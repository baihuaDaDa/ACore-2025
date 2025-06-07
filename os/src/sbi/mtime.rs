use core::ptr::{read_volatile, write_volatile};
use crate::config::{MMIO_CLINT_BASE, MTIME_OFFSET, MTIMECMP_OFFSET};
use riscv::register::{mie, mip, mhartid, mtvec, mstatus};

pub fn init_timer() {
    unsafe { 
        mtvec::write(m_timer_interrupt as usize, mtvec::TrapMode::Direct); // set M-mode trap handler
        mstatus::set_mie(); // enable M-mode interrupt
    }
}

#[repr(C)]
pub struct SbiRet {
    pub error: usize,
    pub value: usize,
}

impl SbiRet {
    pub fn success(value: usize) -> Self {
        Self { error: 0, value }
    }
}

pub fn sbi_set_timer(stime_value: u64) -> SbiRet {
    let hart_id = mhartid::read();
    // set mtimecmp
    unsafe {
        mie::clear_mtimer(); // disable M-mode timer interrupt
        let mtimecmp_addr = (MMIO_CLINT_BASE + MTIMECMP_OFFSET + 8 * hart_id) as *mut u64;
        write_volatile(mtimecmp_addr, stime_value);
        mip::clear_stimer(); // clear S-mode timer interrupt pending
        mie::set_mtimer(); // enable M-mode timer interrupt
    }
    SbiRet::success(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn m_timer_interrupt() {
    unsafe {
        mip::set_stimer(); // forward the M-mode timer interrupt to S-mode
        mie::clear_mtimer(); // disable M-mode timer interrupt until the next timer interrupt
    }
}
