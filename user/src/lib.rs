#![no_std]
#![feature(linkage)]

mod syscall;
#[macro_use]
pub mod console;
mod lang_items;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    init_heap();
    exit(main());
    panic!("unreachable after sys_exit!");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
fn main() -> i32 {
    panic!("Cannot find main!");
}

use core::alloc::Layout;
use core::ptr::addr_of_mut;
use buddy_system_allocator::LockedHeap;

const USER_HEAP_SIZE: usize = 16384;

static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();
#[alloc_error_handler]
pub fn handle_alloc_error(layout: Layout) -> ! {
    panic!("Heap allocation error: layout = {:?}", layout);
}

fn init_heap() {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(addr_of_mut!(HEAP_SPACE) as usize, USER_HEAP_SIZE);
    }
}


use syscall::*;
pub fn read(fd: usize, buf: &mut [u8]) -> isize { sys_read(fd, buf) }
pub fn write(fd: usize, buf: &[u8]) -> isize { sys_write(fd, buf) }
pub fn exit(exit_code: i32) -> isize { sys_exit(exit_code) }
pub fn yield_() -> isize { sys_yield() }
pub fn get_time() -> isize { sys_get_time() }
pub fn fork() -> isize { sys_fork() }
pub fn exec(path: &str) -> isize { sys_exec(path) }
pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut _) {
            -2 => { yield_(); }
            exit_pid => return exit_pid,
        }
    }
}
pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid as isize, exit_code) {
            -2 => { yield_(); }
            exit_pid => return exit_pid,
        }
    }
}