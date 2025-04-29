#![no_std]

extern crate alloc;

mod block_dev;
mod block_cache;
mod layout;
mod bitmap;
mod efs;
mod vfs;

pub const BLOCK_SIZE: usize = 512;
pub use block_dev::BlockDevice;
pub use vfs::Inode;
pub use efs::EasyFileSystem;;