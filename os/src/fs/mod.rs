mod inode;
mod stdio;
mod pipe;

use crate::mm::UserBuffer;

pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
}

pub use inode::{OpenFlags, open_file, list_apps};
pub use stdio::{Stdin, Stdout, Stderr};
pub use pipe::make_pipe;