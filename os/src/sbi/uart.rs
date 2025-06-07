use crate::config::{MMIO_VIRT_UART, UART_DIVISOR};
use bitflags::*;
use core::sync::atomic::{AtomicU8, Ordering};
use lazy_static::lazy_static;
use crate::sync::UPSafeCell;

macro_rules! wait_for {
    ($condition:expr) => {
        while !($condition) {
            core::hint::spin_loop();
        }
    };
}

bitflags! {
    struct InterruptEnable: u8 {
        const RX_AVAILABLE = 1 << 0; // Receiver Data Available
        const TX_EMPTY = 1 << 1; // Transmitter Holding Register Empty
    }
    struct FifoControl: u8 {
        const ENABLE = 1 << 0; // Enable FIFO
        const CLEAR_RX_FIFO = 1 << 1; // Clear RX FIFO
        const CLEAR_TX_FIFO = 1 << 2; // Clear TX FIFO
        const TRIGGER_14 = 0b11 << 6; // Trigger level 14 bytes
    }
    struct LineControl: u8 {
        const DATA_8 = 0b11; // 8 data bits, no parity, 1 stop bit
        const DLAB_ENABLE = 1 << 7; // Divisor Latch Access Bit
    }
    struct ModemControl: u8 {
        const DATA_TERMINAL_READY = 1 << 0; // Data Terminal Ready
        const AUXILIARY_OUTPUT_2 = 1 << 3; // Auxiliary Output 2
    }
    struct LineStatus: u8 {
        const INPUT_AVAILABLE = 1 << 0; // Input Data Available
        const OUTPUT_EMPTY = 1 << 5; // Output Holding Register Empty
    }
}

const DLL: u8 = UART_DIVISOR as u8;
const DLM: u8 = (UART_DIVISOR >> 8) as u8;

#[repr(C)]
struct ReadPort {
    rbr: AtomicU8, // Receiver Buffer Register
    ier: AtomicU8, // Interrupt Enable Register
    iir: AtomicU8, // Interrupt Identification Register
    lcr: AtomicU8, // Line Control Register
    mcr: AtomicU8, // Modem Control Register
    lsr: AtomicU8, // Line Status Register
    msr: AtomicU8, // Modem Status Register
    scr: AtomicU8, // Scratch Register
}

#[repr(C)]
struct WritePort {
    thr: AtomicU8, // Transmitter Holding Register
    ier: AtomicU8, // Interrupt Enable Register
    fcr: AtomicU8, // FIFO Control Register
    lcr: AtomicU8, // Line Control Register
    mcr: AtomicU8, // Modem Control Register
    _factory_test: AtomicU8, // Factory Test
    _not_used: AtomicU8, // Not used
    scr: AtomicU8, // Scratch Register
}

#[repr(C)]
struct DivisorLatch {
    dll: AtomicU8, // Divisor Latch LSB
    dlm: AtomicU8, // Divisor Latch MSB
}

pub struct UartRaw {
    pub base: usize,
    read_port: &'static mut ReadPort,
    write_port: &'static mut WritePort,
    divisor_latch: &'static mut DivisorLatch,
}

impl UartRaw {
    pub fn new(base: usize) -> Self {
        Self {
            base,
            read_port: unsafe { &mut *(base as *mut ReadPort) },
            write_port: unsafe { &mut *(base as *mut WritePort) },
            divisor_latch: unsafe { &mut *(base as *mut DivisorLatch) },
        }
    }

    pub fn init(&mut self) {
        self.write_port.ier.store(InterruptEnable::empty().bits, Ordering::Release); // Disable all interrupts
        self.write_port.lcr.store(LineControl::DLAB_ENABLE.bits, Ordering::Release); // Enable DLAB
        self.divisor_latch.dll.store(DLL, Ordering::Release); // Set Divisor Latch LSB for baud rate 38.4K
        self.divisor_latch.dlm.store(DLM, Ordering::Release); // Set Divisor Latch MSB for baud rate 38.4K
        self.write_port.lcr.store(LineControl::DATA_8.bits, Ordering::Release); // 8 data bits and disable DLAB
        self.write_port.fcr.store(
            (FifoControl::ENABLE | FifoControl::TRIGGER_14).bits,
            Ordering::Release
        ); // Enable FIFO and set trigger level
        self.write_port.mcr.store(
            (ModemControl::DATA_TERMINAL_READY | ModemControl::AUXILIARY_OUTPUT_2).bits,
            Ordering::Release
        ); // Set Modem Control Register
        self.write_port.ier.store(
            (InterruptEnable::RX_AVAILABLE | InterruptEnable::TX_EMPTY).bits,
            Ordering::Release
        ); // Enable RX and TX interrupts
    }

    pub fn send(&self, byte: u8) {
        wait_for!((self.read_port.lsr.load(Ordering::Acquire) & LineStatus::OUTPUT_EMPTY.bits) != 0);
        self.write_port.thr.store(byte, Ordering::Release);
    }

    pub fn recv(&self) -> u8 {
        wait_for!((self.read_port.lsr.load(Ordering::Acquire) & LineStatus::INPUT_AVAILABLE.bits) != 0);
        self.read_port.rbr.load(Ordering::Acquire)
    }
}

lazy_static! {
    pub static ref UART: UPSafeCell<UartRaw> = unsafe { UPSafeCell::new(UartRaw::new(MMIO_VIRT_UART.0)) };
}