mod address;
mod page_table;
mod frame_allocator;
mod memory_set;
mod heap_allocator;

pub use memory_set::{KERNEL_SPACE, MemorySet, MapPermission, kernel_token};
pub use address::{StepByOne, PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
pub use page_table::{PageTable, translated_byte_buffer, translated_str, translated_ref, translated_refmut, UserBuffer};
pub use frame_allocator::{frame_alloc, frame_dealloc, FrameTracker};

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().activate();
}