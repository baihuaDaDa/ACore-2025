use crate::config::{FINISHER_FAIL, FINISHER_PASS, MMIO_VIRT_TEST};

pub fn power_off(failure: bool) -> ! {
    // write in the first 4 bytes of VIRT_TEST
    let power_off = MMIO_VIRT_TEST.0 as *mut u32;
    unsafe {
        if !failure {
            *power_off = FINISHER_PASS;
        } else {
            *power_off = FINISHER_FAIL;
        }
    }
    unreachable!()
}