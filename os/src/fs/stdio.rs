use crate::fs::File;
use crate::mm::UserBuffer;
use crate::sbi::console_getchar;
use crate::task::suspend_current_and_run_next;

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl File for Stdin {
    fn readable(&self) -> bool { true }
    fn writable(&self) -> bool { false }
    fn read(&self, mut buf: UserBuffer) -> usize {
        assert_eq!(buf.len(), 1, "Only support len = 1 in sys_read!");
        let mut c: u8;
        loop {
            c = console_getchar();
            if c == 0 {
                suspend_current_and_run_next();
                continue;
            } else {
                break;
            }
        }
        unsafe { buf.buffers[0].as_mut_ptr().write_volatile(c); }
        1
    }
    fn write(&self, _buf: UserBuffer) -> usize {
        panic!("Cannot write to stdin!");
    }
}

impl File for Stdout {
    fn readable(&self) -> bool { false }
    fn writable(&self) -> bool { true }
    fn read(&self, _buf: UserBuffer) -> usize {
        panic!("Cannot read from stdout!");
    }
    fn write(&self, buf: UserBuffer) -> usize {
        for buffer in buf.buffers.iter() {
            print!("{}", core::str::from_utf8(*buffer).unwrap());
        }
        buf.len()
    }
}

impl File for Stderr {
    fn readable(&self) -> bool { false }
    fn writable(&self) -> bool { true }
    fn read(&self, _buf: UserBuffer) -> usize {
        panic!("Cannot read from stdout!");
    }
    fn write(&self, buf: UserBuffer) -> usize {
        for buffer in buf.buffers.iter() {
            print!("{}", core::str::from_utf8(*buffer).unwrap());
        }
        buf.len()
    }
}