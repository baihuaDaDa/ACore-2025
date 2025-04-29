mod virtio_blk;

use crate::board::BlockDeviceImpl;
use alloc::sync::Arc;
use lazy_static::lazy_static;
use easy_fs::BlockDevice;
pub use virtio_blk::VirtIOBlock;

lazy_static! {
    pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = Arc::new(BlockDeviceImpl::new());
}