pub const CLOCK_FREQ: usize = 12500000;
pub const MEMORY_END: usize = 0x8800_0000;
pub const CORE_NUM: usize = 4;

// MMIO
// for peripherals
pub const MMIO_VIRT_IO: &[(usize, usize)] = &[
    (0x1000_1000, 0x1000),
];
// for uart
pub const MMIO_VIRT_UART: (usize, usize) = (0x1000_0000, 0x100);
// for shutdown
pub const MMIO_VIRT_TEST: (usize, usize) = (0x10_0000, 0x1000);
// for timer
pub const MMIO_CLINT_BASE: usize = 0x2000000;
pub const MTIME_OFFSET: usize = 0xBFF8;
pub const MTIMECMP_OFFSET: usize = 0x4000;

// for shutdown
pub const FINISHER_FAIL: u32 = 0x3333;
pub const FINISHER_PASS: u32 = 0x5555;

pub type BlockDeviceImpl = crate::drivers::block::VirtIOBlock;